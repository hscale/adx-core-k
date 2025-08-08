#!/bin/bash

# Development script to run the CLI with tsx for hot reloading
# Usage: ./scripts/dev-cli.sh <command> [args...]

if [ $# -eq 0 ]; then
    echo "Usage: $0 <command> [args...]"
    echo "Commands: setup, sync, status, enable, disable, watch"
    exit 1
fi

npx tsx src/index.ts "$@"