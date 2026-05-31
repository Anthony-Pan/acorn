const MUTE_KEY = "acorn:sounds-muted";
const CUSTOM_PREFIX = "acorn:sound:";

export type SoundKey = "chime" | "error" | "pop";

let ctx: AudioContext | null = null;

function getContext(): AudioContext | null {
  if (typeof window === "undefined") return null;
  if (ctx !== null) return ctx;
  const Ctor =
    window.AudioContext ??
    (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Ctor) return null;
  ctx = new Ctor();
  return ctx;
}

export function isMuted(): boolean {
  try {
    return localStorage.getItem(MUTE_KEY) === "true";
  } catch {
    return false;
  }
}

export function setMuted(value: boolean): void {
  try {
    if (value) localStorage.setItem(MUTE_KEY, "true");
    else localStorage.removeItem(MUTE_KEY);
  } catch {}
}

function customKey(key: SoundKey): string {
  return `${CUSTOM_PREFIX}${key}`;
}

/** The user's custom audio for an event, stored as a data URL, or null. */
export function getCustomSound(key: SoundKey): string | null {
  try {
    return localStorage.getItem(customKey(key));
  } catch {
    return null;
  }
}

export function setCustomSound(key: SoundKey, dataUrl: string): void {
  try {
    localStorage.setItem(customKey(key), dataUrl);
  } catch {}
}

export function clearCustomSound(key: SoundKey): void {
  try {
    localStorage.removeItem(customKey(key));
  } catch {}
}

function playCustom(key: SoundKey): boolean {
  const data = getCustomSound(key);
  if (!data) return false;
  try {
    const audio = new Audio(data);
    audio.volume = 0.7;
    void audio.play().catch(() => {});
    return true;
  } catch {
    return false;
  }
}

function tone(frequency: number, duration: number, gainPeak: number): void {
  const audio = getContext();
  if (!audio) return;
  const osc = audio.createOscillator();
  const gain = audio.createGain();
  osc.type = "sine";
  osc.frequency.value = frequency;
  const now = audio.currentTime;
  gain.gain.setValueAtTime(0, now);
  gain.gain.linearRampToValueAtTime(gainPeak, now + 0.01);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
  osc.connect(gain).connect(audio.destination);
  osc.start(now);
  osc.stop(now + duration);
}

export const sounds = {
  chime(): void {
    if (isMuted() || playCustom("chime")) return;
    tone(880, 0.18, 0.08);
    setTimeout(() => tone(1175, 0.22, 0.06), 90);
  },
  error(): void {
    if (isMuted() || playCustom("error")) return;
    tone(196, 0.32, 0.1);
  },
  pop(): void {
    if (isMuted() || playCustom("pop")) return;
    tone(523, 0.08, 0.05);
  },
};
