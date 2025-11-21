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
