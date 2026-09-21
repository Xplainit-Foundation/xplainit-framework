/**
 * Xplainit Node.js Automatic Tracer
 *
 * Wraps JavaScript functions, classes, and objects so that every invocation is
 * captured with real argument values, real return values, and real exceptions,
 * then forwarded to a backend that implements the Xplainit callback contract:
 *
 *   backend.onFunctionEnter(name, argsObject, file, line)
 *   backend.onFunctionExit(name, returnValueString)
 *   backend.onException(errorType, message, file, line)
 *
 * The backend is INJECTABLE. In production it is the native Neon addon
 * (require('../index.node')), which records the calls into the Rust
 * RuntimeEngine. In tests it can be a pure-JS mock that records the same
 * callbacks, so the capture logic can be exercised without the native addon.
 *
 * DESIGN NOTE (why wrapper instrumentation instead of the V8 inspector):
 * The V8 Inspector `Debugger.paused` approach requires driving the inspector
 * protocol from a separate thread/agent and stepping the whole program, which
 * is impractical to run deterministically inside a single JS process (and is
 * not reliably supported outside a real `node` binary). Wrapping the functions
 * we want to trace gives us the real live values (arguments, return, thrown
 * error) directly, works under both node and bun, and mirrors what the Python
 * AutoTracer captures.
 */

'use strict';

/**
 * Convert a single runtime value into a short, stable string representation.
 * Mirrors the Python backend behavior where every value is serialized to a
 * string before being handed to the Rust layer.
 */
function stringifyValue(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';

    const t = typeof value;
    if (t === 'string') return value;
    if (t === 'number' || t === 'boolean' || t === 'bigint') return String(value);
    if (t === 'symbol') return value.toString();
    if (t === 'function') return `[Function: ${value.name || 'anonymous'}]`;

    // Objects and arrays: try JSON, fall back to a safe description.
    try {
        return JSON.stringify(value);
    } catch (_err) {
        if (Array.isArray(value)) return `[Array(${value.length})]`;
        return Object.prototype.toString.call(value);
    }
}

/**
 * Build the arguments object ({ arg0: "...", arg1: "..." }) that the backend
 * expects. Keys are positional so the recorded order is preserved.
 */
function buildArgsObject(args) {
    const obj = {};
    for (let i = 0; i < args.length; i++) {
        obj[`arg${i}`] = stringifyValue(args[i]);
    }
    return obj;
}

class XplainitNodeTracer {
    /**
     * @param {object} backend - object exposing onFunctionEnter / onFunctionExit
     *   / onException. Defaults to the native addon if none is provided.
     */
    constructor(backend) {
        this.backend = backend || null;
        this.enabled = false;
        // Guard against re-entrancy from the backend calling into traced code.
        this._recording = false;
    }

    /**
     * Enable tracing. Wrapping only takes effect after enable().
     */
    enable() {
        this.enabled = true;
        return this;
    }

    /**
     * Disable tracing. Wrapped functions become pass-through.
     */
    disable() {
        this.enabled = false;
        return this;
    }

    isEnabled() {
        return this.enabled;
    }

    /**
     * Set / replace the backend at runtime (useful for tests).
     */
    setBackend(backend) {
        this.backend = backend;
        return this;
    }

    _emitEnter(name, args, file, line) {
        if (!this.backend || typeof this.backend.onFunctionEnter !== 'function') return;
        this._recording = true;
        try {
            this.backend.onFunctionEnter(name, buildArgsObject(args), file, line);
        } catch (err) {
            // Never let recording break the traced program.
            // eslint-disable-next-line no-console
            console.error('[xplainit] onFunctionEnter failed:', err && err.message);
        } finally {
            this._recording = false;
        }
    }

    _emitExit(name, returnValue) {
        if (!this.backend || typeof this.backend.onFunctionExit !== 'function') return;
        this._recording = true;
        try {
            this.backend.onFunctionExit(name, stringifyValue(returnValue));
        } catch (err) {
            // eslint-disable-next-line no-console
            console.error('[xplainit] onFunctionExit failed:', err && err.message);
        } finally {
            this._recording = false;
        }
    }

