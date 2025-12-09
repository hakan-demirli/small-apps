import dap


def test_find_occurrences_strategies():
    src = ["a", "  b", "c"]
    assert dap.find_occurrences(src, "  b") == ([1], 1)

    assert dap.find_occurrences(src, "\n  b\n") == ([1], 1)

    src_indented = ["    x", "    y", "    z"]
    block_flat = "x\ny\nz"
    assert dap.find_occurrences(src_indented, block_flat) == ([0], 3)

    src_mixed = ["  start", "    middle", "  end"]
    block_mixed = "start\nmiddle\nend"
    assert dap.find_occurrences(src_mixed, block_mixed) == ([0], 3)


def test_parse_old_format():
    patch_text = """
src/main.rs
<<<<<<< SEARCH
old
=======
new
>>>>>>> REPLACE
"""
    patches = list(dap.parse_diff_fenced(patch_text))
    assert len(patches) == 1
    assert patches[0]["file_path"] == "src/main.rs"
    assert "old" in patches[0]["search_block"]
    assert patches[0]["delete_file"] is False


def test_parse_new_format():
    patch_text = """
>>>> src/lib.rs
<<<<
true
====
false
>>>>
"""
    patches = list(dap.parse_source_dest_blocks(patch_text))
    assert len(patches) == 1
    assert patches[0]["file_path"] == "src/lib.rs"
    assert "true" in patches[0]["search_block"]
    assert patches[0]["delete_file"] is False


