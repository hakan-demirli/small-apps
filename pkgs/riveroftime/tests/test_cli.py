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


if __name__ == "__main__":
    unittest.main()
