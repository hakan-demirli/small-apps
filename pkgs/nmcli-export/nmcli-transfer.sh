#!/usr/bin/env bash
# Usage: sudo <app> > import_connections.sh

set -e

if [ "$(id -u)" -ne 0 ]; then
  echo "Error: This script must be run as root to access NetworkManager secrets." >&2
  exit 1
fi

echo "#!/usr/bin/env bash"
echo "# Generated import script for NetworkManager connections"
echo "# strips hardware-specific bindings (MAC, interface) for portability"
echo
echo "set -e"
echo
echo "if [ \"\$(id -u)\" -ne 0 ]; then echo 'Please run as root'; exit 1; fi"
echo

for file in /etc/NetworkManager/system-connections/*.nmconnection; do
  [ -e "$file" ] || continue
  filename=$(basename "$file")

  echo "echo 'Importing $filename...'"
  echo "cat > /etc/NetworkManager/system-connections/\"$filename\" << 'EOF'"
  grep -vE '^(mac-address=|interface-name=|permissions=)' "$file"
  echo "EOF"
  echo "chmod 600 /etc/NetworkManager/system-connections/\"$filename\""
  echo "chown root:root /etc/NetworkManager/system-connections/\"$filename\""
  echo
done

echo "echo 'Reloading NetworkManager connections...'"
echo "nmcli con reload"
echo "echo 'Done. Connections imported and reloaded.'"
