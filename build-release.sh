#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/release-build"

# Build Linux release
build_linux() {
    echo "=== Building Linux Release ==="
    cargo build --release

    echo ""
    echo "=== Preparing Linux release folder ==="
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"

    # Copy binary
    cp "$SCRIPT_DIR/target/release/wave_github_gameoff2025" "$BUILD_DIR/tidalcave"

    # Copy assets
    cp -r "$SCRIPT_DIR/assets" "$BUILD_DIR/assets"

    # Copy shaders
    cp -r "$SCRIPT_DIR/shaders" "$BUILD_DIR/shaders"

    echo ""
    echo "=== Creating Linux archive ==="
    cd "$SCRIPT_DIR"
    tar -czvf "tidalcave-linux.tar.gz" -C "$BUILD_DIR" .

    rm -rf "$BUILD_DIR"
    echo "Created: tidalcave-linux.tar.gz"
}

# Build Windows release
build_windows() {
    echo "=== Building Windows Release ==="

    # Check if Windows target is installed
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        echo "Installing Windows target..."
        rustup target add x86_64-pc-windows-gnu
    fi

    # Check for MinGW
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "Error: MinGW not found. Install with:"
        echo "  sudo apt install mingw-w64"
        exit 1
    fi

    cargo build --release --target x86_64-pc-windows-gnu

    echo ""
    echo "=== Preparing Windows release folder ==="
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"

    # Copy binary
    cp "$SCRIPT_DIR/target/x86_64-pc-windows-gnu/release/wave_github_gameoff2025.exe" "$BUILD_DIR/tidalcave.exe"

    # Copy assets
    cp -r "$SCRIPT_DIR/assets" "$BUILD_DIR/assets"

    # Copy shaders
    cp -r "$SCRIPT_DIR/shaders" "$BUILD_DIR/shaders"

    echo ""
    echo "=== Creating Windows archive ==="
    cd "$SCRIPT_DIR"
    zip -r "tidalcave-windows.zip" "$BUILD_DIR"/*
    # Re-add folders with structure
    cd "$BUILD_DIR"
    zip -r "$SCRIPT_DIR/tidalcave-windows.zip" assets shaders

    cd "$SCRIPT_DIR"
    rm -rf "$BUILD_DIR"
    echo "Created: tidalcave-windows.zip"
}

# Parse arguments
case "${1:-all}" in
    linux)
        build_linux
        ;;
    windows)
        build_windows
        ;;
    all)
        build_linux
        echo ""
        build_windows
        ;;
    *)
        echo "Usage: $0 [linux|windows|all]"
        exit 1
        ;;
esac

echo ""
echo "=== Release build complete! ==="
