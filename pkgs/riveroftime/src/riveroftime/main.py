import calendar
import time
from datetime import datetime, timedelta

from rich.console import Group
from rich.live import Live
from rich.text import Text

from .shared import (
    BASE_COLORS,
    DAYS_BEFORE_TODAY,
    EVENTS_FILE_PATH,
    FADE_TARGET_RGB,
    STATUS_COLORS,
    STATUS_SYMBOLS,
    TRACE_LEVEL_NUM,
    console,
    get_faded_color,
    parse_events,
    read_events_from_file,
    rgb_to_hex,
    setup_logging,
)

STATIC_STYLES = {
    "today": "white",
    "bold": "bold",
    "unhandled_past": "#ff5050",
}


def generate_calendar_view(events_dict, logger):
    renderables = []

    try:
        available_lines = console.size.height - 2
        logger.debug(f"Terminal height detected: {available_lines} available lines.")
    except Exception:
        available_lines = 40
        logger.warning(
            f"Could not get terminal size. Defaulting to {available_lines} lines."
        )

    today = datetime.now().date()
    current_date = today - timedelta(days=DAYS_BEFORE_TODAY)

    lines_printed = 0
    last_printed_month = None

    header_color = rgb_to_hex(*BASE_COLORS["header"])
    past_event_color = rgb_to_hex(*FADE_TARGET_RGB)
    unhandled_past_style = STATIC_STYLES["unhandled_past"]

    while lines_printed < available_lines:
        if current_date.month != last_printed_month:
            if lines_printed + 2 > available_lines:
                break
            month_name = calendar.month_name[current_date.month]
            renderables.append(Text(f"{month_name.upper()}", style=header_color))
            lines_printed += 1
            last_printed_month = current_date.month

        events_for_day = events_dict.get(current_date, [])
        day_num = current_date.day
        distance = (current_date - today).days

        has_unhandled_past_event = False
        if (
            distance < 0
            and events_for_day
            and any(s not in ["x", "X", ">"] for s, _, _ in events_for_day)
        ):
            has_unhandled_past_event = True

        day_style = ""
        if distance < 0:
            day_color = (
                unhandled_past_style if has_unhandled_past_event else past_event_color
            )
            countdown_color = past_event_color
        elif distance == 0:
            day_color = STATIC_STYLES["today"]
            day_style = STATIC_STYLES["bold"]
            countdown_color = rgb_to_hex(*BASE_COLORS["countdown"])
        else:
            day_color = get_faded_color(BASE_COLORS["day"], distance)
            countdown_color = get_faded_color(BASE_COLORS["countdown"], distance)

        day_num_formatted = f"{day_num:>2}"

        full_day_style = day_color
        if day_style:
            full_day_style = f"{day_style} {day_color}"

        day_gutter = Text(day_num_formatted, style=full_day_style)

        final_line = Text("  ") + day_gutter + Text(" ")

        if events_for_day:
            countdown_text = " -" if distance < 0 else f"{distance:>2}"
            countdown_gutter = Text(countdown_text, style=countdown_color)

            final_line.append(countdown_gutter)
            final_line.append(" ")

            status_char, event_name, _ = events_for_day[0]
            status_symbol = STATUS_SYMBOLS.get(status_char, "○")
            status_base_color = STATUS_COLORS.get(status_char, BASE_COLORS["event"])

            if distance < 0:
                status_color = (
                    unhandled_past_style
                    if status_char not in ["x", "X", ">"]
                    else past_event_color
                )
            elif distance == 0:
                status_color = rgb_to_hex(*status_base_color)
            else:
                status_color = get_faded_color(status_base_color, distance)

            first_event_text = Text(f"{status_symbol} {event_name}", style=status_color)
            final_line.append(first_event_text)

            renderables.append(final_line)
            lines_printed += 1

            if len(events_for_day) > 1:
                event_indentation = " " * 9
                for status_char, event_name, _ in events_for_day[1:]:
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
                        status_color = rgb_to_hex(*status_base_color)
                    else:
                        status_color = get_faded_color(status_base_color, distance)

                    event_line = Text(event_indentation)
                    event_line.append(
                        f"{status_symbol} {event_name}", style=status_color
                    )
                    renderables.append(event_line)
                    lines_printed += 1
        else:
            renderables.append(final_line)
            lines_printed += 1

        current_date += timedelta(days=1)

    return Group(*renderables)


def run(file_path=None):
    if file_path is None:
        file_path = EVENTS_FILE_PATH

    DESIRED_LOG_LEVEL = TRACE_LEVEL_NUM

    logger = setup_logging(DESIRED_LOG_LEVEL, log_filename="riveroftime.log")
    logger.info("--- Script started ---")

    with Live(console=console, auto_refresh=False, screen=True) as live:
        while True:
            try:
                event_lines = read_events_from_file(file_path, logger)
                parsed_event_data = parse_events(event_lines, logger)

                view_group = generate_calendar_view(parsed_event_data, logger)

                live.update(view_group, refresh=True)

                logger.debug("Main loop iteration complete. Sleeping for 5 seconds.")
                time.sleep(5)
            except KeyboardInterrupt:
                logger.info("--- Script stopped by user (KeyboardInterrupt) ---")
                console.print("\nExiting.")
                break
            except Exception as e:
                logger.error(
                    f"An unexpected error occurred in the main loop: {e}", exc_info=True
                )
                live.stop()
                console.print(f"[bold red]An unexpected error occurred: {e}[/]")

                time.sleep(10)
                live.start()


if __name__ == "__main__":
    run()
