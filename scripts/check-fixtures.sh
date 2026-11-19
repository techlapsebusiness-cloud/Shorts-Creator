#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
fix="$root/tests/fixtures"

require_duration() {
  local file="$1"
  local dur
  dur="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$file")"
  awk -v d="$dur" 'BEGIN { exit !(d >= 0.95 && d <= 1.05) }' || {
    echo "duration out of range for $file: $dur" >&2
    exit 1
  }
}

codec="$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 "$fix/testsrc.mp4")"
width="$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 "$fix/testsrc.mp4")"
height="$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 "$fix/testsrc.mp4")"
rate="$(ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of csv=p=0 "$fix/testsrc.mp4")"

[[ "$codec" == "h264" ]] || { echo "expected h264, got $codec" >&2; exit 1; }
[[ "$width" == "64" ]] || { echo "expected width 64, got $width" >&2; exit 1; }
[[ "$height" == "64" ]] || { echo "expected height 64, got $height" >&2; exit 1; }
[[ "$rate" == "30/1" ]] || { echo "expected 30/1, got $rate" >&2; exit 1; }
require_duration "$fix/testsrc.mp4"

for audio in sine.wav speech.wav; do
  acodec="$(ffprobe -v error -select_streams a:0 -show_entries stream=codec_name -of csv=p=0 "$fix/$audio")"
  arate="$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate -of csv=p=0 "$fix/$audio")"
  [[ "$acodec" == "pcm_s16le" ]] || { echo "expected pcm_s16le for $audio, got $acodec" >&2; exit 1; }
  [[ "$arate" == "48000" ]] || { echo "expected 48000 Hz for $audio, got $arate" >&2; exit 1; }
  require_duration "$fix/$audio"
done

echo "fixtures ok"
