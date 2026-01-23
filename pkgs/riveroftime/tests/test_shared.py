import unittest
from unittest.mock import MagicMock, mock_open, patch

from riveroftime.shared import read_events_from_file


class TestShared(unittest.TestCase):
    def setUp(self):
        self.logger = MagicMock()

    @patch("riveroftime.shared.os.path.exists")
    @patch("riveroftime.shared.os.path.expanduser")
    @patch("builtins.open", new_callable=mock_open)
    def test_read_events_multiple_files(self, mock_file, mock_expanduser, mock_exists):
        # Setup
        mock_expanduser.side_effect = lambda x: x
        mock_exists.return_value = True

        # Configure mock_open to return different file handles for each call
        handle1 = mock_open(read_data="event1\n").return_value
        handle2 = mock_open(read_data="event2\n").return_value
        mock_file.side_effect = [handle1, handle2]

        # Act
        lines = read_events_from_file(["file1.md", "file2.md"], self.logger)

        # Assert
        self.assertEqual(lines, ["event1", "event2"])
        self.assertEqual(mock_file.call_count, 2)

    @patch("riveroftime.shared.os.path.exists")
    @patch("riveroftime.shared.os.path.expanduser")
    @patch("builtins.open", new_callable=mock_open, read_data="event1\n")
    def test_read_events_single_file_string(
        self, mock_file, mock_expanduser, mock_exists
    ):
        # Test backward compatibility with single string argument
        mock_expanduser.side_effect = lambda x: x
        mock_exists.return_value = True

        lines = read_events_from_file("file1.md", self.logger)

        self.assertEqual(lines, ["event1"])

    @patch("riveroftime.shared.os.path.exists")
    @patch("riveroftime.shared.os.path.expanduser")
    @patch("builtins.open", new_callable=mock_open)
    def test_read_events_comma_separated(self, mock_file, mock_expanduser, mock_exists):
        # Setup
        mock_expanduser.side_effect = lambda x: x
        mock_exists.return_value = True

        handle1 = mock_open(read_data="eventA\n").return_value
        handle2 = mock_open(read_data="eventB\n").return_value
        mock_file.side_effect = [handle1, handle2]

        # Act
        lines = read_events_from_file(["fileA.md, fileB.md"], self.logger)

        # Assert
        self.assertEqual(lines, ["eventA", "eventB"])
        self.assertEqual(mock_file.call_count, 2)


if __name__ == "__main__":
    unittest.main()
