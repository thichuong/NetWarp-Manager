#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Define colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color
BLUE='\033[0;34m'
BOLD='\033[1m'

echo -e "${BLUE}${BOLD}=== WiWarp Fedora Installer ===${NC}"

# Check if run as root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Error: Please run this script with sudo or as root!${NC}"
  echo -e "Usage: ${BOLD}sudo ./install.sh${NC}"
  exit 1
fi

# Locate the compiled Slint binary
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_SRC="$PROJECT_DIR/target/release/netwarp-manager"
ICON_SRC="$PROJECT_DIR/assets/logo.svg"

if [ ! -f "$BINARY_SRC" ]; then
  echo -e "${RED}Error: NetWarp Slint binary not found at $BINARY_SRC${NC}"
  echo -e "Please ensure you have built the application using 'cargo build --release' first."
  exit 1
fi

# 1. Create target directories
echo -e "${BLUE}Creating directory /opt/wiwarp...${NC}"
mkdir -p /opt/wiwarp

# 2. Copy binary and make it executable
echo -e "${BLUE}Copying Slint binary to /opt/wiwarp/netwarp-manager...${NC}"
cp "$BINARY_SRC" /opt/wiwarp/netwarp-manager
chmod +x /opt/wiwarp/netwarp-manager

# 3. Copy application icon
if [ -f "$ICON_SRC" ]; then
  echo -e "${BLUE}Copying application icon...${NC}"
  cp "$ICON_SRC" /opt/wiwarp/icon.svg
else
  echo -e "${RED}Warning: Icon not found at $ICON_SRC. Skipping icon installation.${NC}"
fi

# 4. Create terminal command wrapper in /usr/local/bin/wiwarp
echo -e "${BLUE}Creating terminal command '/usr/local/bin/wiwarp'...${NC}"
cat << 'EOF' > /usr/local/bin/wiwarp
#!/bin/bash
# Terminal wrapper for WiWarp (NetWarp Manager - Slint Engine)
exec /opt/wiwarp/netwarp-manager "$@"
EOF

chmod +x /usr/local/bin/wiwarp

# 5. Create desktop entry shortcut
echo -e "${BLUE}Creating application launcher shortcut...${NC}"
cat << EOF > /usr/share/applications/wiwarp.desktop
[Desktop Entry]
Name=WiWarp
Comment=Manage Wi-Fi and Cloudflare WARP connections seamlessly (Slint Native UI)
Exec=/opt/wiwarp/netwarp-manager
Icon=/opt/wiwarp/icon.svg
Terminal=false
Type=Application
Categories=Network;Utility;System;
Keywords=wifi;warp;cloudflare;network;vpn;
StartupNotify=true
EOF

echo -e "${GREEN}${BOLD}✔ WiWarp Slint Native has been successfully installed!${NC}"
echo -e "--------------------------------------------------"
echo -e "1. You can now launch it from the application menu (search for ${BOLD}WiWarp${NC})."
echo -e "2. Or run it directly from the terminal using the command: ${BOLD}wiwarp${NC}"
echo -e "--------------------------------------------------"

