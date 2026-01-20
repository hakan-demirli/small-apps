import logging
import os
import re
from datetime import datetime

from colorama import Fore, Style, init

init(autoreset=True)

EVENTS_FILE_PATH = os.path.expanduser("~/Desktop/state/scratchpads/scratchpad4.md")
MAX_FADE_DAYS = 30
FADE_TARGET_RGB = (85, 85, 85)
DAYS_BEFORE_TODAY = 5

TRACE_LEVEL_NUM = 5
logging.addLevelName(TRACE_LEVEL_NUM, "TRACE")


class MyLogger(logging.Logger):
    def trace(self, message, *args, **kws):
        if self.isEnabledFor(TRACE_LEVEL_NUM):
            self._log(TRACE_LEVEL_NUM, message, args, **kws)


logging.setLoggerClass(MyLogger)


def setup_logging(log_level, log_filename="riveroftime.log"):
    log_dir = os.path.expanduser("~/.cache/my-state")
    log_file_path = os.path.join(log_dir, log_filename)

    logger = logging.getLogger(__name__)
    logger.setLevel(log_level)

    if logger.hasHandlers():
        logger.handlers.clear()

    try:
        os.makedirs(log_dir, exist_ok=True)
        handler = logging.FileHandler(log_file_path)
        formatter = logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
        handler.setFormatter(formatter)
        logger.addHandler(handler)
    except Exception as e:
        print(
            f"{Fore.RED}Failed to configure logging to '{log_file_path}': {e}{Style.RESET_ALL}"
        )
        logger.addHandler(logging.NullHandler())

    return logger


def rgb_to_ansi(r, g, b):
    return f"\033[38;2;{int(r)};{int(g)};{int(b)}m"


def get_faded_color(base_rgb, distance_from_today):
    if distance_from_today <= 0:
        return rgb_to_ansi(*base_rgb)

    fade_factor = min(1.0, abs(distance_from_today) / MAX_FADE_DAYS)
    r = base_rgb[0] + (FADE_TARGET_RGB[0] - base_rgb[0]) * fade_factor
    g = base_rgb[1] + (FADE_TARGET_RGB[1] - base_rgb[1]) * fade_factor
    return rgb_to_ansi(r, g, b)


def interpolate_color(start_rgb, end_rgb, fraction):
    fraction = max(0.0, min(1.0, fraction))
    r = start_rgb[0] + (end_rgb[0] - start_rgb[0]) * fraction
    g = start_rgb[1] + (end_rgb[1] - start_rgb[1]) * fraction
    b = start_rgb[2] + (end_rgb[2] - start_rgb[2]) * fraction
    return rgb_to_ansi(r, g, b)


BASE_COLORS = {
    "day": (128, 128, 128),
    "event": (127, 210, 228),
    "countdown": (189, 147, 249),
    "header": (85, 85, 85),
}

STATUS_SYMBOLS = {
    " ": "○",
    "x": "✓",
    "X": "✓",
    ">": "\u203a",
    "!": "!",
    "-": "-",
    "/": "…",
    "?": "?",
    "o": "⊘",
    "I": "\u2139",
    "L": "⚲",
    "*": "*",
    "<": "\u2039",
}

STATUS_COLORS = {
    " ": (127, 210, 228),
    "x": (85, 85, 85),
    "X": (85, 85, 85),
    ">": (150, 120, 180),
    "!": (255, 140, 80),
    "-": (85, 85, 85),
    "/": (180, 200, 100),
    "?": (220, 180, 100),
    "o": (220, 87, 125),
    "I": (100, 180, 220),
    "L": (100, 220, 120),
    "*": (150, 150, 150),
    "<": (120, 150, 220),
}


STATIC_STYLES = {
    "today": Fore.WHITE,
    "bold": Style.BRIGHT,
    "reset": Style.RESET_ALL,
    "unhandled_past": rgb_to_ansi(255, 80, 80),
}


def clear_screen():
    os.system("cls" if os.name == "nt" else "clear")


