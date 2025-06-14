#!/usr/bin/env sh
set -eu

trap 'echo "Interrupted"; exit 1' INT

# Determine the directory of this script (experiments folder)
DIR="$(cd "$(dirname "$0")" && pwd)"

pids=""

cargo build --bin pumpkin-solver --release

for model in "$DIR/models"/*.mzn; do
  # Extract the base name without extension (e.g., community-detection, nsite, vaccine)
  name="$(basename "$model" .mzn)"

  mkdir -p "$DIR/output/$name/"

  input_dir="$DIR/input/$name"
  [ -d "$input_dir" ] || continue
  [ -z "$(ls -A "$input_dir" 2>/dev/null)" ] && continue

  for input in "$input_dir"/*.fzn; do
    instance="$(basename "$input" .fzn)"


    rm "$DIR/output/$name/$instance.log" || true
    target/release/pumpkin-solver -s "$input" --gcc-propagation-method extended-resolution > "$DIR/output/$name/$instance.log"
    echo "$name: $instance"

    # pids="$pids $!"
  done
done

for pid in $pids; do
  wait "$pid"
done