import time
from datetime import datetime

from rich.live import Live
from rich.table import Table
from rich.text import Text

from .shared import (
    EVENTS_FILE_PATH,
    STATUS_SYMBOLS,
    TRACE_LEVEL_NUM,
    console,
    interpolate_color,
    parse_events,
    read_events_from_file,
    setup_logging,
)


def generate_deadlines_table(events_dict, logger, target_symbols):
    today = datetime.now().date()
    all_events = []

    for event_date, events in events_dict.items():
        days_remaining = (event_date - today).days
        for status_char, event_name, line_number in events:
            if status_char in target_symbols:
                all_events.append(
                    (days_remaining, line_number, status_char, event_name)
                )

    if not all_events:
        return Text("No upcoming deadlines found.")

    # Sort by Days first, then Line Number (file order)
    all_events.sort(key=lambda x: (x[0], x[1]))

    total_items = len(all_events)
    GRADIENT_START = (189, 147, 249)
    GRADIENT_END = (127, 210, 228)

    table = Table(show_header=False, box=None, padding=(0, 1))
    table.add_column("Days", justify="right")
    table.add_column("Symbol", justify="center")
    table.add_column("Event", justify="left")

    for i, (days, _, status, name) in enumerate(all_events):
        fraction = i / (total_items - 1) if total_items > 1 else 0.0

        color_hex = interpolate_color(GRADIENT_START, GRADIENT_END, fraction)

        symbol = STATUS_SYMBOLS.get(status, "○")

        days_str = f"{days}"

        table.add_row(
            Text(days_str, style=color_hex),
            Text(symbol, style=color_hex),
            Text(name, style=f"{color_hex}"),
        )
    return table


def run(file_path=None, symbols=None):
    if file_path is None:
        file_path = EVENTS_FILE_PATH

    target_symbols = symbols if symbols else ["<"]

    DESIRED_LOG_LEVEL = TRACE_LEVEL_NUM
    logger = setup_logging(DESIRED_LOG_LEVEL, log_filename="deadlines.log")

    with Live(console=console, auto_refresh=False, screen=True) as live:
        while True:
            try:
                event_lines = read_events_from_file(file_path, logger)
                parsed_event_data = parse_events(event_lines, logger)

                table_or_text = generate_deadlines_table(
                    parsed_event_data, logger, target_symbols
                )
                live.update(table_or_text, refresh=True)

                time.sleep(5)
            except KeyboardInterrupt:
                console.print("\nExiting.")
                break
            except Exception as e:
                logger.error(f"Unexpected error: {e}", exc_info=True)
                live.stop()
                console.print(f"[bold red]An unexpected error occurred: {e}[/]")
                time.sleep(10)
                live.start()


if __name__ == "__main__":
    run()
