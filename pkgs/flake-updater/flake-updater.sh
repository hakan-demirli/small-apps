#!/usr/bin/env bash
set -uo pipefail

echo "[DEBUG] Checking upstream for latest NixOS version..."
TARGET_VERSION=$(git ls-remote --sort=-v:refname https://github.com/NixOS/nixpkgs 'nixos-??.??' | awk '{ sub("^.*/","",$2); print $2; exit}')

if [[ -z $TARGET_VERSION ]]; then
  echo "[ERROR]  Failed to fetch latest version."
  exit 1
fi

echo "[DEBUG] Target Version: $TARGET_VERSION"

find . -type f -name "flake.nix" -not -path "*/.git/*" -not -path "*/_deprecated/*" -print0 | while IFS= read -r -d '' flake_file; do
  dir=$(dirname "$flake_file")

  CURRENT_VERSION=$(grep -oE "nixos-[0-9]{2}\.[0-9]{2}" "$flake_file" | head -n1)

  if [[ -z $CURRENT_VERSION ]]; then
    echo "[WARN]  Could not determine version for $dir. Skipping."
    continue
  fi

  echo "------------------------------------------------"
  echo "[DEBUG] Processing: $dir ($CURRENT_VERSION)"

  VERSION_CHANGED=false

  if [[ $CURRENT_VERSION != "$TARGET_VERSION" ]]; then
    echo "[DEBUG] Version bump required: $CURRENT_VERSION -> $TARGET_VERSION"

    if [[ $OSTYPE == "darwin"* ]]; then
      sed -i '' "s/$CURRENT_VERSION/$TARGET_VERSION/g" "$flake_file"
    else
      sed -i "s/$CURRENT_VERSION/$TARGET_VERSION/g" "$flake_file"
    fi
    VERSION_CHANGED=true
  fi

  echo "[DEBUG] Running 'nix flake update'..."
  if (cd "$dir" && nix flake update &> /dev/null); then

    LOCK_CHANGED=false
    if ! git diff --quiet "$dir/flake.lock"; then
      LOCK_CHANGED=true
    fi

    if [[ $VERSION_CHANGED == "true" ]] || [[ $LOCK_CHANGED == "true" ]]; then
      if [[ $VERSION_CHANGED == "true" ]]; then
        echo "[INFO]  $dir: Upgraded to $TARGET_VERSION"
      elif [[ $LOCK_CHANGED == "true" ]]; then
        echo "[INFO]  $dir: Remained on $TARGET_VERSION, but inputs updated (Backports/Fixes)"
      fi

      echo "[DEBUG] Verifying build..."
      if (cd "$dir" && nix flake check &> /dev/null); then
        echo "[SUCCESS] $dir is healthy."
      else
        echo "[ERROR]   $dir: Check Failed (Build broken after update)"
      fi
    else
      echo "[INFO]  $dir: Already up to date (No changes in version or lockfile)."
    fi
  else
    echo "[ERROR]   $dir: 'nix flake update' failed."
  fi
done

echo "Done."
