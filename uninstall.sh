#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Define colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color
BLUE='\033[0;34m'
BOLD='\033[1m'

echo -e "${BLUE}${BOLD}=== WiWarp Linux Uninstaller ===${NC}"

# Check if run as root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Error: Please run this script with sudo or as root!${NC}"
  echo -e "Usage: ${BOLD}sudo ./uninstall.sh${NC}"
  exit 1
fi

# 1. Remove terminal command wrapper
if [ -f /usr/local/bin/wiwarp ]; then
  echo -e "${BLUE}Removing terminal command '/usr/local/bin/wiwarp'...${NC}"
  rm -f /usr/local/bin/wiwarp
fi

# 2. Remove desktop shortcut
if [ -f /usr/share/applications/wiwarp.desktop ]; then
  echo -e "${BLUE}Removing application launcher shortcut...${NC}"
  rm -f /usr/share/applications/wiwarp.desktop
fi

# 3. Remove application files in /opt/wiwarp
if [ -d /opt/wiwarp ]; then
  echo -e "${BLUE}Removing application files from /opt/wiwarp...${NC}"
  rm -rf /opt/wiwarp
fi

echo -e "${GREEN}${BOLD}✔ WiWarp has been successfully uninstalled!${NC}"
echo -e "--------------------------------------------------"
