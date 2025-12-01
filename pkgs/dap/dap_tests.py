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
    patches = list(dap.parse_delete_commands(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "mathweb/flask/oldapp.py"
    assert patches[0]["delete_file"] is True
    assert patches[1]["file_path"] == "mathweb/init.py"
    assert patches[1]["delete_file"] is True


def test_continuous_diff_fenced():
    patch_text = """
lib/helper.nix
<<<<<<< SEARCH
block1
=======
replace1
>>>>>>> REPLACE
<<<<<<< SEARCH
block2
=======
replace2
>>>>>>> REPLACE
"""
    patches = list(dap.parse_diff_fenced(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "lib/helper.nix"
    assert "block1" in patches[0]["search_block"]
    assert patches[1]["file_path"] == "lib/helper.nix"
    assert "block2" in patches[1]["search_block"]


def test_continuous_source_dest():
    patch_text = """
>>>> lib/test.txt
<<<<
blockA
====
replaceA
>>>>
<<<<
blockB
====
replaceB
>>>>
"""
    patches = list(dap.parse_source_dest_blocks(patch_text))
    assert len(patches) == 2
    assert patches[0]["file_path"] == "lib/test.txt"
    assert patches[1]["file_path"] == "lib/test.txt"


def test_arrow_blocks_format():
    patch_text = """
<<<< src/main.rs
fn main() {
    old
}
====
fn main() {
    new
}
>>>>
"""
    patches = list(dap.parse_arrow_blocks(patch_text))
    assert len(patches) == 1
    assert patches[0]["file_path"] == "src/main.rs"
    assert "old" in patches[0]["search_block"]
    assert "new" in patches[0]["replace_block"]
    assert patches[0]["delete_file"] is False


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


def test_smart_detection_old():
    content = "file\n<<<<<<< SEARCH\nfoo\n=======\nbar\n>>>>>>> REPLACE"
    assert "<<<<<<< SEARCH" in content


def test_smart_detection_new():
    content = ">>>> file\n<<<<\nfoo\n====\nbar\n>>>>"
    assert "<<<<<<< SEARCH" not in content


def test_smart_detection_delete():
    content = "file.py <<<<<<< DELETE"
    assert " <<<<<<< DELETE" in content


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
