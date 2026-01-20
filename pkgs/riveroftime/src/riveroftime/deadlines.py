import time
from datetime import datetime

from colorama import Fore

from .shared import (
    BASE_COLORS,
    EVENTS_FILE_PATH,
    FADE_TARGET_RGB,
    STATIC_STYLES,
    STATUS_COLORS,
    STATUS_SYMBOLS,
    TRACE_LEVEL_NUM,
    clear_screen,
    get_faded_color,
    interpolate_color,
    parse_events,
    read_events_from_file,
    rgb_to_ansi,
    setup_logging,
)


def print_deadlines(events_dict, logger):
    logger.debug("Clearing screen and starting deadlines print.")
    clear_screen()

    today = datetime.now().date()
    all_events = []

    for event_date, events in events_dict.items():
        days_remaining = (event_date - today).days
        for status_char, event_name in events:
            if status_char == "<":
                all_events.append((days_remaining, status_char, event_name))

    if not all_events:
        print(f"{STATIC_STYLES['reset']}No upcoming deadlines found.")
        return

    all_events.sort(key=lambda x: x[0])

    total_items = len(all_events)
    GRADIENT_START = (189, 147, 249)
    GRADIENT_END = (127, 210, 228)

    for i, (days, status, name) in enumerate(all_events):
        if total_items > 1:
            fraction = i / (total_items - 1)
        else:
            fraction = 0.0

        color = interpolate_color(GRADIENT_START, GRADIENT_END, fraction)
        count_color = color
        
        symbol = STATUS_SYMBOLS.get(status, "○")
        reset = STATIC_STYLES["reset"]

        print(f"{count_color}{days:>4}{reset} {color}{symbol} {name}{reset}")


def run(file_path=None):
    if file_path is None:
        file_path = EVENTS_FILE_PATH

    DESIRED_LOG_LEVEL = TRACE_LEVEL_NUM
    logger = setup_logging(DESIRED_LOG_LEVEL, log_filename="deadlines.log")

    while True:
        try:
            event_lines = read_events_from_file(file_path, logger)
            parsed_event_data = parse_events(event_lines, logger)
            print_deadlines(parsed_event_data, logger)

            time.sleep(5)
        except KeyboardInterrupt:
            print("\nExiting.")
            break
        except Exception as e:
            logger.error(f"Unexpected error: {e}", exc_info=True)
            print(f"{Fore.RED}An unexpected error occurred: {e}")
            time.sleep(10)


if __name__ == "__main__":
    run()
