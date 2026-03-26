#!/bin/bash
# scripts/download-clangd.sh

set -e

CLANGD_VERSION="17.0.6"
BINARIES_DIR="binaries"

mkdir -p "$BINARIES_DIR"/{linux-x64,darwin-x64,darwin-arm64,windows-x64}

echo "Downloading clangd $CLANGD_VERSION..."

# Linux x64
echo "Downloading Linux x64..."
curl -L "https://github.com/llvm/llvm-project/releases/download/llvmorg-$CLANGD_VERSION/clang+llvm-$CLANGD_VERSION-x86_64-linux-gnu-ubuntu-22.04.tar.xz" |
  tar xJ --strip-components=2 -C "$BINARIES_DIR/linux-x64" \
    "clang+llvm-$CLANGD_VERSION-x86_64-linux-gnu-ubuntu-22.04/bin/clangd"

# macOS ARM (M1/M2)
echo "Downloading macOS ARM..."
curl -L "https://github.com/llvm/llvm-project/releases/download/llvmorg-$CLANGD_VERSION/clang+llvm-$CLANGD_VERSION-arm64-apple-darwin22.0.tar.xz" |
  tar xJ --strip-components=2 -C "$BINARIES_DIR/darwin-arm64" \
    "clang+llvm-$CLANGD_VERSION-arm64-apple-darwin22.0/bin/clangd"

# macOS x64
echo "Downloading macOS x64..."
curl -L "https://github.com/llvm/llvm-project/releases/download/llvmorg-$CLANGD_VERSION/clang+llvm-$CLANGD_VERSION-x86_64-apple-darwin22.0.tar.xz" |
  tar xJ --strip-components=2 -C "$BINARIES_DIR/darwin-x64" \
    "clang+llvm-$CLANGD_VERSION-x86_64-apple-darwin22.0/bin/clangd"

# Windows
echo "Downloading Windows x64..."
# Windows binaries need manual download from https://github.com/llvm/llvm-project/releases
echo "Please manually download Windows LLVM and extract clangd.exe to $BINARIES_DIR/windows-x64/"

chmod +x "$BINARIES_DIR"/*/clangd

echo "✓ clangd binaries downloaded!"
