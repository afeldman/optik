#!/bin/bash

# optik Setup Script

set -e

echo "🎥 optik - Setup Script"
echo "======================="
echo ""

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "✅ Python version: $PYTHON_VERSION"

# Create virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv .venv
fi

# Activate virtual environment
source .venv/bin/activate
echo "✅ Virtual environment activated"

# Install dependencies
echo ""
echo "📥 Installing dependencies..."
pip install -q --upgrade pip
pip install -q -e ".[dev]"

# Install camera SDKs
echo "📥 Installing camera SDKs..."
pip install -q pypylon ids-peak 2>/dev/null || \
    echo "⚠️  Camera SDKs may need manual installation"

# Verify installation
echo ""
echo "✅ optik is ready to use!"
echo ""
echo "Quick start:"
echo "  optik list                    # List available cameras"
echo "  optik grab --camera 0         # Grab frame from camera 0"
echo "  optik mux-server --port 5555  # Start multiplexer server"
echo ""
echo "Examples:"
echo "  python examples_basic.py      # Basic camera discovery"
echo "  python examples_mux.py        # Multiplexer client"
echo ""
echo "Tests:"
echo "  pytest                        # Run all tests"
echo "  pytest -v                     # Verbose output"
echo ""
