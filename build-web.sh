#!/bin/bash
set -e

# Source emscripten environment if available
if [ -f /opt/emscripten/emsdk/emsdk_env.sh ]; then
    source /opt/emscripten/emsdk/emsdk_env.sh > /dev/null 2>&1
elif [ -f "$HOME/emsdk/emsdk_env.sh" ]; then
    source "$HOME/emsdk/emsdk_env.sh" > /dev/null 2>&1
fi

# Check if emcc is available
if ! command -v emcc &> /dev/null; then
    echo "Error: emcc not found. Please install Emscripten SDK and source emsdk_env.sh"
    echo "  https://emscripten.org/docs/getting_started/downloads.html"
    exit 1
fi

EMSCRIPTEN_DIR=$(dirname $(which emcc))

echo "Building for WebAssembly..."

# Set EMCC_CFLAGS for raylib-sys compilation
export EMCC_CFLAGS="-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"

# Build for WASM with cargo
cargo build --target wasm32-unknown-emscripten --release

echo "Copying files to web directory..."

# Copy the compiled files
cp target/wasm32-unknown-emscripten/release/wave_github_gameoff2025.js web/
cp target/wasm32-unknown-emscripten/release/wave_github_gameoff2025.wasm web/

# Copy .data file if it exists
if [ -f target/wasm32-unknown-emscripten/release/wave_github_gameoff2025.data ]; then
    cp target/wasm32-unknown-emscripten/release/wave_github_gameoff2025.data web/
fi

echo "Packaging assets with file_packager..."

# Use Emscripten's file_packager to create the .data file with preloaded assets
python3 "$EMSCRIPTEN_DIR/tools/file_packager.py" \
    web/wave_github_gameoff2025.data \
    --preload assets@assets \
    --preload shaders@shaders \
    --js-output=web/wave_github_gameoff2025.data.js

echo ""
echo "WASM build complete! Files are in web/"
echo ""
echo "To test: cd web && python -m http.server"
