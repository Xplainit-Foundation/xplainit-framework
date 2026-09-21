package io.xplainit;

/**
 * Xplainit - Natural Language Explanations for Java Code Execution
 *
 * This class provides runtime tracing and analysis for Java applications.
 *
 * <p>Two recording paths are supported:
 * <ul>
 *   <li><b>Instance path</b> - create an {@link Xplainit} instance and call
 *       {@link #onMethodEnter(String, String, int)} / {@link #onMethodExit(String)} /
 *       {@link #onException(String, String, String, int)} directly from Java.</li>
 *   <li><b>Agent path</b> - attach the JVMTI agent with
 *       {@code -agentpath:.../libxplainit_agent.so}. The agent calls the static
 *       {@code agent*} methods below (via JNI) as methods enter/exit, funnelling
 *       into a single process-wide runtime.</li>
 * </ul>
 *
 * @version 0.1.0
 */
public class Xplainit implements AutoCloseable {

    static {
        // Load native library. The library name resolves to libxplainit_java.so
        // on Linux. When the JVMTI agent is attached it links this same library.
        System.loadLibrary("xplainit_java");
    }

    private long nativeHandle;
    private boolean closed = false;

    /**
     * Create a new Xplainit tracer instance
     */
    public Xplainit() {
        this.nativeHandle = nativeCreate();
    }

    /**
     * Enable tracing
     *
     * @return true if successfully enabled
     */
    public boolean enable() {
        checkClosed();
        return nativeEnable(nativeHandle);
    }

    /**
     * Disable tracing
     *
     * @return true if successfully disabled
     */
    public boolean disable() {
        checkClosed();
        return nativeDisable(nativeHandle);
    }

    /**
     * Check if tracing is enabled
     *
     * @return true if enabled, false otherwise
     */
    public boolean isEnabled() {
        checkClosed();
        return nativeIsEnabled(nativeHandle);
    }

    /**
     * Record a method-entry event on this instance.
     *
     * @param name      method name
     * @param signature JVM method signature or owning class (used as location file)
     * @param line      source line (may be negative/0 if unknown)
     * @return true if the event was recorded
     */
    public boolean onMethodEnter(String name, String signature, int line) {
        checkClosed();
        return nativeOnMethodEnter(nativeHandle, name, signature, line);
    }

    /**
     * Record a method-exit event on this instance.
     *
     * @param name method name
     * @return true if the event was recorded
     */
    public boolean onMethodExit(String name) {
        checkClosed();
        return nativeOnMethodExit(nativeHandle, name);
    }

    /**
     * Record an exception event on this instance.
     *
     * @param errorType exception class name
     * @param message   exception message
     * @param file      source file
     * @param line      source line
     * @return true if the event was recorded
     */
    public boolean onException(String errorType, String message, String file, int line) {
        checkClosed();
        return nativeOnException(nativeHandle, errorType, message, file, line);
    }

    /**
     * Get all captured events as JSON string
     *
     * @return JSON array of events
     */
    public String getEvents() {
        checkClosed();
        return nativeGetEvents(nativeHandle);
    }

    /**
     * Clear all captured events
     *
     * @return true if successfully cleared
     */
    public boolean clearEvents() {
        checkClosed();
        return nativeClearEvents(nativeHandle);
    }

    /**
     * Get statistics about captured events
     *
     * @return Statistics object
     */
    public Statistics getStatistics() {
        checkClosed();
        String json = nativeGetStatistics(nativeHandle);
        return Statistics.parse(json);
    }

    // ===== Agent (JVMTI) recording path =====

    /**
     * Initialise the process-wide agent runtime. Called by the JVMTI agent at
     * startup; safe to call from Java too.
     *
     * @return true on success
     */
    public static boolean agentInit() {
        return nativeAgentInit();
    }

