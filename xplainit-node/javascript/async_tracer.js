/**
 * Xplainit Node.js Async/Await Tracing
 * 
 * Provides utilities for tracing asynchronous JavaScript code including
 * Promises, async/await, and callbacks.
 */

const async_hooks = require('async_hooks');

class XplainitAsyncTracer {
    constructor(rustBackend) {
        this.rustBackend = rustBackend;
        this.enabled = false;
        this.asyncHook = null;
        this.asyncContexts = new Map();
        this.asyncIdStack = [];
    }

    /**
     * Enable async tracing
     */
    enable() {
        if (this.enabled) {
            console.warn('Async tracer already enabled');
            return;
        }

        this.enabled = true;

        // Create async_hooks instance
        this.asyncHook = async_hooks.createHook({
            init: this.onAsyncInit.bind(this),
            before: this.onAsyncBefore.bind(this),
            after: this.onAsyncAfter.bind(this),
            destroy: this.onAsyncDestroy.bind(this),
        });

        this.asyncHook.enable();
        console.log('Xplainit async tracer enabled');
    }

    /**
     * Disable async tracing
     */
    disable() {
        if (!this.enabled) {
            return;
        }

        this.enabled = false;

        if (this.asyncHook) {
            this.asyncHook.disable();
            this.asyncHook = null;
        }

        this.asyncContexts.clear();
        this.asyncIdStack = [];
        console.log('Xplainit async tracer disabled');
    }

    /**
     * Called when a new async resource is created
     */
    onAsyncInit(asyncId, type, triggerAsyncId, resource) {
        if (!this.enabled) return;

        // Store context about this async operation
        this.asyncContexts.set(asyncId, {
            type,
            triggerAsyncId,
            parentId: this.getCurrentAsyncId(),
            startTime: Date.now(),
            resource,
        });

        // Record async operation creation
        if (this.rustBackend && this.rustBackend.onAsyncCreate) {
            try {
                this.rustBackend.onAsyncCreate(asyncId, type, triggerAsyncId);
            } catch (error) {
                // Ignore errors to not break execution
            }
        }
    }

    /**
     * Called before an async callback executes
     */
    onAsyncBefore(asyncId) {
        if (!this.enabled) return;

        this.asyncIdStack.push(asyncId);

        const context = this.asyncContexts.get(asyncId);
        if (context && this.rustBackend && this.rustBackend.onAsyncEnter) {
            try {
                this.rustBackend.onAsyncEnter(asyncId, context.type);
            } catch (error) {
                // Ignore errors
            }
        }
    }

    /**
     * Called after an async callback completes
     */
    onAsyncAfter(asyncId) {
        if (!this.enabled) return;

        // Pop from stack
        if (this.asyncIdStack.length > 0) {
            this.asyncIdStack.pop();
        }

        const context = this.asyncContexts.get(asyncId);
        if (context && this.rustBackend && this.rustBackend.onAsyncExit) {
            const duration = Date.now() - context.startTime;
            try {
                this.rustBackend.onAsyncExit(asyncId, context.type, duration);
            } catch (error) {
                // Ignore errors
            }
        }
    }

    /**
     * Called when an async resource is destroyed
     */
    onAsyncDestroy(asyncId) {
        if (!this.enabled) return;

        // Clean up context
        this.asyncContexts.delete(asyncId);

        if (this.rustBackend && this.rustBackend.onAsyncDestroy) {
            try {
                this.rustBackend.onAsyncDestroy(asyncId);
            } catch (error) {
                // Ignore errors
            }
        }
    }

    /**
     * Get the current async execution ID
     */
    getCurrentAsyncId() {
        return this.asyncIdStack.length > 0
            ? this.asyncIdStack[this.asyncIdStack.length - 1]
            : async_hooks.executionAsyncId();
    }

    /**
     * Get statistics about async operations
     */
    getStatistics() {
        return {
            activeContexts: this.asyncContexts.size,
            stackDepth: this.asyncIdStack.length,
            enabled: this.enabled,
        };
    }
}

/**
 * Wrapper to trace Promise chains
 */
function tracePromise(promise, name = 'Promise') {
    const startTime = Date.now();

    return promise
        .then((result) => {
            const duration = Date.now() - startTime;
            console.log(`[Trace] ${name} resolved in ${duration}ms`);
            return result;
        })
        .catch((error) => {
            const duration = Date.now() - startTime;
            console.log(`[Trace] ${name} rejected in ${duration}ms:`, error.message);
            throw error;
        });
}

/**
 * Decorator for async functions
 */
function traceAsync(rustBackend, name) {
    return function (target, propertyKey, descriptor) {
        const originalMethod = descriptor.value;

        descriptor.value = async function (...args) {
            const functionName = name || `${target.constructor.name}.${propertyKey}`;
            const startTime = Date.now();

            // Record entry
            if (rustBackend && rustBackend.onFunctionEnter) {
                try {
                    const argsObj = {};
                    args.forEach((arg, i) => {
                        argsObj[`arg${i}`] = String(arg);
                    });
                    rustBackend.onFunctionEnter(functionName, argsObj, __filename, 0);
                } catch (error) {
                    // Ignore
                }
            }

            try {
                const result = await originalMethod.apply(this, args);

                // Record exit
                if (rustBackend && rustBackend.onFunctionExit) {
                    try {
                        rustBackend.onFunctionExit(functionName, String(result));
                    } catch (error) {
                        // Ignore
                    }
                }

                return result;
            } catch (error) {
                // Record exception
                if (rustBackend && rustBackend.onException) {
                    try {
                        rustBackend.onException(
                            error.constructor.name,
                            error.message,
                            __filename,
                            0
                        );
                    } catch (e) {
                        // Ignore
                    }
                }

                throw error;
            }
        };

        return descriptor;
    };
}

module.exports = {
    XplainitAsyncTracer,
    tracePromise,
    traceAsync,
};
