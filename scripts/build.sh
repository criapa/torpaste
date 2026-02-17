#!/bin/bash
# Build script for TorChat-Paste

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}TorChat-Paste Build Script${NC}"
echo "================================"

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    echo -e "${RED}Flutter not found. Please install Flutter first.${NC}"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Rust not found. Please install Rust first.${NC}"
    exit 1
fi

# Build Rust core
echo -e "${YELLOW}Building Rust core...${NC}"
cd core
cargo build --release
cd ..

# Build Flutter app for current platform
echo -e "${YELLOW}Building Flutter app...${NC}"
cd ui
flutter build apk --release
cd ..

echo -e "${GREEN}Build completed successfully!${NC}"
echo "APK location: ui/build/app/outputs/flutter-apk/"
