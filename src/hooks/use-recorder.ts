import { useCallback, useEffect, useRef, useState } from "react";

import { transcribe } from "@/lib/ai";

export type RecorderState = "idle" | "recording" | "transcribing";

interface UseRecorder {
  state: RecorderState;
  start: () => Promise<void>;
  stopAndTranscribe: () => Promise<string>;
  cancel: () => void;
}

export function useRecorder(): UseRecorder {
  const [state, setState] = useState<RecorderState>("idle");
  const recorderRef = useRef<MediaRecorder | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const chunksRef = useRef<Blob[]>([]);

  const cleanup = useCallback(() => {
    for (const track of streamRef.current?.getTracks() ?? []) {
      track.stop();
    }
    streamRef.current = null;
    recorderRef.current = null;
    chunksRef.current = [];
  }, []);

  useEffect(() => () => cleanup(), [cleanup]);

  const start = useCallback(async () => {
    if (!navigator.mediaDevices?.getUserMedia) {
      throw new Error("Microphone is not available in this environment");
    }
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    streamRef.current = stream;

    const mimeType = MediaRecorder.isTypeSupported("audio/webm;codecs=opus")
      ? "audio/webm;codecs=opus"
      : "audio/webm";

    const recorder = new MediaRecorder(stream, { mimeType });
    chunksRef.current = [];
    recorder.ondataavailable = (event) => {
      if (event.data.size > 0) chunksRef.current.push(event.data);
    };
    recorder.start();
    recorderRef.current = recorder;
    setState("recording");
  }, []);

  const stopAndTranscribe = useCallback(async () => {
    const recorder = recorderRef.current;
    if (!recorder) throw new Error("No active recording");

    setState("transcribing");

    const blob = await new Promise<Blob>((resolve) => {
      recorder.onstop = () => resolve(new Blob(chunksRef.current, { type: recorder.mimeType }));
      recorder.stop();
    });

    cleanup();

    try {
      const buffer = await blob.arrayBuffer();
      const text = await transcribe(new Uint8Array(buffer), blob.type);
      setState("idle");
      return text;
    } catch (err) {
      setState("idle");
      throw err;
    }
  }, [cleanup]);

  const cancel = useCallback(() => {
    if (recorderRef.current && recorderRef.current.state !== "inactive") {
      recorderRef.current.stop();
    }
    cleanup();
    setState("idle");
  }, [cleanup]);

  return { state, start, stopAndTranscribe, cancel };
}
