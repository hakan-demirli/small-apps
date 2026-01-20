import calendar
import os
import sys
import time
from datetime import datetime, timedelta

from colorama import Fore, Style

from .shared import (
    BASE_COLORS,
    DAYS_BEFORE_TODAY,
    EVENTS_FILE_PATH,
    FADE_TARGET_RGB,
    STATIC_STYLES,
    STATUS_COLORS,
    STATUS_SYMBOLS,
    TRACE_LEVEL_NUM,
    clear_screen,
    get_faded_color,
    parse_events,
    read_events_from_file,
    rgb_to_ansi,
    setup_logging,
)


def print_dynamic_calendar(events_dict, logger):
    logger.debug("Clearing screen and starting calendar print.")
    clear_screen()

    try:
        available_lines = os.get_terminal_size().lines - 2
        logger.debug(f"Terminal height detected: {available_lines} available lines.")
    except OSError:
        available_lines = 40
        logger.warning(
            f"Could not get terminal size. Defaulting to {available_lines} lines."
        )

    today = datetime.now().date()
    current_date = today - timedelta(days=DAYS_BEFORE_TODAY)

    lines_printed = 0
    last_printed_month = None

    header_color = rgb_to_ansi(*BASE_COLORS["header"])
    past_event_color = rgb_to_ansi(*FADE_TARGET_RGB)
    unhandled_past_style = STATIC_STYLES["unhandled_past"]

    while lines_printed < available_lines:
        if current_date.month != last_printed_month:
            if lines_printed + 2 > available_lines:
                break
            month_name = calendar.month_name[current_date.month]
            print(f"{header_color}{month_name.upper()}{STATIC_STYLES['reset']}")
            lines_printed += 1
            last_printed_month = current_date.month

        events_for_day = events_dict.get(current_date, [])
        day_num = current_date.day
        distance = (current_date - today).days

        has_unhandled_past_event = False
        if distance < 0 and events_for_day:
            if any(s not in ["x", "X", ">"] for s, _ in events_for_day):
                has_unhandled_past_event = True

        if distance < 0:
            day_color = (
                unhandled_past_style if has_unhandled_past_event else past_event_color
            )
            countdown_color = past_event_color
            day_style = ""
        elif distance == 0:
            day_color, day_style = STATIC_STYLES["today"], STATIC_STYLES["bold"]
            countdown_color = rgb_to_ansi(*BASE_COLORS["countdown"])
        else:
            day_style = ""
            day_color = get_faded_color(BASE_COLORS["day"], distance)
            countdown_color = get_faded_color(BASE_COLORS["countdown"], distance)

        day_num_formatted = f"{day_num:>2}"
        day_gutter = (
            f"  {day_style}{day_color}{day_num_formatted}{STATIC_STYLES['reset']}"
        )

        if events_for_day:
            countdown_text = " -" if distance < 0 else f"{distance:>2}"
            countdown_gutter = (
                f"{countdown_color}{countdown_text}{STATIC_STYLES['reset']}"
            )

            status_char, event_name = events_for_day[0]
            status_symbol = STATUS_SYMBOLS.get(status_char, "○")
            status_base_color = STATUS_COLORS.get(status_char, BASE_COLORS["event"])

            if distance < 0:
                status_color = (
                    unhandled_past_style
                    if status_char not in ["x", "X", ">"]
                    else past_event_color
                )
            elif distance == 0:
                status_color = rgb_to_ansi(*status_base_color)
            else:
                status_color = get_faded_color(status_base_color, distance)

            first_event_text = f"{status_symbol} {event_name}"
            print(
                f"{day_gutter} {countdown_gutter} {status_color}{first_event_text}{STATIC_STYLES['reset']}"
            )
            lines_printed += 1

            if len(events_for_day) > 1:
                event_indentation = " " * 9
                for status_char, event_name in events_for_day[1:]:
                    if lines_printed >= available_lines:
                        break

                    status_symbol = STATUS_SYMBOLS.get(status_char, "○")
                    status_base_color = STATUS_COLORS.get(
                        status_char, BASE_COLORS["event"]
                    )

                    if distance < 0:
                        status_color = (
                            unhandled_past_style
                            if status_char not in ["x", "X", ">"]
                            else past_event_color
                        )
                    elif distance == 0:
                        status_color = rgb_to_ansi(*status_base_color)
                    else:
                        status_color = get_faded_color(status_base_color, distance)

                    print(
                        f"{event_indentation}{status_color}{status_symbol} {event_name}{STATIC_STYLES['reset']}"
                    )
                    lines_printed += 1
        else:
            print(f"{day_gutter}")
            lines_printed += 1

        current_date += timedelta(days=1)


def run(file_path=None):
    if file_path is None:
        file_path = EVENTS_FILE_PATH

    # if len(sys.argv) > 1 and sys.argv[1] == "--test":
    #    run_tests()
    #    sys.exit()

    DESIRED_LOG_LEVEL = TRACE_LEVEL_NUM

    logger = setup_logging(DESIRED_LOG_LEVEL, log_filename="riveroftime.log")
    logger.info("--- Script started ---")

    while True:
        try:
            event_lines = read_events_from_file(file_path, logger)
            parsed_event_data = parse_events(event_lines, logger)
            print_dynamic_calendar(parsed_event_data, logger)

            logger.debug("Main loop iteration complete. Sleeping for 5 seconds.")
            time.sleep(5)
        except KeyboardInterrupt:
            logger.info("--- Script stopped by user (KeyboardInterrupt) ---")
            print("\nExiting.")
            break
        except Exception as e:
            logger.error(
                f"An unexpected error occurred in the main loop: {e}", exc_info=True
            )
            print(f"{Fore.RED}An unexpected error occurred: {e}")
            time.sleep(10)

if __name__ == "__main__":
    run()
