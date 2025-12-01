#!/usr/bin/env python3
import argparse
import os
import re
import sys


def parse_diff_fenced(patch_content):
    """
    Parses the "SEARCH/REPLACE" format:
    file_path
    <<<<<<< SEARCH
    ...
    =======
    ...
    >>>>>>> REPLACE
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
                potential_path = previous_line.strip()

                if potential_path:
                    file_path = potential_path
                elif file_path:
                    pass
                else:
                    pass

                state = "in_search"
                search_lines = []
                replace_lines = []
            else:
                previous_line = line if stripped_line else ""
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
                    "delete_file": False,
                }
                state = "idle"
                previous_line = ""
            else:
                replace_lines.append(line)


def parse_arrow_blocks(patch_content):
    """
    Parses the format:
    <<<< file_path
    search_content
    ====
    replace_content
    >>>>
    """
    lines = patch_content.splitlines(True)

    state = "idle"
    file_path = None
    search_lines = []
    replace_lines = []

    for line in lines:
        stripped_line = line.strip()

        if state == "idle":
            if stripped_line.startswith("<<<< ") and not stripped_line.startswith(
                "<<<<<<<"
            ):
                file_path = stripped_line[5:].strip()
                state = "in_search"
                search_lines = []
                replace_lines = []

        elif state == "in_search":
            if stripped_line == "====":
                state = "in_replace"
            else:
                search_lines.append(line)

        elif state == "in_replace":
            if stripped_line == ">>>>":
                yield {
                    "file_path": file_path,
                    "search_block": "".join(search_lines),
                    "replace_block": "".join(replace_lines),
                    "delete_file": False,
                }
                state = "idle"
            else:
                replace_lines.append(line)


def parse_source_dest_blocks(patch_content):
    """
    Parses the format:
    >>>> file_path
    <<<<
    search_content
    ====
    replace_content
    >>>>
    """
    lines = patch_content.splitlines(True)

    state = "idle"
    file_path = None
    search_lines = []
    replace_lines = []

    for line in lines:
        stripped_line = line.strip()

        if state == "idle":
            if stripped_line.startswith(">>>> ") and not stripped_line.startswith(
                ">>>>>>>"
            ):
                file_path = stripped_line[5:].strip()

            elif stripped_line == "<<<<":
                if not file_path:
                    continue
                state = "in_search"
                search_lines = []
                replace_lines = []

        elif state == "in_search":
            if stripped_line == "====":
                state = "in_replace"
            else:
                search_lines.append(line)

        elif state == "in_replace":
            if stripped_line == ">>>>":
                yield {
                    "file_path": file_path,
                    "search_block": "".join(search_lines),
                    "replace_block": "".join(replace_lines),
                    "delete_file": False,
                }
                state = "idle"
            else:
                replace_lines.append(line)


def parse_delete_commands(patch_content):
    """
    Parses the format:
    path/to/file <<<<<<< DELETE
    """
    lines = patch_content.splitlines()
    for line in lines:
        line = line.strip()
        if line.endswith("<<<<<<< DELETE"):
            # Extract the file path (everything before the marker)
            parts = line.split("<<<<<<< DELETE")
            if len(parts) > 0:
                file_path = parts[0].strip()
                if file_path:
                    yield {
                        "file_path": file_path,
                        "search_block": "",
                        "replace_block": "",
                        "delete_file": True,
                    }


def find_occurrences(source_lines, search_block_str):
    """
    Finds where the lines in search_block_str occur in source_lines.

    Hierarchy of search:
    1. Strict Match: Exact content and indentation.
    2. Strict Match (Trimmed): Exact content/indentation after removing leading/trailing
       empty lines from the search block (handles copy-paste artifacts).
    3. Loose Match: Ignores indentation (whitespace) on both source and search lines.

    Returns:
        (matches, length_of_match_in_lines)
        matches: list of start indices (0-based)
        length_of_match_in_lines: number of lines in the source that matched
    """

    src_strict = [line.rstrip("\n\r") for line in source_lines]

    search_lines_strict = search_block_str.splitlines()
    matches = _find_sublist(src_strict, search_lines_strict)
    if matches:
        return matches, len(search_lines_strict)

    search_block_stripped = search_block_str.strip("\n\r")
    if search_block_stripped != search_block_str:
        search_lines_trimmed = search_block_stripped.splitlines()
        if search_lines_trimmed:
            matches = _find_sublist(src_strict, search_lines_trimmed)
            if matches:
                return matches, len(search_lines_trimmed)

    src_loose = [line.strip() for line in source_lines]
    if "search_lines_trimmed" not in locals():
        search_lines_trimmed = search_block_str.strip("\n\r").splitlines()

    search_lines_loose = [line.strip() for line in search_lines_trimmed]

    if not search_lines_loose:
        return [], 0

    matches = _find_sublist(src_loose, search_lines_loose)
    return matches, len(search_lines_trimmed)


def _find_sublist(full_list, sub_list):
    """Helper to find all occurrences of sub_list in full_list."""
    matches = []
    n = len(full_list)
    m = len(sub_list)
    if m == 0:
        return []

    for i in range(n - m + 1):
        if full_list[i : i + m] == sub_list:
            matches.append(i)
    return matches


def run_preflight_checks(patches):
    """
    Checks all patches before applying them.
    Ensures target files exist and search blocks are found uniquely.
    Returns True if all checks pass, False otherwise.
    """
    print("--- Running Preflight Checks ---")
    errors = []

    for i, patch in enumerate(patches):
        file_path = patch["file_path"]
        search_block = patch["search_block"]
        delete_file = patch.get("delete_file", False)
        check_prefix = f"  - Patch #{i + 1} for '{file_path}':"

        if delete_file:
            if os.path.exists(file_path):
                print(f"{check_prefix} OK (File scheduled for deletion)")
            else:
                errors.append(f"{check_prefix} FAILED (File not found, cannot delete)")
            continue

        if not search_block.strip():
            if not os.path.exists(file_path):
                print(f"{check_prefix} OK (New File Creation)")
                continue

            try:
                with open(file_path, encoding="utf-8") as f:
                    content = f.read()
                if not content.strip():
                    print(f"{check_prefix} OK (Overwrite Empty File)")
                    continue
                else:
                    errors.append(
                        f"{check_prefix} FAILED (Search block is empty, but target file is not empty)"
                    )
                    continue
            except Exception as e:
                errors.append(
                    f"{check_prefix} FAILED (Could not read existing file: {e})"
                )
                continue

        if not os.path.exists(file_path):
            errors.append(f"{check_prefix} FAILED (File not found)")
            continue

        try:
            with open(file_path, encoding="utf-8") as f:
                source_lines = f.readlines()
        except Exception as e:
            errors.append(f"{check_prefix} FAILED (Could not read file: {e})")
            continue

        matches, _ = find_occurrences(source_lines, search_block)
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
    delete_file = patch.get("delete_file", False)

    print(f"--- Applying patch to: {file_path}")

    if delete_file:
        if dry_run:
            print("    [DRY RUN] File would be deleted.")
            return True
        try:
            os.remove(file_path)
            print("    [SUCCESS] File deleted.")
            return True
        except Exception as e:
            print(f"    [ERROR] Could not delete file: {e}")
            return False

    if not search_block.strip():
        if dry_run:
            print("    [DRY RUN] File would be created/overwritten.")
            return True

        try:
            parent_dir = os.path.dirname(file_path)
            if parent_dir and not os.path.exists(parent_dir):
                os.makedirs(parent_dir, exist_ok=True)

            with open(file_path, "w", encoding="utf-8") as f:
                f.write(replace_block)
            print("    [SUCCESS] File created/overwritten.")
            return True
        except Exception as e:
            print(f"    [ERROR] Could not create/write file: {e}")
            return False

    try:
        with open(file_path, encoding="utf-8") as f:
            source_lines = f.readlines()
    except FileNotFoundError:
        print("    [ERROR] File not found during application.")
        return False

    matches, match_len = find_occurrences(source_lines, search_block)

    if len(matches) != 1:
        print(
            f"    [ERROR] Expected 1 replacement, but {len(matches)} occurred. Aborting."
        )
        return False

    start_idx = matches[0]
    end_idx = start_idx + match_len

    if dry_run:
        print("    [DRY RUN] Patch would be applied successfully.")
        return True

    replace_lines = replace_block.splitlines(True)
    source_lines[start_idx:end_idx] = replace_lines

    try:
        with open(file_path, "w", encoding="utf-8") as f:
            f.writelines(source_lines)
        print("    [SUCCESS] Patch applied.")
        return True
    except Exception as e:
        print(f"    [ERROR] Could not write changes to file: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Apply custom patches (Search/Replace, Git-Merge, or Delete format) using line-based matching."
    )

    parser.add_argument(
        "patch_file",
        nargs="?",
        default=None,
        help="Path to the patch file. If omitted, reads from stdin.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run preflight checks and show changes without modifying files.",
    )
    args = parser.parse_args()

    patch_content = ""

    if args.patch_file:
        if not os.path.exists(args.patch_file):
            print(
                f"Error: Patch file not found at '{args.patch_file}'", file=sys.stderr
            )
            sys.exit(1)
        with open(args.patch_file, encoding="utf-8") as f:
            patch_content = f.read()
    else:
        if sys.stdin.isatty():
            print(
                "Error: No patch file specified and no data piped from stdin.",
                file=sys.stderr,
            )
            parser.print_usage(file=sys.stderr)
            sys.exit(1)
        patch_content = sys.stdin.read()

    if "<<<<<<< SEARCH" in patch_content:
        print("Detected format: SEARCH/REPLACE block")
        patches = list(parse_diff_fenced(patch_content))
    elif " <<<<<<< DELETE" in patch_content:
        print("Detected format: DELETE commands")
        patches = list(parse_delete_commands(patch_content))
    elif re.search(r"^>>>> ", patch_content, re.MULTILINE):
        print(
            "Detected format: Source/Dest block (>>>> file <<<< search ==== replace >>>>)"
        )
        patches = list(parse_source_dest_blocks(patch_content))
    else:
        print("Detected format: Arrow block (<<<< file search ==== replace >>>>)")
        patches = list(parse_arrow_blocks(patch_content))
    if not patches:
        print("No valid patch blocks found in the input.")
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
