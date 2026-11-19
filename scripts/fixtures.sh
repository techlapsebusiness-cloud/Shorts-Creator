#!/usr/bin/env bash
# Regenerates gitignored media fixtures with FFmpeg lavfi sources.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/tests/fixtures"
mkdir -p "$out"

ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -i "testsrc2=size=64x64:rate=30:duration=1" \
  -pix_fmt yuv420p -c:v libx264 \
  "$out/testsrc.mp4"

ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -i "sine=frequency=440:sample_rate=48000:duration=1" \
  -c:a pcm_s16le \
  "$out/sine.wav"

# Stand-in for a speech clip until M07 brings a real transcript fixture.
ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -i "sine=frequency=220:sample_rate=48000:duration=1" \
  -af "volume=0.2,tremolo=5:0.7" \
  -c:a pcm_s16le \
  "$out/speech.wav"

echo "fixtures written to $out"
