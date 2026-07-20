//! macOS system speech provider: Apple's `SFSpeechRecognizer` driven through
//! `objc2-speech`. Audio is fed to `SFSpeechURLRecognitionRequest` as a file
//! URL; `requiresOnDeviceRecognition` is set to `true` so the audio never
//! leaves the user's machine.
//!
//! All `objc2` Retained values and `block2` blocks are `!Send`, so every FFI
//! call site is confined inside `tokio::task::spawn_blocking` to keep the
//! command future `Send`.

use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use block2::{Block, RcBlock};
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_foundation::{NSError, NSLocale, NSString, NSURL};
use objc2_speech::{
    SFSpeechRecognitionResult, SFSpeechRecognizer, SFSpeechRecognizerAuthorizationStatus,
    SFSpeechURLRecognitionRequest,
};
use tempfile::NamedTempFile;

use crate::speech::error::{SpeechError, SpeechResult};
use crate::speech::metadata::SpeechProviderMetadata;
use crate::speech::provider::SpeechProvider;
use crate::speech::types::TranscribeResponse;

/// Upper bound on how long we wait for `SFSpeechRecognizer` to deliver a final
/// result. Without this, empty/too-short/silent clips (where the framework may
/// never invoke the result handler) would block the worker thread forever and
/// leave the UI stuck on "Transcribing…".
const RECOGNITION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub struct SystemSpeechMacosProvider {
    metadata: SpeechProviderMetadata,
}

impl SystemSpeechMacosProvider {
    pub fn new(metadata: SpeechProviderMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait]
impl SpeechProvider for SystemSpeechMacosProvider {
    fn metadata(&self) -> &SpeechProviderMetadata {
        &self.metadata
    }

    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> SpeechResult<TranscribeResponse> {
        let mime = mime_type.to_string();
        let lang = language.map(|s| s.to_string());
        let provider_id = self.metadata.id.to_string();

        tokio::task::spawn_blocking(move || transcribe_blocking(audio, &mime, lang.as_deref()))
            .await
            .map_err(|e| SpeechError::NativeApi(format!("blocking join: {e}")))?
            .map(|text| TranscribeResponse { text, provider_id })
    }

    async fn validate_availability(&self) -> SpeechResult<()> {
        tokio::task::spawn_blocking(validate_blocking)
            .await
            .map_err(|e| SpeechError::NativeApi(format!("blocking join: {e}")))?
    }
}

fn transcribe_blocking(
    audio: Vec<u8>,
    mime_type: &str,
    language: Option<&str>,
) -> SpeechResult<String> {
    ensure_authorized_blocking()?;

    if audio.is_empty() {
        return Err(SpeechError::NativeApi(
            "no audio was captured — try holding the record button a little longer".into(),
        ));
    }

    let suffix = mime_to_extension(mime_type);
    let temp = NamedTempFile::with_suffix(suffix).map_err(|e| SpeechError::Io(e.to_string()))?;
    std::fs::write(temp.path(), &audio).map_err(|e| SpeechError::Io(e.to_string()))?;

    let recognizer = build_recognizer(language)?;

    // SAFETY: Calling an Apple API method that returns a boolean. The
    // receiver is a valid Retained<SFSpeechRecognizer>; no aliasing.
    let available = unsafe { recognizer.isAvailable() };
    if !available {
        return Err(SpeechError::Unavailable);
    }

    let path_str = temp
        .path()
        .to_str()
        .ok_or_else(|| SpeechError::Io("temp path not valid UTF-8".into()))?;
    let ns_path = NSString::from_str(path_str);
    let url = NSURL::fileURLWithPath(&ns_path);

    let alloc = SFSpeechURLRecognitionRequest::alloc();
    // SAFETY: initWithURL is the standard Objective-C alloc/init pair on a
    // fresh Allocated<Self>; url is a valid Retained<NSURL>.
    let request = unsafe { SFSpeechURLRecognitionRequest::initWithURL(alloc, &url) };
    // SAFETY: Boolean setter on a valid SFSpeechURLRecognitionRequest.
    unsafe {
        request.setRequiresOnDeviceRecognition(true);
    }

    // The callback is Fn (may be invoked for partial + final results). We
    // hand the std::mpsc Sender out of an Arc<Mutex<Option<_>>> only on the
    // first final result, so the channel is closed exactly once.
    let (tx, rx) = std_mpsc::channel::<SpeechResult<String>>();
    let tx_slot = Arc::new(Mutex::new(Some(tx)));

    let block = {
        let slot = tx_slot.clone();
        RcBlock::new(
            move |result: *mut SFSpeechRecognitionResult, error: *mut NSError| {
                let payload = match decode_callback(result, error) {
                    CallbackPayload::Partial => return,
                    CallbackPayload::Final(v) => v,
                };
                if let Ok(mut guard) = slot.lock() {
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(payload);
                    }
                }
            },
        )
    };
    let block_ref: &Block<dyn Fn(_, _)> = &block;

    // SAFETY: recognitionTaskWithRequest:resultHandler: borrows the request
    // and block; the returned task is kept alive in `task` until the wait on
    // `rx` returns, so the session stays alive until the callback fires.
    let task = unsafe { recognizer.recognitionTaskWithRequest_resultHandler(&request, block_ref) };

