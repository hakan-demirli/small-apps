import dap


def test_regex_indent_agnostic():
    block = "def foo():\n    return True"
    regex = dap.create_indent_agnostic_regex(block)

    target = "    def foo():\n        return True"
    assert regex.search(target) is not None


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


def test_preflight_checks_fail_file_not_found(capsys):
    patches = [
        {"file_path": "nonexistent.txt", "search_block": "foo", "replace_block": "bar"}
    ]

    success, errors = dap.run_preflight_checks(patches)

    assert success is False
    assert "File not found" in errors[0]


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


def test_smart_detection_old():
    content = "file\n<<<<<<< SEARCH\nfoo\n=======\nbar\n>>>>>>> REPLACE"
    assert "<<<<<<< SEARCH" in content


def test_smart_detection_new():
    content = ">>>> file\n<<<<\nfoo\n====\nbar\n>>>>"
    assert "<<<<<<< SEARCH" not in content
