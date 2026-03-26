#!/bin/bash
# scripts/build-pylsp.sh

set -e

echo "Building standalone pylsp..."

# Install dependencies
pip install python-lsp-server pyinstaller

# Build for current platform
pyinstaller --onefile --name pylsp $(which pylsp)

# Determine platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM="linux-x64"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  if [[ $(uname -m) == "arm64" ]]; then
    PLATFORM="darwin-arm64"
  else
    PLATFORM="darwin-x64"
  fi
else
  echo "Unsupported platform: $OSTYPE"
  exit 1
fi

# Copy to binaries
mkdir -p "binaries/$PLATFORM"
cp dist/pylsp "binaries/$PLATFORM/"
chmod +x "binaries/$PLATFORM/pylsp"

echo "✓ pylsp built for $PLATFORM"
echo "Note: For cross-platform builds, run this on each platform"
