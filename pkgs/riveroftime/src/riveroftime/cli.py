import argparse

from . import calendar_view, deadlines
from . import main as flow_main


def main():
    parser = argparse.ArgumentParser(
        description="RiverOfTime - Flow and Deadlines Manager"
    )
    parser.add_argument("--deadlines", action="store_true", help="Show deadlines view")
    parser.add_argument("--flow", action="store_true", help="Show flow (calendar) view")
    parser.add_argument(
        "--calendar", action="store_true", help="Show simple calendar view"
    )
    parser.add_argument("--file", type=str, help="Path to events file", default=None)
    parser.add_argument(
        "--symbols",
        type=str,
        help="Filter by status symbols (e.g. '<' or '!')",
        default=None,
    )

    args = parser.parse_args()

    if args.deadlines:
        symbol_list = list(args.symbols) if args.symbols else None
        deadlines.run(file_path=args.file, symbols=symbol_list)
    elif args.flow:
        flow_main.run(file_path=args.file)
    elif args.calendar:
        calendar_view.run()
    else:
        flow_main.run(file_path=args.file)


if __name__ == "__main__":
    main()