    /** Record a method-entry event into the agent runtime (called by the agent). */
    public static boolean agentOnMethodEnter(String name, String signature, int line) {
        return nativeAgentOnMethodEnter(name, signature, line);
    }

    /** Record a method-exit event into the agent runtime (called by the agent). */
    public static boolean agentOnMethodExit(String name) {
        return nativeAgentOnMethodExit(name);
    }

    /** Record an exception event into the agent runtime (called by the agent). */
    public static boolean agentOnException(String errorType, String message, String file, int line) {
        return nativeAgentOnException(errorType, message, file, line);
    }

    /** Get all events captured by the agent runtime as JSON (drains the store). */
    public static String agentGetEvents() {
        return nativeAgentGetEvents();
    }

    /** Get statistics for the agent runtime (drains the store). */
    public static Statistics agentGetStatistics() {
        return Statistics.parse(nativeAgentGetStatistics());
    }

    /**
     * Statistics about captured events.
     *
     * <p>Parsed by hand from the small, well-known JSON shape emitted by the
     * native layer so the library has no external JSON dependency (important for
     * offline / no-Maven builds).
     */
    public static class Statistics {
        public long total_events;
        public long function_calls;
        public long errors;

        /**
         * Parse a statistics JSON string of the form
         * {@code {"total_events":N,"function_calls":N,"errors":N}}.
         */
        public static Statistics parse(String json) {
            Statistics s = new Statistics();
            if (json == null) {
                return s;
            }
            s.total_events = extractLong(json, "total_events");
            s.function_calls = extractLong(json, "function_calls");
            s.errors = extractLong(json, "errors");
            return s;
        }

        private static long extractLong(String json, String key) {
            String needle = "\"" + key + "\"";
            int k = json.indexOf(needle);
            if (k < 0) {
                return 0L;
            }
            int colon = json.indexOf(':', k + needle.length());
            if (colon < 0) {
                return 0L;
            }
            int i = colon + 1;
            while (i < json.length() && Character.isWhitespace(json.charAt(i))) {
                i++;
            }
            int start = i;
            if (i < json.length() && (json.charAt(i) == '-' || json.charAt(i) == '+')) {
                i++;
            }
            while (i < json.length() && Character.isDigit(json.charAt(i))) {
                i++;
            }
            if (i == start) {
                return 0L;
            }
            try {
                return Long.parseLong(json.substring(start, i));
            } catch (NumberFormatException e) {
                return 0L;
            }
        }

        @Override
        public String toString() {
            return String.format(
                "Statistics{total_events=%d, function_calls=%d, errors=%d}",
                total_events, function_calls, errors
            );
        }
    }

    /**
     * Close and free the native resources
     */
    @Override
    public void close() {
        if (!closed) {
            nativeFree(nativeHandle);
            closed = true;
        }
    }

    private void checkClosed() {
        if (closed) {
            throw new IllegalStateException("Xplainit instance has been closed");
        }
    }

    // Native methods (instance path)
    private static native long nativeCreate();
    private static native void nativeFree(long handle);
    private static native boolean nativeEnable(long handle);
    private static native boolean nativeDisable(long handle);
    private static native boolean nativeIsEnabled(long handle);
    private static native String nativeGetEvents(long handle);
    private static native boolean nativeClearEvents(long handle);
    private static native String nativeGetStatistics(long handle);
    private static native boolean nativeOnMethodEnter(long handle, String name, String signature, int line);
    private static native boolean nativeOnMethodExit(long handle, String name);
    private static native boolean nativeOnException(long handle, String errorType, String message, String file, int line);

    // Native methods (agent path)
    private static native boolean nativeAgentInit();
    private static native boolean nativeAgentOnMethodEnter(String name, String signature, int line);
    private static native boolean nativeAgentOnMethodExit(String name);
    private static native boolean nativeAgentOnException(String errorType, String message, String file, int line);
    private static native String nativeAgentGetEvents();
    private static native String nativeAgentGetStatistics();
}
