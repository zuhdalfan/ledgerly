"""Turn the recorded webm clips into mp4 (all chapters) and gif (focused ones).

    python tools/preview-record/encode.py
    python tools/preview-record/encode.py --gif-only

Reads `out/manifest.json` (written by record.py), trims the page-load lead-in
off each clip, and writes into `docs/preview/`. The overview gif used by the
README stays the HyperFrames composition at docs/screenshots/preview.gif.

Development tooling only.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / "out"
ROOT = HERE.parents[1]
# docs/preview is not created for you if you delete it; the encoder owns it.
DEST = ROOT / "docs" / "preview"

# Every chapter gets a gif (embed-able in a README) and an mp4 (smaller,
# higher quality, for the docs index). The gif is deliberately modest — a gif
# of continuous UI motion gets heavy fast.


def run(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(f"ffmpeg failed:\n{' '.join(cmd)}\n{result.stderr[-800:]}")


def to_mp4(src: pathlib.Path, dst: pathlib.Path, trim: float):
    run([
        "ffmpeg", "-y", "-loglevel", "error",
        "-i", str(src),
        "-ss", f"{trim:.3f}",
        "-c:v", "libx264", "-preset", "slow", "-crf", "20",
        "-pix_fmt", "yuv420p", "-movflags", "+faststart",
        str(dst),
    ])


def to_gif(src: pathlib.Path, dst: pathlib.Path, trim: float, width: int = 660, fps: int = 8):
    graph = (
        f"[0:v]fps={fps},scale={width}:-2:flags=lanczos,split[s0][s1];"
        "[s0]palettegen=max_colors=64:stats_mode=diff[p];"
        "[s1][p]paletteuse=dither=none:diff_mode=rectangle[out]"
    )
    run([
        "ffmpeg", "-y", "-loglevel", "error",
        "-i", str(src),
        "-ss", f"{trim:.3f}",
        "-filter_complex", graph, "-map", "[out]", "-loop", "0",
        str(dst),
    ])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gif-only", action="store_true")
    ap.add_argument("--mp4-only", action="store_true")
    args = ap.parse_args()

    manifest_path = OUT / "manifest.json"
    if not manifest_path.exists():
        raise SystemExit("no out/manifest.json — run record.py first")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    DEST.mkdir(parents=True, exist_ok=True)
    index = []
    for seg in manifest["segments"]:
        if seg["status"] != "ok" or not seg.get("file"):
            continue
        src = HERE / seg["file"]
        if not src.exists():
            print(f"skip {seg['id']}: missing {src}")
            continue
        trim = seg["trimStart"]
        entry = {**seg, "artifacts": []}
        if not args.gif_only:
            mp4 = DEST / f"{seg['id']}.mp4"
            to_mp4(src, mp4, trim)
            entry["artifacts"].append(f"{seg['id']}.mp4")
        if not args.mp4_only:
            gif = DEST / f"{seg['id']}.gif"
            to_gif(src, gif, trim)
            entry["artifacts"].append(f"{seg['id']}.gif")
        sizes = {a: round((DEST / a).stat().st_size / 1024, 1) for a in entry["artifacts"]}
        entry["sizesKB"] = sizes
        index.append(entry)
        print(f"{seg['id']:<20} -> {', '.join(f'{k} {v}KB' for k, v in sizes.items())}")

    (DEST / "manifest.json").write_text(json.dumps({"chapters": index}, indent=2), encoding="utf-8")
    print(f"\nWrote {len(index)} chapters to {DEST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
