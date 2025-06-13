#!/usr/bin/env sh
set -eu

# Determine the directory of this script (experiments folder)
DIR="$(cd "$(dirname "$0")" && pwd)"

pids=""

# Loop over all .mzn models
for model in "$DIR/models"/*.mzn; do
  # Extract the base name without extension (e.g., community-detection, nsite, vaccine)
  name="$(basename "$model" .mzn)"

  mkdir -p "$DIR/input/$name/"

  # Find all instance subdirectories matching this model name
  for inst_dir in "$DIR/instances_subset"/*"$name"*; do
    # Ensure it is a directory
    [ -d "$inst_dir" ] || continue

    # Loop over each .dzn data file in the instance directory
    for data in "$inst_dir"/*.dzn; do
      # Skip if no files found
      [ -e "$data" ] || continue

      instance="$(basename "$data" .dzn)"

      (
        minizinc -c --solver minizinc/pumpkin.msc "$model" "$data" --output-fzn-to-file "$DIR/input/$name/$instance.fzn" 2>/dev/null
        echo "Generated FlatZinc instance for model '$name' with data '$instance'"
      ) &
      pids="$pids $!"
    done
  done
done

for pid in $pids; do
  wait $pid
done