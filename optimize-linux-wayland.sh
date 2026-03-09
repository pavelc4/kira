#!/bin/bash

echo "Kira Linux/Wayland Performance Optimization Setup"
echo "======================================"
echo ""

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "This script is designed for Linux (Ubuntu/Debian) with Wayland"
    echo "For macOS/Windows, no optimization needed"
    exit 1
fi

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo "Please don't run this script as root"
    exit 1
fi

echo "Installing optimization dependencies..."
echo ""

# Update package list
echo "Updating package list..."
sudo apt update

# Install mold linker
echo ""
echo "Installing mold linker..."
if command -v mold &> /dev/null; then
    echo "mold is already installed ($(mold --version))"
else
    sudo apt install -y mold
    if [ $? -eq 0 ]; then
        echo "mold installed successfully"
    else
        echo "Failed to install mold"
        exit 1
    fi
fi

# Install clang
echo ""
echo "Installing clang..."
if command -v clang &> /dev/null; then
    echo "clang is already installed ($(clang --version | head -n1))"
else
    sudo apt install -y clang
    if [ $? -eq 0 ]; then
        echo "clang installed successfully"
    else
        echo "Failed to install clang"
        exit 1
    fi
fi

# Verify installations
echo ""
echo "Verifying installations..."
echo ""

if command -v mold &> /dev/null && command -v clang &> /dev/null; then
    echo "All dependencies installed successfully!"
    echo ""
    echo "Installed versions:"
    echo "  - mold: $(mold --version)"
    echo "  - clang: $(clang --version | head -n1)"
else
    echo "Some dependencies failed to install"
    exit 1
fi

echo ""
echo "Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Run: bun install"
echo "  2. Run: bun run tauri:x11 dev  (recommended)"
echo "  3. Or:  bun run tauri:wayland dev"
echo ""
echo "Read CONTRIBUTING.md for more development tips"
