// Turns the generated manifest into the small API the film uses. Nothing here
// assumes a track exists: a missing file means that layer is simply silent.

import { AUDIO_MANIFEST } from "./audio-manifest";

export const MUSIC: string | null = AUDIO_MANIFEST.music ?? null;

const CLIPS = AUDIO_MANIFEST.voiceover as Record<string, string | null>;

export const voClip = (id: string): string | null => CLIPS[id] ?? null;

export const hasVoiceover = Object.values(CLIPS).some((path) => path !== null);

export const usingScratchVoiceover = Object.values(CLIPS).some(
  (path) => path !== null && path.startsWith("audio/vo-scratch/")
);
