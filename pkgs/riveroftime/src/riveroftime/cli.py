import argparse
import sys
from . import deadlines, main as flow_main

def main():
    parser = argparse.ArgumentParser(description="RiverOfTime - Flow and Deadlines Manager")
    parser.add_argument("--deadlines", action="store_true", help="Show deadlines view")
    parser.add_argument("--flow", action="store_true", help="Show flow (calendar) view")
    parser.add_argument("--file", type=str, help="Path to events file", default=None)

    args = parser.parse_args()

    # Determine mode
    # If deadlines is specified, usage is deadlines.
    # If flow is specified, usage is flow.
    # If neither, default to flow (as per typical usage where one might just type command).
    # If both, user didn't specify behavior, but let's prioritize deadlines if user asked for both? 
    # Or maybe run both? The user prompt implies mutually exclusive or mode switching.
    # The user said "riveroftime --deadlines" and "riveroftime --flow".
    
    if args.deadlines:
        deadlines.run(file_path=args.file)
    elif args.flow:
        flow_main.run(file_path=args.file)
    else:
        # Default behavior
        flow_main.run(file_path=args.file)

if __name__ == "__main__":
    main()
