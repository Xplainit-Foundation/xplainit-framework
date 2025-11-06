package io.xplainit;

import com.google.gson.Gson;
import com.google.gson.JsonObject;

/**
 * Xplainit - Natural Language Explanations for Java Code Execution
 * 
 * This class provides runtime tracing and analysis for Java applications.
 * 
 * @version 0.1.0
 */
public class Xplainit implements AutoCloseable {
    
    static {
        // Load native library
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
        Gson gson = new Gson();
        return gson.fromJson(json, Statistics.class);
    }
    
    /**
     * Statistics about captured events
     */
    public static class Statistics {
        public long total_events;
        public long function_calls;
        public long errors;
        
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
    
    // Native methods
    private static native long nativeCreate();
    private static native void nativeFree(long handle);
    private static native boolean nativeEnable(long handle);
    private static native boolean nativeDisable(long handle);
    private static native boolean nativeIsEnabled(long handle);
    private static native String nativeGetEvents(long handle);
    private static native boolean nativeClearEvents(long handle);
    private static native String nativeGetStatistics(long handle);
}
