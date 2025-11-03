#!/usr/bin/env python3
import http.server
import json
import mimetypes  # Needed for guessing MIME type
import os
import socketserver
import sys  # For error messages
import threading
from pathlib import Path

# --- Configuration ---
HOST = "localhost"
PORT = 8003
# Get the directory where the script is located
SCRIPT_DIR = Path(__file__).parent.resolve()
# *Important*: STATIC_DIR is still needed for index.html, css, js etc.
# It just won't contain the dynamic wallpaper.
STATIC_DIR = SCRIPT_DIR / "static"
STATE_FILE_DIR = Path.home() / ".local" / "share" / "homepage"
STATE_FILE_PATH = STATE_FILE_DIR / "state.json"

# Define the path to the actual wallpaper file
WALLPAPER_SOURCE_PATH = Path("/tmp/wp.png")

# Ensure the state directory exists
STATE_FILE_DIR.mkdir(parents=True, exist_ok=True)
# Ensure the static directory exists (optional, but good practice)
# STATIC_DIR.mkdir(exist_ok=True) # Uncomment if you want the script to create it

# --- State Management ---
state_lock = threading.Lock()  # To prevent race conditions when saving state


def load_state():
    """Loads state from the JSON file."""
    with state_lock:
        if not STATE_FILE_PATH.is_file():
            return {"tabs": [{"name": "Home", "items": []}], "activeTab": 0}
        try:
            with open(STATE_FILE_PATH, encoding="utf-8") as f:
                data = json.load(f)
                if (
                    not isinstance(data, dict)
                    or "tabs" not in data
                    or "activeTab" not in data
                ):
                    print(
                        "Warning: Invalid state file format. Using default state.",
                        file=sys.stderr,
                    )
                    return {"tabs": [{"name": "Home", "items": []}], "activeTab": 0}
                return data
        except json.JSONDecodeError:
            print(
                f"Error: Could not decode JSON from {STATE_FILE_PATH}. Using default state.",
                file=sys.stderr,
            )
            return {"tabs": [{"name": "Home", "items": []}], "activeTab": 0}
        except Exception as e:
            print(f"Error loading state: {e}. Using default state.", file=sys.stderr)
            return {"tabs": [{"name": "Home", "items": []}], "activeTab": 0}


def save_state(data):
    """Saves state to the JSON file."""
    with state_lock:
        try:
            with open(STATE_FILE_PATH, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=4)
        except Exception as e:
            print(f"Error saving state: {e}", file=sys.stderr)


