import argparse

from . import deadlines
from . import main as flow_main


def main():
    parser = argparse.ArgumentParser(
        description="RiverOfTime - Flow and Deadlines Manager"
    )
    parser.add_argument("--deadlines", action="store_true", help="Show deadlines view")
    parser.add_argument("--flow", action="store_true", help="Show flow (calendar) view")
    parser.add_argument("--file", type=str, help="Path to events file", default=None)

    args = parser.parse_args()

    if args.deadlines:
        deadlines.run(file_path=args.file)
    elif args.flow:
        flow_main.run(file_path=args.file)
    else:
        flow_main.run(file_path=args.file)


if __name__ == "__main__":
    main()
