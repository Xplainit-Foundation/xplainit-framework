/**
 * Xplainit Node.js JavaScript Tracing Package
 * 
 * Exports both Inspector-based and Profiler-based tracers,
 * plus async/await tracking utilities
 */

const { XplainitNodeTracer } = require('./tracer');
const { XplainitProfilerTracer } = require('./profiler_tracer');
const { XplainitAsyncTracer, tracePromise, traceAsync } = require('./async_tracer');

module.exports = {
    XplainitNodeTracer,
    XplainitProfilerTracer,
    XplainitAsyncTracer,
    tracePromise,
    traceAsync,
};
