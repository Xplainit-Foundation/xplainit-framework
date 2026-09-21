#!/bin/bash
# Build, run, and VERIFY the Xplainit C -finstrument-functions tracing example.
#
# This script is a real gate: it exits non-zero if the build fails, the program
# fails to run, or no real events (with expected function names) were captured.
set -euo pipefail

echo "================================================"
echo "Building Xplainit C Tracing Example"
echo "================================================"

# Resolve paths relative to this script so it works from any CWD.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
C_CRATE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${C_CRATE_DIR}/.." && pwd)"
RELEASE_DIR="${REPO_ROOT}/target/release"
INCLUDE_DIR="${C_CRATE_DIR}/include"
LIB_DIR="${C_CRATE_DIR}/lib"

CC="${CC:-gcc}"
if ! command -v "${CC}" >/dev/null 2>&1; then
    echo "Error: C compiler '${CC}' not found." >&2
    exit 1
fi

echo "Building libxplainit_c via cargo build --release..."
cargo build --release -p xplainit-c --manifest-path "${REPO_ROOT}/Cargo.toml"

if [ ! -f "${RELEASE_DIR}/libxplainit_c.so" ] && [ ! -f "${RELEASE_DIR}/libxplainit_c.a" ]; then
    echo "Error: libxplainit_c not found in ${RELEASE_DIR}" >&2
    exit 1
fi

echo "Compiling shim + example with -finstrument-functions..."
BIN="${SCRIPT_DIR}/example_traced"
"${CC}" -finstrument-functions \
    -rdynamic \
    -g -O0 -Wall -Wextra \
    -I"${INCLUDE_DIR}" \
    -I"${LIB_DIR}" \
    "${SCRIPT_DIR}/example_traced.c" \
    "${LIB_DIR}/trace.c" \
    -L"${RELEASE_DIR}" \
    -lxplainit_c \
    -ldl \
    -lpthread \
    -o "${BIN}"

echo "✓ Build successful!"
echo ""
echo "Running instrumented program..."
OUTPUT_FILE="$(mktemp)"
# Point the dynamic loader at the freshly built shared library.
LD_LIBRARY_PATH="${RELEASE_DIR}:${LD_LIBRARY_PATH:-}" "${BIN}" | tee "${OUTPUT_FILE}"

echo ""
echo "================================================"
echo "Verifying captured events..."
echo "================================================"

FAIL=0

# 1. The statistics summary must report a non-zero total of captured events.
TOTAL_LINE="$(grep -E 'total_events *= *[0-9]+' "${OUTPUT_FILE}" || true)"
TOTAL_COUNT="$(echo "${TOTAL_LINE}" | grep -oE '[0-9]+' | head -n1 || true)"
if [ -z "${TOTAL_COUNT}" ] || [ "${TOTAL_COUNT}" -le 0 ]; then
    echo "✗ FAIL: no events captured (total_events=${TOTAL_COUNT:-none})" >&2
    FAIL=1
else
    echo "✓ Captured ${TOTAL_COUNT} events"
fi

# 2. The events JSON must be present and include expected function names.
if ! grep -q "EVENTS_JSON_BEGIN" "${OUTPUT_FILE}"; then
    echo "✗ FAIL: events JSON block missing" >&2
    FAIL=1
fi

for fn in factorial add multiply calculate_something; do
    if grep -q "\"${fn}\"" "${OUTPUT_FILE}"; then
        echo "✓ Found traced function: ${fn}"
    else
        echo "✗ FAIL: expected traced function '${fn}' not found in events" >&2
        FAIL=1
    fi
done

# 3. The error path must have recorded an exception.
if grep -q "DivisionByZero" "${OUTPUT_FILE}"; then
    echo "✓ Error path recorded (DivisionByZero)"
else
    echo "✗ FAIL: expected DivisionByZero exception not found in events" >&2
    FAIL=1
fi

rm -f "${OUTPUT_FILE}"

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "✗ Verification FAILED"
    exit 1
fi

echo ""
echo "✓ Verification PASSED: real events captured from instrumented C program"