# --- HTTP Request Handler ---
class MyHttpRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        # Serve static files (HTML, CSS, JS) from STATIC_DIR
        if not STATIC_DIR.is_dir():
            print(f"Error: Static directory not found: {STATIC_DIR}", file=sys.stderr)
            print(
                "Please ensure the 'static' directory exists relative to the script.",
                file=sys.stderr,
            )
            raise FileNotFoundError(f"Static directory not found: {STATIC_DIR}")
        super().__init__(*args, directory=str(STATIC_DIR), **kwargs)

    def _send_response(self, code, content_type, body):
        """Helper to send a complete HTTP response."""
        self.send_response(code)
        self.send_header("Content-type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def send_file_response(self, file_path, content_type):
        """Helper to send a file as response, handling potential errors."""
        try:
            with open(file_path, "rb") as f:
                fs = os.fstat(f.fileno())
                file_size = fs[6]
                self.send_response(200)
                self.send_header("Content-type", content_type)
                self.send_header("Content-Length", str(file_size))
                # --- Add Cache-Control Headers ---
                # Ensure the browser always checks for a new version
                self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
                self.send_header(
                    "Pragma", "no-cache"
                )  # HTTP/1.0 backward compatibility
                self.send_header("Expires", "0")  # Proxies
                self.end_headers()
                # Read and send file content chunk by chunk (safer for large files)
                # For a wallpaper, reading all at once might be okay, but this is more robust
                buffer_size = 8192
                while True:
                    chunk = f.read(buffer_size)
                    if not chunk:
                        break
                    self.wfile.write(chunk)
                return True  # Indicate success
        except FileNotFoundError:
            print(f"Wallpaper source file not found: {file_path}", file=sys.stderr)
            # Send 404 from here if desired, or let the main GET handler do it
            return False
        except OSError as e:
            print(f"Error reading wallpaper file {file_path}: {e}", file=sys.stderr)
            # Could send 500 here, but maybe 404 is safer if file becomes unreadable
            return False
        except Exception as e:
            print(f"Unexpected error serving file {file_path}: {e}", file=sys.stderr)
            return False  # Indicate failure

    def do_GET(self):
        """Handles GET requests."""
        # --- Handle API endpoint ---
        if self.path == "/api/state":
            try:
                current_state = load_state()
                state_json = json.dumps(current_state).encode("utf-8")
                self._send_response(200, "application/json", state_json)
            except Exception as e:
                print(f"Error fetching state: {e}", file=sys.stderr)
                self._send_response(500, "text/plain", b"Internal Server Error")
            return  # API handled

        # --- Handle dynamic wallpaper request ---
        # IMPORTANT: Update your CSS to use url("/wp.png") (leading slash)
        if self.path == "/wp.png":
            if WALLPAPER_SOURCE_PATH.is_file():
                # Guess mime type, default to image/png if guess fails
                mime_type, _ = mimetypes.guess_type(WALLPAPER_SOURCE_PATH)
                if not mime_type:
                    mime_type = "image/png"  # Default assumption

                if not self.send_file_response(WALLPAPER_SOURCE_PATH, mime_type):
                    # send_file_response failed (e.g., IOError reading), send 500 or 404
                    # Let's send 404 for simplicity, as the file is "not available" to serve
                    self.send_error(404, "Wallpaper file not found or unreadable")
            else:
                # The source file /tmp/wp.png does not exist
                print(
                    f"Wallpaper source file not found: {WALLPAPER_SOURCE_PATH}",
                    file=sys.stderr,
                )
                self.send_error(404, "Wallpaper file not found")
            return  # Wallpaper request handled (or errored)

        # --- Default: Serve static files from STATIC_DIR ---
        # Let the parent SimpleHTTPRequestHandler handle other paths (index.html, etc.)
        try:
            return super().do_GET()
        except BrokenPipeError:
            # print("Broken pipe error, client likely disconnected.")
            pass
        except FileNotFoundError:
            # This might not be reached as SimpleHTTPRequestHandler sends its own 404
            self._send_response(404, "text/plain", b"Not Found")
        except Exception as e:
            print(f"Error during GET {self.path}: {e}", file=sys.stderr)
            # Avoid sending response if headers might already be sent by parent

    # --- do_POST remains the same ---
    def do_POST(self):
        """Handles POST requests."""
        # API endpoint for updating state
        if self.path == "/api/state":
            try:
                content_length = int(self.headers["Content-Length"])
                if content_length <= 0:
                    raise ValueError("Content-Length must be positive")

                post_data_bytes = self.rfile.read(content_length)
                if not post_data_bytes:
                    raise ValueError("Received empty POST data")

                post_data_str = post_data_bytes.decode("utf-8")
                new_state = json.loads(post_data_str)

                # Basic validation (add more if needed)
                if (
                    not isinstance(new_state, dict)
                    or "tabs" not in new_state
                    or not isinstance(new_state["tabs"], list)
                    or "activeTab" not in new_state
                    or not isinstance(new_state["activeTab"], int)
                ):
                    raise ValueError("Invalid state data structure received")

                save_state(new_state)
                self._send_response(200, "application/json", b'{"status": "ok"}')
            except json.JSONDecodeError:
                print("Error: Invalid JSON received in POST request.", file=sys.stderr)
                self._send_response(
                    400, "application/json", b'{"error": "Invalid JSON"}'
                )
            except (ValueError, TypeError) as ve:
                print(f"Error: Invalid POST data or value: {ve}", file=sys.stderr)
                self._send_response(
                    400,
                    "application/json",
                    f'{{"error": "Bad Request: {ve}"}}'.encode(),
                )
            except Exception as e:
                print(f"Error processing POST /api/state: {e}", file=sys.stderr)
                self._send_response(
                    500, "application/json", b'{"error": "Internal Server Error"}'
                )
        else:
            self._send_response(405, "text/plain", b"Method Not Allowed")


# --- Main Execution Guard ---
if __name__ == "__main__":
    # Check if static directory exists on startup (still needed for base files)
    if not STATIC_DIR.is_dir():
        print(f"Error: Static directory not found: {STATIC_DIR}", file=sys.stderr)
        print(
            "Please ensure the 'static' directory exists relative to this script",
            file=sys.stderr,
        )
        print("containing your index.html, CSS, and JS files.", file=sys.stderr)
        exit(1)

    # --- Server Setup ---
    Handler = MyHttpRequestHandler
    httpd = socketserver.ThreadingTCPServer((HOST, PORT), Handler)

    print(f"Serving homepage assets from read-only: {STATIC_DIR}")
    print(f"Dynamically serving wallpaper from: {WALLPAPER_SOURCE_PATH}")
    print(f"Listening on http://{HOST}:{PORT}")
    print(f"State file: {STATE_FILE_PATH}")
    print("Press Ctrl+C to stop the server.")

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down server...")
        httpd.shutdown()
        print("Server stopped.")
    except Exception as e:
        print(f"\nAn unexpected error occurred: {e}", file=sys.stderr)
        httpd.shutdown()
        print("Server stopped due to error.")
