/**
 * Xplainit Node.js JavaScript Tracing Package
 *
 * Exports the wrapper-based automatic tracer (XplainitNodeTracer) plus the
 * Inspector/Profiler-based tracers and async/await tracking utilities.
 *
 * The primary, sandbox-verifiable entry point is XplainitNodeTracer, which
 * accepts an injectable backend (native Neon addon in production, or a mock in
 * tests) implementing onFunctionEnter / onFunctionExit / onException.
 */

'use strict';

const { XplainitNodeTracer, stringifyValue, buildArgsObject } = require('./tracer');
const { XplainitProfilerTracer } = require('./profiler_tracer');
const { XplainitAsyncTracer, tracePromise, traceAsync } = require('./async_tracer');

/**
 * Best-effort loader for the native Neon addon. Returns the addon object, or
 * null if it cannot be loaded (e.g. not built, or running under a runtime that
 * cannot load N-API addons). Callers should fall back to a JS backend.
 */
function loadNativeBackend() {
    try {
        // Built by `cargo build --release` + the npm build step into index.node.
        return require('../index.node');
    } catch (_err) {
        return null;
    }
}

module.exports = {
    XplainitNodeTracer,
    XplainitProfilerTracer,
    XplainitAsyncTracer,
    tracePromise,
    traceAsync,
    stringifyValue,
    buildArgsObject,
    loadNativeBackend,
};
