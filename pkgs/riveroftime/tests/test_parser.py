import logging
import unittest
from datetime import datetime

from riveroftime.shared import parse_events


class TestParser(unittest.TestCase):
    def setUp(self):
        self.maxDiff = None
        self.logger = logging.getLogger("test_logger")
        self.logger.setLevel(logging.CRITICAL + 1)
        self.logger.addHandler(logging.NullHandler())

    def test_parse_events(self):
        test_cases = [
            "[01/11/2025]",
            "[01-11-2025]",
            "[01.11.2025]",
            "[1.11.2025]",
            "[1.11.26]",
            "[8/11/26]",
            "[8.11/29]",
            "[8/11.29]",
            "[8.11.38]",
            "* [8/11.29] do smth1",
            "* do smth2 [8/11.29]",
            "* do smth3[8/11.29]",
            "* do smth4[01-11-2025]",
            "    * do smth5[01-11-2025]",
            "        * do smth6[01-11-2025]",
            "        * [8/11.29] do smth7",
            "    * [8/11.29] do smth8",
            "[8/11.29]: do smth9",
            "do smth10 [8/11.29]",
            "do smth11:[8/11.29]",
            "do smth12: [8/11.29]",
            "do smth13: [8/11.29]",
            "* [ ] task1 [15/11/2025]",
            "* [x] task2 [15/11/2025]",
            "* [X] task3 [15/11/2025]",
            "* [-] task4 [15/11/2025]",
            "* [a] task5 [15/11/2025]",
            "[15/11/2025] * [ ] task6",
            "[15/11/2025]: * [x] task7",
            "* [!] urgent1 [16/11/2025]",
            "* [>] delegated1 [16/11/2025]",
            "* [/] inprogress1 [16/11/2025]",
            "* [?] clarify1 [16/11/2025]",
        ]

        expected_result = {
            datetime(2025, 11, 1).date(): sorted(
                [
                    (" ", "Untitled Event"),
                    (" ", "Untitled Event"),
                    (" ", "Untitled Event"),
                    (" ", "Untitled Event"),
                    (" ", "* do smth4"),
                    (" ", "* do smth5"),
                    (" ", "* do smth6"),
                ]
            ),
            datetime(2025, 11, 15).date(): sorted(
                [
                    (" ", "task1"),
                    ("x", "task2"),
                    ("X", "task3"),
                    ("-", "task4"),
                    ("a", "task5"),
                    (" ", "task6"),
                    ("x", "task7"),
                ]
            ),
            datetime(2025, 11, 16).date(): sorted(
                [
                    ("!", "urgent1"),
                    (">", "delegated1"),
                    ("/", "inprogress1"),
                    ("?", "clarify1"),
                ]
            ),
            datetime(2026, 11, 1).date(): sorted([(" ", "Untitled Event")]),
            datetime(2026, 11, 8).date(): sorted([(" ", "Untitled Event")]),
            datetime(2029, 11, 8).date(): sorted(
                [
                    (" ", "Untitled Event"),
                    (" ", "Untitled Event"),
                    (" ", "* do smth1"),
                    (" ", "* do smth2"),
                    (" ", "* do smth3"),
                    (" ", "* do smth7"),
                    (" ", "* do smth8"),
                    (" ", "do smth9"),
                    (" ", "do smth10"),
                    (" ", "do smth11"),
                    (" ", "do smth12"),
                    (" ", "do smth13"),
                ]
            ),
            datetime(2038, 11, 8).date(): sorted([(" ", "Untitled Event")]),
        }

        actual_result = parse_events(test_cases, self.logger)

        for key in actual_result:
            actual_result[key].sort()

        self.assertEqual(actual_result, expected_result)


if __name__ == "__main__":
    unittest.main()