    let outcome = match rx.recv_timeout(RECOGNITION_TIMEOUT) {
        Ok(result) => result,
        Err(std_mpsc::RecvTimeoutError::Timeout) => Err(SpeechError::NativeApi(
            "speech recognition timed out before producing a result".into(),
        )),
        Err(std_mpsc::RecvTimeoutError::Disconnected) => Err(SpeechError::NativeApi(
            "recognition channel closed unexpectedly".into(),
        )),
    };

    // Keep the recognition task alive until we have stopped waiting on it.
    drop(task);
    drop(temp);
    outcome
}

fn validate_blocking() -> SpeechResult<()> {
    ensure_authorized_blocking()?;
    let recognizer = build_recognizer(None)?;
    // SAFETY: same justification as in transcribe_blocking.
    let available = unsafe { recognizer.isAvailable() };
    if available {
        Ok(())
    } else {
        Err(SpeechError::Unavailable)
    }
}

enum CallbackPayload {
    Partial,
    Final(SpeechResult<String>),
}

fn decode_callback(result: *mut SFSpeechRecognitionResult, error: *mut NSError) -> CallbackPayload {
    // SAFETY: Pointers come from the Speech framework's callback invocation;
    // they are valid for the duration of this call. We treat either null
    // pointer or non-null error as a final outcome.
    unsafe {
        if !error.is_null() {
            let err = &*error;
            return CallbackPayload::Final(Err(SpeechError::NativeApi(
                err.localizedDescription().to_string(),
            )));
        }
        if result.is_null() {
            return CallbackPayload::Final(Err(SpeechError::NativeApi(
                "Speech framework returned null result and null error".into(),
            )));
        }
        let r = &*result;
        if !r.isFinal() {
            return CallbackPayload::Partial;
        }
        let transcription = r.bestTranscription();
        let text = transcription.formattedString().to_string();
        CallbackPayload::Final(Ok(text))
    }
}

fn build_recognizer(language: Option<&str>) -> SpeechResult<Retained<SFSpeechRecognizer>> {
    // SAFETY: Standard Apple constructors. `new` returns Retained<Self>,
    // `initWithLocale` returns Option<Retained<Self>> (nil when the locale
    // lacks a recogniser). Allocation is safe from any thread.
    let opt: Option<Retained<SFSpeechRecognizer>> = unsafe {
        match language {
            Some(lang) => {
                let lang_ns = NSString::from_str(lang);
                let locale = NSLocale::localeWithLocaleIdentifier(&lang_ns);
                let alloc = SFSpeechRecognizer::alloc();
                SFSpeechRecognizer::initWithLocale(alloc, &locale)
            }
            None => Some(SFSpeechRecognizer::new()),
        }
    };
    opt.ok_or(SpeechError::Unavailable)
}

fn ensure_authorized_blocking() -> SpeechResult<()> {
    // SAFETY: authorizationStatus is a class method returning a Copy enum.
    let status = unsafe { SFSpeechRecognizer::authorizationStatus() };

    if status == SFSpeechRecognizerAuthorizationStatus::Authorized {
        return Ok(());
    }
    if status == SFSpeechRecognizerAuthorizationStatus::Denied
        || status == SFSpeechRecognizerAuthorizationStatus::Restricted
    {
        return Err(SpeechError::PermissionDenied);
    }

    // NotDetermined: prompt the user (one-time system dialog). The Info.plist
    // entry NSSpeechRecognitionUsageDescription drives the prompt copy.
    let (tx, rx) = std_mpsc::channel::<SFSpeechRecognizerAuthorizationStatus>();
    let tx_slot = Arc::new(Mutex::new(Some(tx)));
    let block = {
        let slot = tx_slot.clone();
        RcBlock::new(move |status: SFSpeechRecognizerAuthorizationStatus| {
            if let Ok(mut guard) = slot.lock() {
                if let Some(sender) = guard.take() {
                    let _ = sender.send(status);
                }
            }
        })
    };
    let block_ref: &Block<dyn Fn(_)> = &block;
    // SAFETY: requestAuthorization invokes the block exactly once on a
    // private dispatch queue; the block reference is borrowed for that call.
    unsafe {
        SFSpeechRecognizer::requestAuthorization(block_ref);
    }
    let status = rx
        .recv()
        .map_err(|_| SpeechError::NativeApi("authorization handler dropped".into()))?;
    if status == SFSpeechRecognizerAuthorizationStatus::Authorized {
        Ok(())
    } else {
        Err(SpeechError::PermissionDenied)
    }
}

fn mime_to_extension(mime: &str) -> &'static str {
    let mime = mime.to_ascii_lowercase();
    if mime.contains("mp4") || mime.contains("m4a") || mime.contains("aac") {
        ".m4a"
    } else if mime.contains("wav") {
        ".wav"
    } else if mime.contains("mp3") || mime.contains("mpeg") {
        ".mp3"
    } else if mime.contains("aiff") {
        ".aiff"
    } else if mime.contains("caf") {
        ".caf"
    } else {
        // WKWebView's MediaRecorder default is audio/mp4 — best guess if MIME unclear.
        ".m4a"
    }
}
