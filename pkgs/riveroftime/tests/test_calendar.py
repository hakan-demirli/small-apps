import datetime
import unittest
from unittest.mock import patch

from riveroftime import calendar_view


class TestCalendarView(unittest.TestCase):
    @patch("riveroftime.calendar_view.datetime")
    @patch("builtins.print")
    def test_run_prints_calendar(self, mock_print, mock_datetime_module):
        class FakeDate(datetime.date):
            @classmethod
            def today(cls):
                return cls(2025, 1, 1)

        mock_datetime_module.date = FakeDate

        calendar_view.run()

        self.assertTrue(mock_print.call_count > 0)

        printed_content = [call.args[0] for call in mock_print.call_args_list]
        combined_output = "\n".join(printed_content)

        self.assertIn("January 2025", combined_output)
        self.assertIn("February 2025", combined_output)
        self.assertIn("March 2025", combined_output)


if __name__ == "__main__":
    unittest.main()
