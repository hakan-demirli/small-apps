import sys
import unittest
from unittest.mock import MagicMock, patch

sys.modules["colorama"] = MagicMock()


class TestCLI(unittest.TestCase):
    @patch("riveroftime.calendar_view.run")
    @patch("sys.argv", ["riveroftime", "--calendar"])
    def test_calendar_flag_calls_view(self, mock_run):
        from riveroftime import cli

        cli.main()
        mock_run.assert_called_once()

    @patch("riveroftime.deadlines.run")
    @patch("sys.argv", ["riveroftime", "--deadlines"])
    def test_deadlines_flag_calls_view(self, mock_run):
        from riveroftime import cli

        cli.main()
        mock_run.assert_called_once()

    @patch("riveroftime.main.run")
    @patch("sys.argv", ["riveroftime", "--flow"])
    def test_flow_flag_calls_view(self, mock_run):
        from riveroftime import cli

        cli.main()
        mock_run.assert_called_once()

    @patch("riveroftime.deadlines.run")
    @patch(
        "sys.argv", ["riveroftime", "--deadlines", "--files", "file1.md", "file2.md"]
    )
    def test_deadlines_files_arg(self, mock_run):
        from riveroftime import cli

        cli.main()
        mock_run.assert_called_once()
        call_args = mock_run.call_args
        self.assertEqual(call_args.kwargs["file_path"], ["file1.md", "file2.md"])

    @patch("riveroftime.main.run")
    @patch("sys.argv", ["riveroftime", "--flow", "--file", "fileA.md", "fileB.md"])
    def test_flow_files_arg(self, mock_run):
        from riveroftime import cli

        cli.main()
        mock_run.assert_called_once()
        call_args = mock_run.call_args
        self.assertEqual(call_args.kwargs["file_path"], ["fileA.md", "fileB.md"])


if __name__ == "__main__":
    unittest.main()
