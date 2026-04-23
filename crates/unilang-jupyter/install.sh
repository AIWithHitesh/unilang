#!/usr/bin/env bash
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.
#
# Install the UniLang Jupyter kernel.
#
# Usage:
#   ./install.sh [--user]            # install for current user (default)
#   ./install.sh --system            # install system-wide (needs sudo)
#
# The unilang-jupyter binary must already be on PATH (cargo install or cargo build --release).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$SCRIPT_DIR/kernel"
INSTALL_MODE="${1:---user}"

# Verify the binary exists.
if ! command -v unilang-jupyter &>/dev/null; then
    echo "ERROR: 'unilang-jupyter' not found on PATH."
    echo "Build it first with:  cargo build --release -p unilang-jupyter"
    echo "Then add target/release/ to your PATH, or run:"
    echo "  cargo install --path crates/unilang-jupyter"
    exit 1
fi

# Verify Jupyter is available.
if ! command -v jupyter &>/dev/null; then
    echo "ERROR: 'jupyter' not found on PATH. Install Jupyter first."
    exit 1
fi

echo "Installing UniLang Jupyter kernel from: $KERNEL_DIR"

case "$INSTALL_MODE" in
    --user)
        jupyter kernelspec install "$KERNEL_DIR" --name unilang --user
        ;;
    --system)
        jupyter kernelspec install "$KERNEL_DIR" --name unilang
        ;;
    *)
        echo "Usage: $0 [--user|--system]"
        exit 1
        ;;
esac

echo ""
echo "UniLang kernel installed. Verify with:"
echo "  jupyter kernelspec list"
