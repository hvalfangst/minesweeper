#!/bin/bash

# Build the WebAssembly package
echo "Building WebAssembly package..."
wasm-pack build --target web --out-dir pkg --no-typescript

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful! Starting Python HTTP server..."

    # Check if port 8000 is already in use and kill the process if it is
    PORT=8000
    if lsof -i :$PORT > /dev/null 2>&1; then
        echo "Port $PORT is already in use. Attempting to free it..."
        PID=$(lsof -t -i :$PORT)
        kill -9 $PID
        echo "Freed port $PORT."
    fi

    # Start Python HTTP server on port 8000
    python3 -m http.server 8000
else
    echo "Build failed!"
    exit 1
fi