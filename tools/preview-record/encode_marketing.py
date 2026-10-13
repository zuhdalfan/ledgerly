"""Encode the marketing cuts into docs/marketing/.

    python tools/preview-record/encode_marketing.py          # mp4 only
    python tools/preview-record/encode_marketing.py --gif    # also a gif fallback

Trims the page-load lead-in. MP4 is the primary format — platforms autoplay it
and it is a fraction of the size; the gif is opt-in for embeds that demand an
image (a gif of moving UI runs to megabytes).

Development tooling only.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / "out-marketing"
ROOT = HERE.parents[1]
DEST = ROOT / "docs" / "marketing"

# The gif is the embed fallback, so it is sized to the frame rather than the
# recording: wide for a page, narrow for social.
GIF_WIDTH = {"landscape": 900, "square": 720, "portrait": 540}


def run(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(f"ffmpeg failed:\n{' '.join(cmd)}\n{result.stderr[-800:]}")


def to_mp4(src: pathlib.Path, dst: pathlib.Path, trim: float):
    run([
        "ffmpeg", "-y", "-loglevel", "error", "-i", str(src), "-ss", f"{trim:.3f}",
        "-c:v", "libx264", "-preset", "slow", "-crf", "20", "-pix_fmt", "yuv420p",
        "-movflags", "+faststart", str(dst),
    ])


def to_gif(src: pathlib.Path, dst: pathlib.Path, trim: float, width: int):
    graph = (
        f"[0:v]fps=12,scale={width}:-2:flags=lanczos,split[s0][s1];"
        "[s0]palettegen=max_colors=128:stats_mode=diff[p];"
        "[s1][p]paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle[out]"
    )
    run([
        "ffmpeg", "-y", "-loglevel", "error", "-i", str(src), "-ss", f"{trim:.3f}",
        "-filter_complex", graph, "-map", "[out]", "-loop", "0", str(dst),
    ])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gif", action="store_true", help="also write a gif fallback")
    args = ap.parse_args()

    manifest_path = OUT / "manifest.json"
    if not manifest_path.exists():
        raise SystemExit("no out-marketing/manifest.json — run marketing.py first")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    DEST.mkdir(parents=True, exist_ok=True)
    index = []
    for seg in manifest["segments"]:
        if seg["status"] != "ok" or not seg.get("file"):
            continue
        src = HERE / seg["file"]
        if not src.exists():
            print(f"skip {seg['id']}: missing clip")
            continue
        stem = f"{seg['id']}-{seg['format']}"
        entry = {**seg, "artifacts": []}
        mp4 = DEST / f"{stem}.mp4"
        to_mp4(src, mp4, seg["trimStart"])
        entry["artifacts"].append(mp4.name)
        if args.gif:
            gif = DEST / f"{stem}.gif"
            to_gif(src, gif, seg["trimStart"], GIF_WIDTH.get(seg["format"], 720))
            entry["artifacts"].append(gif.name)
        entry["sizesKB"] = {
            a: round((DEST / a).stat().st_size / 1024, 1) for a in entry["artifacts"]
        }
        index.append(entry)
        print(f"{stem:<24} " + ", ".join(f"{k} {v}KB" for k, v in entry["sizesKB"].items()))

    (DEST / "manifest.json").write_text(json.dumps({"cuts": index}, indent=2), encoding="utf-8")
    print(f"\nWrote {len(index)} cuts to {DEST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