def read_events_from_file(file_path, logger):
    logger.info(f"Attempting to read events file: {file_path}")
    if not os.path.exists(file_path):
        message = f"Warning: Events file not found at '{file_path}'."
        print(f"{Fore.YELLOW}{message}{Style.RESET_ALL}")
        logger.warning(message)
        return []

    try:
        with open(file_path) as f:
            lines = f.readlines()
        lines = [line.rstrip("\n") for line in lines]
        logger.info(f"Successfully read {len(lines)} lines from file.")
        return lines
    except Exception as e:
        message = f"Error reading events file '{file_path}': {e}"
        print(f"{Fore.RED}{message}{Style.RESET_ALL}")
        logger.error(message)
        return []


def parse_events(event_list, logger):
    logger.info(f"Starting to parse {len(event_list)} lines with context awareness.")
    parsed = {}

    bracket_pattern = re.compile(r"\[(\d{1,2})[/\.-](\d{1,2})[/\.-](\d{2,4})\]")
    prefix_pattern = re.compile(r"^(\d{1,2})[/\.-](\d{1,2})[/\.-](\d{2,4}):")
    status_pattern = re.compile(r"^\*?\s*\[(.)\]\s*")

    context_stack = {}

    for line in event_list:
        if not line.strip():
            continue

        logger.trace(f"Processing line: '{line}'")
        indent_level = len(line) - len(line.lstrip(" "))

        stale_indents = [indent for indent in context_stack if indent >= indent_level]
        for indent in stale_indents:
            del context_stack[indent]
            logger.trace(f"Popped context at indent {indent}")

        cleaned_line = line.strip()

        is_header = (
            cleaned_line.endswith(":")
            and not bracket_pattern.search(line)
            and not prefix_pattern.search(line)
        )

        if is_header:
            tag = cleaned_line.removesuffix(":").lstrip("* ").strip()
            context_stack[indent_level] = tag
            logger.trace(f"Pushed new context at indent {indent_level}: '{tag}'")
            continue

        match = bracket_pattern.search(line)
        if not match:
            match = prefix_pattern.search(line)

        if match:
            parent_tag = None
            if context_stack:
                parent_indent = max(
                    [i for i in context_stack if i < indent_level], default=-1
                )
                if parent_indent != -1:
                    parent_tag = context_stack[parent_indent]

            status_char = " "
            if match.re == bracket_pattern:
                temp_name = line.replace(match.group(0), "").strip()
                if temp_name.startswith(":"):
                    temp_name = temp_name.lstrip(":").strip()

                status_match = status_pattern.match(temp_name)
                if status_match:
                    status_char = status_match.group(1)
                    temp_name = temp_name[status_match.end() :]
                event_name_str = " ".join(temp_name.strip(":").strip().split())
            else:
                event_name_str = line[match.end() :].strip()
                status_match = status_pattern.match(event_name_str)
                if status_match:
                    status_char = status_match.group(1)
                    event_name_str = event_name_str[status_match.end() :].strip()

            if parent_tag:
                event_name_str = f"{parent_tag}: {event_name_str}"
                logger.trace(f"Applied tag '{parent_tag}' to event.")

            day_str, month_str, year_str = match.groups()
            if len(year_str) == 2:
                year_str = f"20{year_str}"

            try:
                date_str_normalized = f"{day_str}/{month_str}/{year_str}"
                event_date = datetime.strptime(date_str_normalized, "%d/%m/%Y").date()

                if not event_name_str:
                    event_name_str = "Untitled Event"

                logger.debug(
                    f"Parsed date '{event_date}' with event '{event_name_str}' and status '{status_char}' from line."
                )
                parsed.setdefault(event_date, []).append((status_char, event_name_str))
            except ValueError:
                logger.warning(
                    f"Found a valid-looking date pattern but failed to parse it: '{match.group(0)}'. Skipping line: '{line}'"
                )
                continue
        else:
            logger.trace(f"Line is not a header or a dated event, skipping: '{line}'")

    logger.info(
        f"Parsing complete. Found {sum(len(v) for v in parsed.values())} events across {len(parsed)} dates."
    )
    return parsed
