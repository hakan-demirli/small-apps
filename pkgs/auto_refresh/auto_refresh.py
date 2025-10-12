#!/usr/bin/env python3

import logging
import os
import subprocess
import time

import gi

gi.require_version("GUdev", "1.0")

from gi.repository import GLib, GUdev  # noqa: E42

LOG_FILE_PATH = os.path.expanduser("~/.cache/auto_refresh/log.txt")
CONFIG_FILE_PATH = os.path.expanduser("~/.config/hypr/monitors.conf")
AC_STATUS_FILE_PATH = "/sys/class/power_supply/AC/online"
if not os.path.exists(AC_STATUS_FILE_PATH):
    AC_STATUS_FILE_PATH = "/sys/class/power_supply/ACAD/online"

TARGET_MONITOR = "desc:Chimei Innolux Corporation 0x1521"
MAX_REFRESH_RATE = 144
MIN_REFRESH_RATE = 60

os.makedirs(os.path.dirname(LOG_FILE_PATH), exist_ok=True)
logging.basicConfig(
    filename=LOG_FILE_PATH,
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def sed(regex: str, path: str):
    """Runs a sed command and logs its outcome."""
    command = ["sed", "-i", regex, path]
    logging.info(f"Running command: {' '.join(command)}")
    try:
        result = subprocess.run(
            command, check=True, capture_output=True, text=True
        )
        logging.info("Command executed successfully.")
        if result.stderr:
            logging.warning(f"[sed stderr]: {result.stderr.strip()}")
    except FileNotFoundError:
        logging.error(f"sed command not found. Please ensure 'sed' is installed and in your PATH.")
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed with exit code {e.returncode}")
        logging.error(f"[sed stdout]: {e.stdout.strip()}")
        logging.error(f"[sed stderr]: {e.stderr.strip()}")
    except Exception as e:
        logging.exception(f"An unexpected error occurred while running sed: {e}")


def reload_hyprland():
    """Runs 'hyprctl reload' and logs its outcome."""
    command = ["hyprctl", "reload"]
    logging.info(f"Running command: {' '.join(command)}")
    try:
        result = subprocess.run(
            command, check=True, capture_output=True, text=True
        )
        logging.info("Hyprland reloaded successfully.")
        if result.stdout:
            logging.info(f"[hyprctl stdout]: {result.stdout.strip()}")
        if result.stderr:
            logging.warning(f"[hyprctl stderr]: {result.stderr.strip()}")
    except FileNotFoundError:
        logging.error(f"hyprctl command not found. Please ensure Hyprland is running and 'hyprctl' is in your PATH.")
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed with exit code {e.returncode}")
        logging.error(f"[hyprctl stdout]: {e.stdout.strip()}")
        logging.error(f"[hyprctl stderr]: {e.stderr.strip()}")
    except Exception as e:
        logging.exception(f"An unexpected error occurred while running hyprctl: {e}")


def set_refresh_rate(target_rate: int):
    """Sets the refresh rate by modifying the config file, but only if the TARGET_MONITOR is found."""
    if target_rate == MAX_REFRESH_RATE:
        from_rate, to_rate = MIN_REFRESH_RATE, MAX_REFRESH_RATE
        logging.info(f"AC power connected. Attempting to set refresh rate to max for {TARGET_MONITOR}.")
    elif target_rate == MIN_REFRESH_RATE:
        from_rate, to_rate = MAX_REFRESH_RATE, MIN_REFRESH_RATE
        logging.info(f"AC power disconnected. Attempting to set refresh rate to min for {TARGET_MONITOR}.")
    else:
        logging.error(f"Invalid target rate: {target_rate}")
        return

    try:
        with open(CONFIG_FILE_PATH, "r") as file:
            file_lines = file.readlines()

        target_line = None
        for line in file_lines:
            if TARGET_MONITOR in line:
                target_line = line
                break

        if target_line:
            if f"@{from_rate}" in target_line:
                regex = f"/{TARGET_MONITOR}/s/@{from_rate}/@{to_rate}/g"
                sed(regex, CONFIG_FILE_PATH)
                reload_hyprland()
            else:
                logging.info(f"Refresh rate for {TARGET_MONITOR} is not @{from_rate}. No change needed.")
        else:
            logging.warning(f"Target '{TARGET_MONITOR}' not found in {CONFIG_FILE_PATH}. No changes made.")

    except FileNotFoundError:
        logging.error(f"Config file not found at: {CONFIG_FILE_PATH}")
    except Exception as e:
        logging.exception(f"Failed to read config file: {e}")


def check_initial_power_status():
    """Initially, AC status can be unreliable. Retry until it is valid."""
    while True:
        try:
            with open(AC_STATUS_FILE_PATH, "r") as file:
                online = file.read().strip()
                logging.info(f"[Initial check] Raw power status is '{online}'")
                if online == "0":
                    set_refresh_rate(MIN_REFRESH_RATE)
                    break
                elif online == "1":
                    set_refresh_rate(MAX_REFRESH_RATE)
                    break
                else:
                    logging.warning(f"[Initial check] Invalid status: {online}. Retrying...")
        except FileNotFoundError:
            logging.error(f"AC status file not found at {AC_STATUS_FILE_PATH}. Exiting.")
            exit(1)
        except Exception as e:
            logging.exception(f"Error during initial check: {e}. Retrying...")

        time.sleep(1)


def ac_event_handler(client, action, device, user_data):
    if action == "change" and device.get_property("SUBSYSTEM") == "power_supply":
        online = device.get_property("POWER_SUPPLY_ONLINE")
        logging.info(f"[AC event] online status is: {online}")

        if online == "1":
            set_refresh_rate(MAX_REFRESH_RATE)
        elif online == "0":
            set_refresh_rate(MIN_REFRESH_RATE)

        if device.get_property("POWER_SUPPLY_CAPACITY_LEVEL") == "critical":
            logging.warning("Battery level is critical!")
            subprocess.run(
                [
                    "notify-send",
                    "--urgency=critical",
                    "Battery critical!",
                ]
            )


def main():
    logging.info("--- Auto Refresh Rate Service Started ---")
    client = GUdev.Client(subsystems=["power_supply"])
    check_initial_power_status()
    client.connect("uevent", ac_event_handler, None)
    loop = GLib.MainLoop()
    try:
        loop.run()
    except KeyboardInterrupt:
        logging.info("--- Service stopped by user ---")
        pass


if __name__ == "__main__":
    """
    Monitors power supply events and automatically changes monitor refresh rate.
    Only Hyprland Window Manager is supported.
    """
    main()