def test_parse_delete_format():
    patch_text = """
mathweb/flask/oldapp.py <<<<<<< DELETE
mathweb/init.py <<<<<<< DELETE
"""
    patches = list(dap.parse_raw_line_commands(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "mathweb/flask/oldapp.py"
    assert patches[0]["delete_file"] is True
    assert patches[1]["file_path"] == "mathweb/init.py"
    assert patches[1]["delete_file"] is True


def test_parse_move_format():
    patch_text = """
mathweb/init.py <<<<<<< MOVE >>>>>>> mathweb/webapi/__init__.py
mathweb/hi.cpp <<<<<<< MOVE >>>>>>> hi.cpp
"""
    patches = list(dap.parse_raw_line_commands(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "mathweb/init.py"
    assert patches[0]["move_destination"] == "mathweb/webapi/__init__.py"
    assert patches[1]["file_path"] == "mathweb/hi.cpp"
    assert patches[1]["move_destination"] == "hi.cpp"


def test_mixed_diff_fenced_and_commands():
    patch_text = """
tests/src/lib.rs
<<<<<<< SEARCH
old
=======
new
>>>>>>> REPLACE

tests/src/main.rs <<<<<<< DELETE

tests/tests/integration.rs
<<<<<<< SEARCH
foo
=======
bar
>>>>>>> REPLACE

run.sh <<<<<<< DELETE
"""
    patches = list(dap.parse_diff_fenced(patch_text))
    assert len(patches) == 4

    assert patches[0]["file_path"] == "tests/src/lib.rs"
    assert not patches[0]["delete_file"]

    assert patches[1]["file_path"] == "tests/src/main.rs"
    assert patches[1]["delete_file"]

    assert patches[2]["file_path"] == "tests/tests/integration.rs"
    assert not patches[2]["delete_file"]

    assert patches[3]["file_path"] == "run.sh"
    assert patches[3]["delete_file"]


def test_arrow_blocks_mixed_with_commands():
    patch_text = """
<<<< file1.rs
search1
====
replace1
>>>>
file2.rs <<<<<<< DELETE
"""
    patches = list(dap.parse_arrow_blocks(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "file1.rs"
    assert patches[1]["file_path"] == "file2.rs"
    assert patches[1]["delete_file"]


def test_source_dest_mixed_with_commands():
    patch_text = """
>>>> file1.rs
<<<<
search1
====
replace1
>>>>
file2.rs <<<<<<< DELETE
"""
    patches = list(dap.parse_source_dest_blocks(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "file1.rs"
    assert patches[1]["file_path"] == "file2.rs"
    assert patches[1]["delete_file"]


def test_parse_diff_fenced_with_code_fences():
    """
    Tests that code fences (```) immediately following the filename
    and preceding the SEARCH block are ignored.
    """
    patch_text = """
repx-tui/src/app.rs
```rust
<<<<<<< SEARCH
old_code
=======
new_code
>>>>>>> REPLACE
"""
    patches = list(dap.parse_diff_fenced(patch_text))
    assert len(patches) == 1
    assert patches[0]["file_path"] == "repx-tui/src/app.rs"
    assert "old_code" in patches[0]["search_block"]


def test_preflight_checks_fail_file_not_found(capsys):
    patches = [
        {"file_path": "nonexistent.txt", "search_block": "foo", "replace_block": "bar"}
    ]

    success, errors = dap.run_preflight_checks(patches)

    assert success is False
    assert "File not found" in errors[0]


def test_preflight_checks_fail_delete_not_found(capsys):
    patches = [
        {
            "file_path": "nonexistent.txt",
            "search_block": "",
            "replace_block": "",
            "delete_file": True,
        }
    ]
    success, errors = dap.run_preflight_checks(patches)
    assert success is False
    assert "File not found, cannot delete" in errors[0]


def test_preflight_checks_fail_move_source_missing(capsys):
    patches = [
        {
            "file_path": "nonexistent.txt",
            "move_destination": "dest.txt",
            "search_block": "",
            "replace_block": "",
        }
    ]
    success, errors = dap.run_preflight_checks(patches)
    assert success is False
    assert "Source file not found" in errors[0]


def test_preflight_checks_fail_move_dest_exists(tmp_path):
    src = tmp_path / "src.txt"
    src.touch()
    dst = tmp_path / "dst.txt"
    dst.touch()

    patches = [
        {
            "file_path": str(src),
            "move_destination": str(dst),
            "search_block": "",
            "replace_block": "",
        }
    ]
    success, errors = dap.run_preflight_checks(patches)
    assert success is False
    assert "Destination file" in errors[0]
    assert "already exists" in errors[0]


def test_apply_patch_success(tmp_path, capsys):
    f = tmp_path / "code.py"
    f.write_text("def hello():\n    print('Hi')", encoding="utf-8")

    patches = [
        {
            "file_path": str(f),
            "search_block": "def hello():\n    print('Hi')",
            "replace_block": "def hello():\n    print('Hello World')",
        }
    ]

    dap.apply_patch(patches[0])

    content = f.read_text(encoding="utf-8")
    assert "Hello World" in content
    assert "Hi" not in content


def test_apply_delete_success(tmp_path):
    f = tmp_path / "trash.py"
    f.write_text("content", encoding="utf-8")

    patches = [
        {
            "file_path": str(f),
            "search_block": "",
            "replace_block": "",
            "delete_file": True,
        }
    ]

    assert f.exists()
    dap.apply_patch(patches[0])
    assert not f.exists()


def test_apply_move_success(tmp_path):
    src = tmp_path / "old.py"
    src.write_text("import os", encoding="utf-8")
    dst = tmp_path / "subdir" / "new.py"

    patches = [
        {
            "file_path": str(src),
            "move_destination": str(dst),
            "search_block": "",
            "replace_block": "",
        }
    ]

    assert src.exists()
    assert not dst.exists()

    dap.apply_patch(patches[0])

    assert not src.exists()
    assert dst.exists()
    assert dst.read_text(encoding="utf-8") == "import os"


def test_file_creation_logic(tmp_path):
    new_file = tmp_path / "subdir" / "new.rs"

    patches = [
        {
            "file_path": str(new_file),
            "search_block": "",
            "replace_block": "fn main() {}",
        }
    ]

    success, _ = dap.run_preflight_checks(patches)
    assert success is True

    dap.apply_patch(patches[0])

    assert new_file.exists()
    assert new_file.read_text(encoding="utf-8") == "fn main() {}"


def test_overwrite_empty_file_logic(tmp_path):
    empty_file = tmp_path / "empty.txt"
    empty_file.touch()

    patches = [
        {
            "file_path": str(empty_file),
            "search_block": "",
            "replace_block": "filled",
        }
    ]

    success, _ = dap.run_preflight_checks(patches)
    assert success is True

    dap.apply_patch(patches[0])
    assert empty_file.read_text(encoding="utf-8") == "filled"


def test_overwrite_whitespace_only_file_logic(tmp_path):
    # Create a file with just newlines
    ws_file = tmp_path / "ws.txt"
    ws_file.write_text("\n   \n\n", encoding="utf-8")

    patches = [
        {
            "file_path": str(ws_file),
            "search_block": "",
            "replace_block": "filled",
        }
    ]

    success, _ = dap.run_preflight_checks(patches)
    assert success is True

    dap.apply_patch(patches[0])
    assert ws_file.read_text(encoding="utf-8") == "filled"


def test_fail_overwrite_non_empty_file(tmp_path):
    f = tmp_path / "full.txt"
    f.write_text("content", encoding="utf-8")

    patches = [
        {
            "file_path": str(f),
            "search_block": "",
            "replace_block": "new",
        }
    ]

    success, errors = dap.run_preflight_checks(patches)
    assert success is False
    assert "Search block is empty, but target file is not empty" in errors[0]
