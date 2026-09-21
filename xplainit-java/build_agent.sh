#!/usr/bin/env bash
#
# Build and verify the Xplainit Java JVMTI agent end to end.
#
# Steps:
#   1. Detect JAVA_HOME robustly (java/javac may be shims that do not resolve
#      to the real JDK via readlink, so ask the JVM itself).
#   2. cargo build --release  ->  libxplainit_java.so (JNI natives)
#   3. gcc -shared            ->  libxplainit_agent.so (JVMTI agent)
#   4. javac                  ->  classes/ (no external jars; offline-safe)
#   5. Run BasicExample under -agentpath and ASSERT real method events were
#      captured (expected method names present, non-empty). Exits non-zero on
#      any failure so this is a real gate.
#
# Usage:  cd xplainit-java && bash build_agent.sh

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"

echo "==> Detecting JAVA_HOME"
if [ -z "${JAVA_HOME:-}" ] || [ ! -f "${JAVA_HOME:-/nonexistent}/include/jni.h" ]; then
    # `java -XshowSettings:properties` prints the properties (to stderr) and then
    # exits non-zero because no main class was given; tolerate that under
    # `set -e`/`pipefail` by capturing separately.
    JAVA_PROPS="$(java -XshowSettings:properties 2>&1 || true)"
    JAVA_HOME="$(printf '%s\n' "$JAVA_PROPS" | grep 'java.home' | sed 's/.*= *//')"
fi
echo "    JAVA_HOME=$JAVA_HOME"
if [ ! -f "$JAVA_HOME/include/jni.h" ] || [ ! -f "$JAVA_HOME/include/jvmti.h" ]; then
    echo "ERROR: jni.h / jvmti.h not found under $JAVA_HOME/include" >&2
    exit 1
fi

echo "==> Building Rust JNI library (cargo build --release -p xplainit-java)"
( cd "$REPO_ROOT" && cargo build --release -p xplainit-java )

JNI_LIB="$REPO_ROOT/target/release/libxplainit_java.so"
if [ ! -f "$JNI_LIB" ]; then
    echo "ERROR: $JNI_LIB not produced" >&2
    exit 1
fi
echo "    built $JNI_LIB"

echo "==> Compiling JVMTI agent (libxplainit_agent.so)"
gcc -shared -fPIC -O2 \
    -I"$JAVA_HOME/include" -I"$JAVA_HOME/include/linux" \
    -o "$HERE/libxplainit_agent.so" \
    "$HERE/native/agent.c"
echo "    built $HERE/libxplainit_agent.so"

echo "==> Compiling Java classes (javac, no external jars)"
CLASSES="$HERE/classes"
rm -rf "$CLASSES"
mkdir -p "$CLASSES"
javac -d "$CLASSES" \
    "$HERE/src/main/java/io/xplainit/Xplainit.java" \
    "$HERE/examples/BasicExample.java"
echo "    compiled to $CLASSES"

echo "==> Running BasicExample under the agent"
# The JNI library must be discoverable via java.library.path so
# System.loadLibrary("xplainit_java") resolves.
set +e
OUT="$(java \
    -agentpath:"$HERE/libxplainit_agent.so" \
    -Djava.library.path="$REPO_ROOT/target/release" \
    -cp "$CLASSES" \
    io.xplainit.examples.BasicExample 2>&1)"
STATUS=$?
set -e
echo "-------------------- program output --------------------"
echo "$OUT"
echo "--------------------------------------------------------"

if [ $STATUS -ne 0 ]; then
    echo "ERROR: BasicExample exited with status $STATUS" >&2
    exit 1
fi

echo "==> Asserting captured events"
fail=0
for name in doubleValue fibonacci divide; do
    if echo "$OUT" | grep -q "\"$name\""; then
        echo "    OK: captured method '$name'"
    else
        echo "    MISSING: expected method '$name' not found in captured events" >&2
        fail=1
    fi
done

if echo "$OUT" | grep -q "captured_methods_present=true"; then
    echo "    OK: captured_methods_present=true"
else
    echo "    FAIL: captured_methods_present was not true" >&2
    fail=1
fi

if echo "$OUT" | grep -q "event_count_nonzero=true"; then
    echo "    OK: event_count_nonzero=true"
else
    echo "    FAIL: no events captured" >&2
    fail=1
fi

if [ $fail -ne 0 ]; then
    echo "ERROR: agent verification failed" >&2
    exit 1
fi

echo "==> SUCCESS: JVMTI agent captured real method events from BasicExample"