    _emitException(error, file, line) {
        if (!this.backend || typeof this.backend.onException !== 'function') return;
        const errorType = (error && error.constructor && error.constructor.name) || 'Error';
        const message = (error && error.message) || String(error);
        this._recording = true;
        try {
            this.backend.onException(errorType, message, file, line);
        } catch (err) {
            // eslint-disable-next-line no-console
            console.error('[xplainit] onException failed:', err && err.message);
        } finally {
            this._recording = false;
        }
    }

    /**
     * Wrap a single function so calls are traced. Supports both synchronous
     * functions and functions returning a Promise (async/await): for a returned
     * Promise the exit/exception is recorded when the promise settles.
     *
     * @param {Function} fn        - the function to wrap
     * @param {object}   [options] - { name, file, line }
     * @returns {Function} the wrapped function
     */
    wrap(fn, options = {}) {
        if (typeof fn !== 'function') {
            throw new TypeError('wrap() expects a function');
        }

        const tracer = this;
        const name = options.name || fn.name || '<anonymous>';
        const file = options.file || '<unknown>';
        const line = typeof options.line === 'number' ? options.line : 0;

        function wrapped(...args) {
            // Pass through when disabled or while the backend is recording
            // (prevents infinite recursion if a backend calls traced code).
            if (!tracer.enabled || tracer._recording) {
                return fn.apply(this, args);
            }

            tracer._emitEnter(name, args, file, line);

            let result;
            try {
                result = fn.apply(this, args);
            } catch (error) {
                tracer._emitException(error, file, line);
                throw error;
            }

            // Handle async functions / thenables: record settlement.
            if (result && typeof result.then === 'function') {
                return result.then(
                    (value) => {
                        tracer._emitExit(name, value);
                        return value;
                    },
                    (error) => {
                        tracer._emitException(error, file, line);
                        throw error;
                    }
                );
            }

            tracer._emitExit(name, result);
            return result;
        }

        // Preserve identity metadata.
        Object.defineProperty(wrapped, 'name', { value: name, configurable: true });
        wrapped.__xplainitWrapped = true;
        wrapped.__xplainitOriginal = fn;
        return wrapped;
    }

    /**
     * Wrap every own method of a class prototype (and static methods) so that
     * calling `new Klass().method()` is traced. Returns the same class.
     *
     * @param {Function} Klass       - class constructor
     * @param {object}   [options]   - { file }
     */
    wrapClass(Klass, options = {}) {
        if (typeof Klass !== 'function') {
            throw new TypeError('wrapClass() expects a class/constructor');
        }
        const file = options.file || '<unknown>';
        const className = Klass.name || '<anonymous class>';

        // Instance methods live on the prototype.
        const proto = Klass.prototype;
        for (const key of Object.getOwnPropertyNames(proto)) {
            if (key === 'constructor') continue;
            const desc = Object.getOwnPropertyDescriptor(proto, key);
            if (!desc || typeof desc.value !== 'function') continue;
            proto[key] = this.wrap(desc.value, {
                name: `${className}.${key}`,
                file,
                line: desc.value.__xplainitLine || 0,
            });
        }

        // Static methods live on the constructor itself.
        for (const key of Object.getOwnPropertyNames(Klass)) {
            if (['length', 'name', 'prototype'].includes(key)) continue;
            const desc = Object.getOwnPropertyDescriptor(Klass, key);
            if (!desc || typeof desc.value !== 'function' || !desc.writable) continue;
            Klass[key] = this.wrap(desc.value, {
                name: `${className}.${key}`,
                file,
            });
        }

        return Klass;
    }

    /**
     * Wrap every function-valued property of a plain object (e.g. a module's
     * exports). Mutates and returns the object.
     */
    wrapObject(obj, options = {}) {
        if (obj === null || typeof obj !== 'object') {
            throw new TypeError('wrapObject() expects an object');
        }
        const file = options.file || '<unknown>';
        const prefix = options.prefix ? `${options.prefix}.` : '';

        for (const key of Object.keys(obj)) {
            if (typeof obj[key] === 'function' && !obj[key].__xplainitWrapped) {
                obj[key] = this.wrap(obj[key], { name: `${prefix}${key}`, file });
            }
        }
        return obj;
    }
}

module.exports = {
    XplainitNodeTracer,
    stringifyValue,
    buildArgsObject,
};
