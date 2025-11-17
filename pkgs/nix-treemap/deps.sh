#!/usr/bin/env bash

set -e

INPUT_PATH="$1"

PACKAGE_PATH=$(readlink -f "$INPUT_PATH")
OUTPUT_FILE="${2:-dependency-graph.json}"

echo "Analyzing dependencies for $PACKAGE_PATH..."

TEMP_DIR=$(mktemp -d)
DEPS_LIST="$TEMP_DIR/deps-list.txt"
SIZES_LIST="$TEMP_DIR/sizes-list.txt"
REFS_DIR="$TEMP_DIR/refs"

mkdir -p "$REFS_DIR"

echo "Collecting dependencies..."
nix-store -q --requisites "$PACKAGE_PATH" > "$DEPS_LIST"

echo "Collecting sizes..."
xargs nix path-info -s < "$DEPS_LIST" > "$SIZES_LIST"

echo "Building dependency tree..."
while read -r dep; do
  nix-store -q --references "$dep" > "$REFS_DIR/$(basename "$dep")"
done < "$DEPS_LIST"

echo "Generating JSON..."
{
  echo "{"
  echo "  \"name\": \"$(basename "$PACKAGE_PATH")\","
  echo "  \"path\": \"$PACKAGE_PATH\","

  ROOT_SIZE=$(grep "$PACKAGE_PATH" "$SIZES_LIST" | awk '{print $2}')
  ROOT_SIZE=${ROOT_SIZE:-0}
  echo "  \"size\": $ROOT_SIZE,"
  echo '  "children": ['
} > "$OUTPUT_FILE"

processed_deps=""

process_dependencies() {
  local parent="$1"
  local indent="$2"

  local refs_file
  refs_file="$REFS_DIR/$(basename "$parent")"

  if [[ ! -f $refs_file ]] || [[ ! -s $refs_file ]]; then
    return
  fi

  local first_child=true
  while read -r dep; do
    if ! grep -qF "$dep" "$DEPS_LIST"; then
      continue
    fi

    if [[ $processed_deps == *"$dep"* ]]; then
      continue
    fi
    processed_deps="$processed_deps $dep"

    local size
    size=$(grep -F "$dep" "$SIZES_LIST" | awk '{print $2}')
    size=${size:-0}

    {
      if [[ $first_child != "true" ]]; then
        echo ","
      fi
      echo "$indent{"
      echo "$indent  \"name\": \"$(basename "$dep")\","
      echo "$indent  \"path\": \"$dep\","
      echo "$indent  \"size\": $size,"
      echo "$indent  \"children\": ["
    } >> "$OUTPUT_FILE"
    first_child=false

    process_dependencies "$dep" "$indent    "

    {
      echo ""
      echo "$indent  ]"
      echo -n "$indent}"
    } >> "$OUTPUT_FILE"
  done < "$refs_file"
}

process_dependencies "$PACKAGE_PATH" "    "

{
  echo ""
  echo "  ]"
  echo "}"
} >> "$OUTPUT_FILE"

echo "Dependency graph written to $OUTPUT_FILE"
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"
