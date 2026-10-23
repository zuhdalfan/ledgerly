"""Measure this pipeline's output-audio offset (rule S5, sound-design §4.6).

Extracts the render's audio, builds a per-frame energy envelope, and reports the
frame of the loudest transient near a set of known sharp SFX cues. The residual
(measured frame - designed frame) is the offset the render pipeline adds.

    python capture/measure_offset.py out/promo.mp4 672 512 306
"""

import array
import subprocess
import sys
import wave
import pathlib

FPS = 30
SR = 48000
FRAME = SR // FPS  # 1600 samples


def main():
    mp4 = sys.argv[1] if len(sys.argv) > 1 else "out/promo.mp4"
    designed = [int(a) for a in sys.argv[2:]] or [672, 512, 306]

    wav = pathlib.Path("out/_offset_probe.wav")
    subprocess.run(
        ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-i", mp4, "-vn", "-acodec", "pcm_s16le", "-ar", str(SR), "-ac", "1", str(wav)],
        check=True,
    )

    with wave.open(str(wav), "rb") as f:
        raw = f.readframes(f.getnframes())
    samples = array.array("h")
    samples.frombytes(raw)

    n_frames = len(samples) // FRAME
    energy = []
    for i in range(n_frames):
        seg = samples[i * FRAME : (i + 1) * FRAME]
        energy.append(sum(float(s) * s for s in seg) / max(1, len(seg)))

    residuals = []
    for target in designed:
        lo = max(0, target - 10)
        hi = min(n_frames, target + 11)
        if lo >= hi:
            continue
        window = energy[lo:hi]
        peak = max(range(len(window)), key=lambda k: window[k]) + lo
        residuals.append(peak - target)
        print(f"designed f{target} -> measured peak f{peak}  residual {peak - target:+d}f")

    if residuals:
        mean = sum(residuals) / len(residuals)
        print(f"\nmean residual {mean:+.2f}f over {len(residuals)} probes")
        print(f"suggested OUTPUT_AUDIO_OFFSET_F = {mean:+.2f}")

    wav.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
