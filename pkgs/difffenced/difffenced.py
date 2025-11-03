#!/usr/bin/env python3
import argparse
import os
import re
import sys


def create_indent_agnostic_regex(block_string):
    """
    Converts a multi-line string into a regex pattern that ignores
    leading whitespace on each line.
    """
    block_string = block_string.strip("\n\r")

    lines = block_string.splitlines()
    escaped_lines = [re.escape(line.strip()) for line in lines]
    regex_pattern = r"\s*" + r"\s*\n\s*".join(escaped_lines) + r"\s*"

    return re.compile(regex_pattern)


def parse_diff_fenced(patch_content):
    """
    Parses a string for diff-fenced blocks without relying on code fences (```).
    It uses the <<< SEARCH marker as the anchor and the preceding line as the file path.
    """
    lines = patch_content.splitlines(True)

    state = "idle"
    previous_line = ""
    file_path = None
    search_lines = []
    replace_lines = []

    for line in lines:
        stripped_line = line.strip()

        if state == "idle":
            if stripped_line == "<<<<<<< SEARCH":
                file_path = previous_line.strip()
                if not file_path:
                    print(
                        "Warning: Found a patch block start marker without a preceding file path. Skipping."
                    )
                    continue

                state = "in_search"
                search_lines = []
                replace_lines = []
            else:
                if stripped_line:
                    previous_line = line

        elif state == "in_search":
            if stripped_line == "=======":
                state = "in_replace"
            else:
                search_lines.append(line)

        elif state == "in_replace":
            if stripped_line == ">>>>>>> REPLACE":
                yield {
                    "file_path": file_path.strip(),
                    "search_block": "".join(search_lines),
                    "replace_block": "".join(replace_lines),
                }
                state = "idle"
                previous_line = ""
            else:
                replace_lines.append(line)


def run_preflight_checks(patches):
    """
    Checks all patches before applying them.
    Ensures target files exist and search blocks are found uniquely.
    Returns True if all checks pass, False otherwise, along with a list of errors.
    """
    print("--- Running Preflight Checks ---")
    errors = []

    for i, patch in enumerate(patches):
        file_path = patch["file_path"]
        search_block = patch["search_block"]
        check_prefix = f"  - Patch #{i + 1} for '{file_path}':"

        if not os.path.exists(file_path):
            errors.append(f"{check_prefix} FAILED (File not found)")
            continue

        try:
            with open(file_path, encoding="utf-8") as f:
                content = f.read()
        except Exception as e:
            errors.append(f"{check_prefix} FAILED (Could not read file: {e})")
            continue

        if not search_block.strip():
            errors.append(f"{check_prefix} FAILED (Search block is empty)")
            continue

        search_pattern = create_indent_agnostic_regex(search_block)
        matches = re.findall(search_pattern, content)
        count = len(matches)

        if count == 0:
            errors.append(f"{check_prefix} FAILED (Search block not found)")
        elif count > 1:
            errors.append(
                f"{check_prefix} FAILED (Search block is ambiguous, found {count} times)"
            )
        else:
            print(f"{check_prefix} OK")

    if errors:
        return False, errors

    return True, []


def apply_patch(patch, dry_run=False):
    """
    Applies a single parsed patch to the target file.
    Assumes preflight checks have already passed.
    """
    file_path = patch["file_path"]
    search_block = patch["search_block"]
    replace_block = patch["replace_block"]

    print(f"--- Applying patch to: {file_path}")

    with open(file_path, encoding="utf-8") as f:
        original_content = f.read()

    search_pattern = create_indent_agnostic_regex(search_block)
    new_content, num_replacements = search_pattern.subn(
        replace_block, original_content, count=1
    )

    if num_replacements != 1:
        print(
            f"    [ERROR] Expected 1 replacement, but {num_replacements} occurred. Aborting this patch."
        )
        return False

    if dry_run:
        print("    [DRY RUN] Patch would be applied successfully.")
        print("    --- CHANGES PREVIEW ---")
        print(f"    - {search_block.strip()}")
        print(f"    + {replace_block.strip()}")
        print("    -----------------------")
        return True

    try:
        with open(file_path, "w", encoding="utf-8") as f:
            f.write(new_content)
        print("    [SUCCESS] Patch applied.")
        return True
    except Exception as e:
        print(f"    [ERROR] Could not write changes to file: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Apply custom patches in diff-fenced format with preflight checks."
    )
    parser.add_argument("patch_file", help="Path to the patch file.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run preflight checks and show changes without modifying files.",
    )
    args = parser.parse_args()

    if not os.path.exists(args.patch_file):
        print(f"Error: Patch file not found at '{args.patch_file}'")
        sys.exit(1)

    with open(args.patch_file, encoding="utf-8") as f:
        patch_content = f.read()

    patches = list(parse_diff_fenced(patch_content))

    if not patches:
        print("No valid diff-fenced blocks found in the patch file.")
        sys.exit(0)

    preflight_ok, errors = run_preflight_checks(patches)

    if not preflight_ok:
        print("\n--- Preflight Checks Failed ---")
        for error in errors:
            print(error)
        print("\nAborting. No files were modified.")
        sys.exit(1)

    print("\n--- Preflight Checks Passed. Proceeding with patching. ---")

    success_count = 0
    fail_count = 0
    for patch in patches:
        if apply_patch(patch, dry_run=args.dry_run):
            success_count += 1
        else:
            fail_count += 1

    print("\n--- Summary ---")
    print(f"Total patches:        {len(patches)}")
    print(f"Successfully applied: {success_count}")
    print(f"Failed to apply:      {fail_count}")

    if fail_count > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
