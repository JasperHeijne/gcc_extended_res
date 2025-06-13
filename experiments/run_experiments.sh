#!/usr/bin/env sh
set -eux

# Determine the directory of this script (experiments folder)
DIR="$(cd "$(dirname "$0")" && pwd)"

pids=""

cargo build --bin pumpkin-solver --release

for model in "$DIR/models"/*.mzn; do
  # Extract the base name without extension (e.g., community-detection, nsite, vaccine)
  name="$(basename "$model" .mzn)"

  mkdir -p "$DIR/output/$name/"

  for input in "$DIR/input/$name"/*.fzn; do
    instance="$(basename "$input" .fzn)"

    (
      rm "$DIR/output/$name/$instance.log" || true
      target/release/pumpkin-solver -s "$input" --gcc-propagation-method extended-resolution > "$DIR/output/$name/$instance.log"
      echo "$name: $instance"
    )
    # pids="$pids $!"
  done
done

for pid in $pids; do
  wait "$pid"
done