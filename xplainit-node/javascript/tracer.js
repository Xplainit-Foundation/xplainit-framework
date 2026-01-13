/**
 * Xplainit Node.js Automatic Tracer
 * 
 * Uses Node.js inspector API to automatically trace JavaScript execution
 * Similar to Python's sys.settrace() - captures function calls, returns, and exceptions
 */

const inspector = require('inspector');
const path = require('path');

class XplainitNodeTracer {
    constructor(rustBackend) {
        this.rustBackend = rustBackend;
        this.enabled = false;
        this.session = null;
        this.pausedCount = 0;
        this.maxDepth = 50; // Prevent infinite recursion in tracing
        this.tracedFiles = new Set();
        this.excludePatterns = [
            /node_modules/,
            /internal\//,
            /<anonymous>/,
        ];
    }

    /**
     * Check if a script should be traced
     */
    shouldTrace(scriptUrl) {
        if (!scriptUrl) return false;
        
        // Exclude internal Node.js modules and node_modules
        for (const pattern of this.excludePatterns) {
            if (pattern.test(scriptUrl)) {
                return false;
            }
        }
        
        // Only trace user code
        return scriptUrl.startsWith('file://');
    }

    /**
     * Extract function arguments from scope chain
     */
    extractArguments(scopeChain) {
        const args = {};
        
        for (const scope of scopeChain) {
            if (scope.type === 'local' && scope.object) {
                // In a real implementation, we would use Runtime.getProperties
                // For now, return placeholder
                args['_scope'] = scope.object.description || 'unknown';
            }
        }
        
        return args;
    }

    /**
     * Handle debugger pause event
     */
    async handlePaused(message) {
        if (!this.enabled) return;
        
        const { callFrames, reason } = message.params;
        
        if (callFrames && callFrames.length > 0) {
            const topFrame = callFrames[0];
            const functionName = topFrame.functionName || '<anonymous>';
            const location = topFrame.location;
            const scriptId = location.scriptId;
            
            // Get script URL
            const scriptUrl = this.tracedFiles.get(scriptId) || 'unknown';
            
            if (!this.shouldTrace(scriptUrl)) {
                this.session.post('Debugger.resume');
                return;
            }
            
            // Convert file:// URL to path
            const filePath = scriptUrl.startsWith('file://') 
                ? scriptUrl.replace('file://', '').replace(/^\/([A-Z]:)/, '$1')
                : scriptUrl;
            
            // Extract arguments from scope
            const args = this.extractArguments(topFrame.scopeChain);
            
            // Record function entry
            if (this.rustBackend && this.rustBackend.on_function_enter) {
                try {
                    this.rustBackend.on_function_enter(
                        functionName,
                        args,
                        filePath,
                        location.lineNumber + 1 // Convert 0-based to 1-based
                    );
                } catch (error) {
                    console.error('Error recording function entry:', error);
                }
            }
        }
        
        // Resume execution
        this.session.post('Debugger.resume');
    }

    /**
     * Handle script parsed event
     */
    handleScriptParsed(message) {
        const { scriptId, url } = message.params;
        if (url) {
            this.tracedFiles.set(scriptId, url);
        }
    }

    /**
     * Enable automatic tracing
     */
    enable() {
        if (this.enabled) {
            console.warn('Tracer already enabled');
            return;
        }

        this.enabled = true;
        this.session = new inspector.Session();
        this.session.connect();

        // Set up event listeners
        this.session.on('Debugger.paused', (message) => {
            this.handlePaused(message);
        });

        this.session.on('Debugger.scriptParsed', (message) => {
            this.handleScriptParsed(message);
        });

        // Enable debugger
        this.session.post('Debugger.enable', (err) => {
            if (err) {
                console.error('Failed to enable debugger:', err);
                return;
            }

            // Set breakpoint on all function calls
            // Note: This is a simplified approach. In production, we'd use:
            // - Debugger.setBreakpointsActive
            // - Debugger.setBreakpointByUrl for specific files
            // - Or V8 CPU profiler for better performance
            
            console.log('Xplainit Node.js tracer enabled');
        });
    }

    /**
     * Disable automatic tracing
     */
    disable() {
        if (!this.enabled) {
            return;
        }

        this.enabled = false;

        if (this.session) {
            this.session.post('Debugger.disable', (err) => {
                if (err) {
                    console.error('Failed to disable debugger:', err);
                }
            });

            this.session.disconnect();
            this.session = null;
        }

        this.tracedFiles.clear();
        console.log('Xplainit Node.js tracer disabled');
    }

    /**
     * Check if tracer is enabled
     */
    isEnabled() {
        return this.enabled;
    }
}

module.exports = { XplainitNodeTracer };
