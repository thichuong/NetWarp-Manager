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

# Locate the AppImage
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPIMAGE_SRC="$PROJECT_DIR/src-tauri/target/release/bundle/appimage/wiwarp_0.1.0_amd64.AppImage"
ICON_SRC="$PROJECT_DIR/src-tauri/icons/icon.png"

if [ ! -f "$APPIMAGE_SRC" ]; then
  echo -e "${RED}Error: AppImage not found at $APPIMAGE_SRC${NC}"
  echo -e "Please ensure you have built the application using 'npm run tauri build' first."
  exit 1
fi

# 1. Create target directories
echo -e "${BLUE}Creating directory /opt/wiwarp...${NC}"
mkdir -p /opt/wiwarp

# 2. Copy AppImage and make it executable
echo -e "${BLUE}Copying AppImage to /opt/wiwarp/wiwarp.AppImage...${NC}"
cp "$APPIMAGE_SRC" /opt/wiwarp/wiwarp.AppImage
chmod +x /opt/wiwarp/wiwarp.AppImage

# 3. Copy application icon
if [ -f "$ICON_SRC" ]; then
  echo -e "${BLUE}Copying application icon...${NC}"
  cp "$ICON_SRC" /opt/wiwarp/icon.png
else
  echo -e "${RED}Warning: Icon not found at $ICON_SRC. Skipping icon installation.${NC}"
fi

# 4. Create terminal command wrapper in /usr/local/bin/wiwarp
echo -e "${BLUE}Creating terminal command '/usr/local/bin/wiwarp'...${NC}"
cat << 'EOF' > /usr/local/bin/wiwarp
#!/bin/bash
# Terminal wrapper for WiWarp (NetWarp Manager)
export WEBKIT_DISABLE_COMPOSITING_MODE=1
exec /opt/wiwarp/wiwarp.AppImage "$@"
EOF

chmod +x /usr/local/bin/wiwarp

# 5. Create desktop entry shortcut
echo -e "${BLUE}Creating application launcher shortcut...${NC}"
cat << EOF > /usr/share/applications/wiwarp.desktop
[Desktop Entry]
Name=WiWarp
Comment=Manage Wi-Fi and Cloudflare WARP connections seamlessly
Exec=env WEBKIT_DISABLE_COMPOSITING_MODE=1 /opt/wiwarp/wiwarp.AppImage
Icon=/opt/wiwarp/icon.png
Terminal=false
Type=Application
Categories=Network;Utility;System;
Keywords=wifi;warp;cloudflare;network;vpn;
StartupNotify=true
EOF

echo -e "${GREEN}${BOLD}✔ WiWarp has been successfully installed!${NC}"
echo -e "--------------------------------------------------"
echo -e "1. You can now launch it from the application menu (search for ${BOLD}WiWarp${NC})."
echo -e "2. Or run it directly from the terminal using the command: ${BOLD}wiwarp${NC}"
echo -e "--------------------------------------------------"
