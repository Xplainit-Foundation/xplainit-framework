#!/bin/bash
# Build script for C tracing example
#
# This script compiles the example with GCC instrumentation

echo "================================================"
echo "Building Xplainit C Tracing Example"
echo "================================================"
echo ""

# Check if GCC is available
if ! command -v gcc &> /dev/null; then
    echo "Error: GCC not found. Please install GCC."
    exit 1
fi

echo "Compiling with -finstrument-functions..."
echo ""

# Compile the example
gcc -finstrument-functions \
    -rdynamic \
    -DXPLAINIT_DEBUG=1 \
    -g \
    -O0 \
    -Wall \
    -Wextra \
    example_traced.c \
    ../lib/trace.c \
    -lpthread \
    -ldl \
    -o example_traced

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
    echo ""
    echo "Run with:"
    echo "  XPLAINIT_DEBUG=1 ./example_traced"
    echo ""
    echo "Or redirect stderr to see trace output:"
    echo "  XPLAINIT_DEBUG=1 ./example_traced 2>trace.log"
    echo ""
else
    echo "✗ Build failed!"
    exit 1
fi
