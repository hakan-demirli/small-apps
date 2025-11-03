#!/usr/bin/env python3

import argparse
import configparser
import glob
import json
import os
import subprocess
import unicodedata
from multiprocessing import Pool
from pathlib import Path

import yt_dlp


def sanitizeString(in_str):
    BANNED_CHARACTERS = "\\/:*?<>|`;![\\]()^#%&!@:+=},\"{'~"
    for banned_character in BANNED_CHARACTERS:
        in_str = str(in_str).replace(banned_character, "")
    in_str = "".join(
        ch for ch in in_str if ((unicodedata.category(ch)[0] != "C") or (ch == " "))
    )
    in_str = in_str.replace(" ", "_")
    return in_str


def syncFolder(playlist_tuple):
    playlist_folder, playlist_url, max_size = playlist_tuple
    MAX_FILE_SIZE = max_size

    ydl_opts = {
        "outtmpl": f"{playlist_folder}/%(playlist_index)s_%(title)s.%(ext)s.tmpdownload",
        "format": "bestaudio",
        "postprocessors": [
            {
                "key": "FFmpegExtractAudio",
                "preferredcodec": "opus",
                "preferredquality": "best",
            },
            {"key": "EmbedThumbnail"},
        ],
        "download_archive": f"{playlist_folder}/downloaded.txt",
        "embedthumbnail": True,
        "ignoreerrors": True,
        "verbose": True,
        "writethumbnail": True,
        "writedescription": True,
        "writeinfojson": True,
        "embedmetadata": True,
        "force_overwrites": True,
        "ignore_no_formats_error": True,
    }

    print(f"--- Syncing Playlist for: {os.path.basename(playlist_folder)} ---")
    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        ydl.download([playlist_url])

    print(f"--- Post-processing files for: {os.path.basename(playlist_folder)} ---")
    temp_files = glob.glob(f"{playlist_folder}/*.tmpdownload.*")

    if not temp_files:
        print("  -> No new files to process.")
        return

    for tmp_filepath in temp_files:
        final_filepath_base = (
            tmp_filepath.split(".tmpdownload")[0].rsplit(".", 1)[0] + ".opus"
        )
        try:
            file_size = os.path.getsize(tmp_filepath)
            print(
                f"  Processing '{os.path.basename(final_filepath_base)}' - Size: {file_size / 1024 / 1024:.2f} MB"
            )

            if file_size > MAX_FILE_SIZE:
                print("  -> File is too large. Splitting into parts...")
                split_command = [
                    "split",
                    "-b",
                    str(MAX_FILE_SIZE),
                    tmp_filepath,
                    f"{final_filepath_base}.part",
                ]
                subprocess.run(
                    split_command, check=True, capture_output=True, text=True
                )
                print("  -> Splitting complete.")
                os.remove(tmp_filepath)
            else:
                print("  -> File is within size limits. Renaming.")
                os.rename(tmp_filepath, final_filepath_base)

        except FileNotFoundError:
            print(
                f"  -> WARNING: File not found during post-processing: {tmp_filepath}"
            )
        except subprocess.CalledProcessError as e:
            print(f"  -> ERROR: Failed to split file {tmp_filepath}.")
            print(f"  -> Stderr: {e.stderr}")
        except Exception as e:
            print(
                f"  -> ERROR: An unexpected error occurred while processing {tmp_filepath}: {e}"
            )


def discover_and_sync_playlists(music_dir, max_size):
    gitmodules_path = os.path.join(music_dir, ".gitmodules")
    if not os.path.exists(gitmodules_path):
        print(f"ERROR: .gitmodules not found in '{music_dir}'.")
        print(
            "Please run this script from the root of your main music collection repository."
        )
        return

    config = configparser.ConfigParser()
    config.read(gitmodules_path)

    playlist_tuples = []
    print("Discovering playlists from .gitmodules...")
    for section in config.sections():
        if config.has_option(section, "path"):
            submodule_path = config.get(section, "path")
            full_submodule_path = os.path.join(music_dir, submodule_path)
            metadata_file = os.path.join(full_submodule_path, "metadata.json")
            if os.path.exists(metadata_file):
                print(f"  [+] Found submodule: {submodule_path}")
                with open(metadata_file) as f:
                    try:
                        metadata = json.load(f)
                        playlist_url = metadata.get("playlist_url")
                        if playlist_url:
                            playlist_tuples.append(
                                (full_submodule_path, playlist_url, max_size)
                            )
                        else:
                            print(
                                f"    -> WARNING: 'playlist_url' key not found in {metadata_file}"
                            )
                    except json.JSONDecodeError:
                        print(
                            f"    -> ERROR: Could not parse {metadata_file}. Please check for valid JSON."
                        )
            else:
                print(
                    f"  [!] WARNING: No metadata.json found in submodule: {submodule_path}"
                )

    if not playlist_tuples:
        print("\nNo valid playlists with metadata.json found to sync.")
        return

    print(f"\nStarting sync for {len(playlist_tuples)} playlists...")
    with Pool() as pool:
        pool.map(syncFolder, playlist_tuples)

    print("\n[SYNC COMPLETED]")


def create_m3u8_playlists(directory):
    for dirpath, _, filenames in os.walk(directory):
        unique_audio_entries = set()

        for f in filenames:
            if f.lower().endswith(".opus"):
                unique_audio_entries.add(f"./{f}")
            elif ".part" in f:
                base_name = f.rsplit(".part", 1)[0]

                if base_name.lower().endswith(".opus"):
                    unique_audio_entries.add(f"./{base_name}")

        if unique_audio_entries:
            sorted_audio_files = sorted(unique_audio_entries)

            playlist_name = f"{Path(dirpath).name}.m3u8"
            playlist_path = os.path.join(dirpath, playlist_name)
            with open(playlist_path, "w") as playlist_file:
                for audio_file in sorted_audio_files:
                    playlist_file.write(f"{audio_file}\n")
            print(f"Created playlist: {playlist_path}")


def clean_dir(directory):
    file_types = [
        "*.jpg",
        "*.png",
        "*.webp",
        "*.json",
        "*.description",
        "*.m3u8",
    ]
    print("\nCleaning up auxiliary files...")
    for file_type in file_types:
        for dirpath, _, _ in os.walk(directory):
            for file in glob.glob(os.path.join(dirpath, file_type)):
                if os.path.basename(file) == "metadata.json":
                    continue
                try:
                    os.remove(file)
                    print(f"Deleted {file}")
                except OSError as e:
                    print(f"Error: {file} : {e.strerror}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Discovers and syncs YouTube playlists based on .gitmodules structure."
    )
    parser.add_argument(
        "--music_dir",
        type=str,
        default="/home/emre/.local/share/sounds",  # WARNING: abs path with username
        help="The root music directory containing the .gitmodules file.",
    )
    parser.add_argument(
        "--only-m3u8",
        action="store_true",
        help="Only regenerate the M3U8 playlist files.",
    )
    parser.add_argument(
        "--max-size",
        type=int,
        default=40 * 1024 * 1024,
    )

    args = parser.parse_args()

    if args.only_m3u8:
        create_m3u8_playlists(args.music_dir)
    else:
        discover_and_sync_playlists(args.music_dir, args.max_size)
        clean_dir(args.music_dir)
        create_m3u8_playlists(args.music_dir)
        print("\n[ALL TASKS COMPLETED]")
