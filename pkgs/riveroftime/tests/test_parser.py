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
                    (" ", "Untitled Event", 0),
                    (" ", "Untitled Event", 1),
                    (" ", "Untitled Event", 2),
                    (" ", "Untitled Event", 3),
                    (" ", "* do smth4", 12),
                    (" ", "* do smth5", 13),
                    (" ", "* do smth6", 14),
                ]
            ),
            datetime(2025, 11, 15).date(): sorted(
                [
                    (" ", "task1", 22),
                    ("x", "task2", 23),
                    ("X", "task3", 24),
                    ("-", "task4", 25),
                    ("a", "task5", 26),
                    (" ", "task6", 27),
                    ("x", "task7", 28),
                ]
            ),
            datetime(2025, 11, 16).date(): sorted(
                [
                    ("!", "urgent1", 29),
                    (">", "delegated1", 30),
                    ("/", "inprogress1", 31),
                    ("?", "clarify1", 32),
                ]
            ),
            datetime(2026, 11, 1).date(): sorted([(" ", "Untitled Event", 4)]),
            datetime(2026, 11, 8).date(): sorted([(" ", "Untitled Event", 5)]),
            datetime(2029, 11, 8).date(): sorted(
                [
                    (" ", "Untitled Event", 6),
                    (" ", "Untitled Event", 7),
                    (" ", "* do smth1", 9),
                    (" ", "* do smth2", 10),
                    (" ", "* do smth3", 11),
                    (" ", "* do smth7", 15),
                    (" ", "* do smth8", 16),
                    (" ", "do smth9", 17),
                    (" ", "do smth10", 18),
                    (" ", "do smth11", 19),
                    (" ", "do smth12", 20),
                    (" ", "do smth13", 21),
                ]
            ),
            datetime(2038, 11, 8).date(): sorted([(" ", "Untitled Event", 8)]),
        }

        actual_result = parse_events(test_cases, self.logger)

        for key in actual_result:
            actual_result[key].sort()

        self.assertEqual(actual_result, expected_result)

    def test_parse_events_optional_year(self):
        current_year = datetime.now().year
        test_cases = [
            "* MICRO:",
            "  * [!] [02/02] bong",
            "[05/05] Cinco de Mayo",
            "10/10: Ten Ten",
        ]

        expected_result = {
            datetime(current_year, 2, 2).date(): sorted([("!", "MICRO: bong", 1)]),
            datetime(current_year, 5, 5).date(): sorted([(" ", "Cinco de Mayo", 2)]),
            datetime(current_year, 10, 10).date(): sorted([(" ", "Ten Ten", 3)]),
        }

        actual_result = parse_events(test_cases, self.logger)

        # Normalize for comparison
        for key in actual_result:
            actual_result[key].sort()

        self.assertEqual(actual_result, expected_result)


if __name__ == "__main__":
    unittest.main()
