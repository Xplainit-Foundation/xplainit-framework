/**
 * Xplainit Node.js Automatic Tracing Test - Phase 2.2 (FEAT-003)
 *
 * Exercises the wrapper-based XplainitNodeTracer against the plan's cases:
 *   1. simple function      2. recursion
 *   3. exception            4. class method
 *   5. async / await
 *
 * The tracer is driven with an INJECTABLE mock backend (MockBackend) that
 * records every onFunctionEnter / onFunctionExit / onException call. The test
 * asserts the exact sequence of callbacks with correct argument values, return
 * values, and exception data - proving the JS capture logic works WITHOUT the
 * native Neon addon (Node/npm are unavailable in the sandbox; only bun exists).
 *
 * Run:
 *   bun xplainit-node/test_automatic_tracing.js          (mock verification)
 *   bun xplainit-node/test_automatic_tracing.js --native (also try native path)
 *
 * Exits non-zero on any failure.
 */

'use strict';

const { XplainitNodeTracer } = require('./javascript');

// ---------------------------------------------------------------------------
// Tiny assertion harness
// ---------------------------------------------------------------------------
let passCount = 0;
let failCount = 0;
const failures = [];

function check(label, condition, detail) {
    if (condition) {
        passCount++;
        console.log(`  PASS: ${label}`);
    } else {
        failCount++;
        failures.push(label + (detail ? ` (${detail})` : ''));
        console.log(`  FAIL: ${label}${detail ? ` -> ${detail}` : ''}`);
    }
}

function eq(label, actual, expected) {
    check(label, actual === expected, `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

// ---------------------------------------------------------------------------
// Mock backend: records the Xplainit callback contract as an event log.
// Mirrors what the native Neon addon (onFunctionEnter/Exit/Exception) does.
// ---------------------------------------------------------------------------
class MockBackend {
    constructor() {
        this.events = [];
    }
    onFunctionEnter(name, args, file, line) {
        this.events.push({ kind: 'enter', name, args, file, line });
    }
    onFunctionExit(name, returnValue) {
        this.events.push({ kind: 'exit', name, returnValue });
    }
    onException(errorType, message, file, line) {
        this.events.push({ kind: 'exception', errorType, message, file, line });
    }
    clear() {
        this.events = [];
    }
}

// ---------------------------------------------------------------------------
// Test 1: simple function - one enter, one exit, real args + return
// ---------------------------------------------------------------------------
function testSimpleFunction() {
    console.log('\nTest 1: simple function');
    const backend = new MockBackend();
    const tracer = new XplainitNodeTracer(backend).enable();

    const add = tracer.wrap((a, b) => a + b, { name: 'add', file: 'test.js', line: 10 });
    const result = add(2, 3);

    eq('add(2,3) returns 5', result, 5);
    eq('exactly 2 events', backend.events.length, 2);
    eq('first is enter', backend.events[0].kind, 'enter');
    eq('enter name is add', backend.events[0].name, 'add');
    eq('enter arg0 = "2"', backend.events[0].args.arg0, '2');
    eq('enter arg1 = "3"', backend.events[0].args.arg1, '3');
    eq('enter file recorded', backend.events[0].file, 'test.js');
    eq('enter line recorded', backend.events[0].line, 10);
    eq('second is exit', backend.events[1].kind, 'exit');
    eq('exit returnValue = "5"', backend.events[1].returnValue, '5');
}

// ---------------------------------------------------------------------------
// Test 2: recursion - fibonacci(4) produces matched enter/exit pairs
// ---------------------------------------------------------------------------
function testRecursion() {
    console.log('\nTest 2: recursion');
    const backend = new MockBackend();
    const tracer = new XplainitNodeTracer(backend).enable();

    // Self-referential wrapped recursion: fib calls the wrapped version.
    let fib;
    fib = tracer.wrap(function fibonacci(n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }, { name: 'fibonacci', file: 'test.js' });

    const result = fib(4);
    eq('fibonacci(4) returns 3', result, 3);

    // fib(4) -> 9 total calls (T(n)=T(n-1)+T(n-2)+1): 9 enters + 9 exits.
    const enters = backend.events.filter((e) => e.kind === 'enter');
    const exits = backend.events.filter((e) => e.kind === 'exit');
    eq('9 enter events', enters.length, 9);
    eq('9 exit events', exits.length, 9);
    check('all enters named fibonacci', enters.every((e) => e.name === 'fibonacci'));
    eq('first enter arg0 = "4"', enters[0].args.arg0, '4');
    // The outermost call is entered first and exits last with value 3.
    eq('last exit returnValue = "3"', exits[exits.length - 1].returnValue, '3');
    check('no exceptions in recursion', backend.events.every((e) => e.kind !== 'exception'));
}

// ---------------------------------------------------------------------------
// Test 3: exception - enter then exception (no exit), error re-thrown
// ---------------------------------------------------------------------------
function testException() {
    console.log('\nTest 3: exception');
    const backend = new MockBackend();
    const tracer = new XplainitNodeTracer(backend).enable();

    const boom = tracer.wrap(function divide(a, b) {
        if (b === 0) throw new RangeError('division by zero');
        return a / b;
    }, { name: 'divide', file: 'test.js', line: 42 });

    let threw = false;
    try {
        boom(10, 0);
    } catch (err) {
        threw = true;
        eq('re-thrown error message preserved', err.message, 'division by zero');
    }

    check('exception propagated to caller', threw);
    eq('2 events (enter + exception)', backend.events.length, 2);
    eq('first is enter', backend.events[0].kind, 'enter');
    eq('second is exception', backend.events[1].kind, 'exception');
    eq('exception type RangeError', backend.events[1].errorType, 'RangeError');
    eq('exception message captured', backend.events[1].message, 'division by zero');
    eq('exception line captured', backend.events[1].line, 42);
    check('no exit event on throw', backend.events.every((e) => e.kind !== 'exit'));
}

// ---------------------------------------------------------------------------
// Test 4: class method - wrapClass traces instance + static methods
// ---------------------------------------------------------------------------
function testClassMethod() {
    console.log('\nTest 4: class method');
    const backend = new MockBackend();
    const tracer = new XplainitNodeTracer(backend).enable();

    class Calculator {
        multiply(a, b) {
            return a * b;
        }
        static square(x) {
            return x * x;
        }
    }
    tracer.wrapClass(Calculator, { file: 'test.js' });

    const calc = new Calculator();
    const product = calc.multiply(6, 7);
    const sq = Calculator.square(5);

    eq('multiply(6,7) returns 42', product, 42);
    eq('square(5) returns 25', sq, 25);

    const enters = backend.events.filter((e) => e.kind === 'enter');
    const exits = backend.events.filter((e) => e.kind === 'exit');
    eq('2 enter events', enters.length, 2);
    eq('2 exit events', exits.length, 2);
    eq('instance method name qualified', enters[0].name, 'Calculator.multiply');
    eq('static method name qualified', enters[1].name, 'Calculator.square');
    eq('multiply exit = "42"', exits[0].returnValue, '42');
    eq('square exit = "25"', exits[1].returnValue, '25');
}

// ---------------------------------------------------------------------------
// Test 5: async / await - exit recorded when the promise settles
// ---------------------------------------------------------------------------
async function testAsync() {
    console.log('\nTest 5: async / await');
    const backend = new MockBackend();
    const tracer = new XplainitNodeTracer(backend).enable();

    const fetchValue = tracer.wrap(async function fetchValue(x) {
        await Promise.resolve();
        return x * 2;
    }, { name: 'fetchValue', file: 'test.js' });

    const value = await fetchValue(21);
    eq('async fetchValue(21) returns 42', value, 42);
    eq('2 events (enter + exit)', backend.events.length, 2);
    eq('enter recorded first', backend.events[0].kind, 'enter');
    eq('exit recorded after settle', backend.events[1].kind, 'exit');
    eq('async exit resolved value = "42"', backend.events[1].returnValue, '42');

    // Rejected async function should record an exception, not an exit.
    const failing = tracer.wrap(async function failing() {
        await Promise.resolve();
        throw new TypeError('async boom');
    }, { name: 'failing', file: 'test.js' });

    backend.clear();
    let rejected = false;
    try {
        await failing();
    } catch (err) {
        rejected = true;
        eq('rejected message preserved', err.message, 'async boom');
    }
    check('async rejection propagated', rejected);
    eq('async reject: 2 events', backend.events.length, 2);
    eq('async reject: exception recorded', backend.events[1].kind, 'exception');
    eq('async reject: error type TypeError', backend.events[1].errorType, 'TypeError');
}

// ---------------------------------------------------------------------------
// Best-effort native path (only with --native). Records exact result/error;
// never affects the pass/fail outcome of the mock verification.
// ---------------------------------------------------------------------------
function tryNativePath() {
    console.log('\nBest-effort native path (Neon addon):');
    const { loadNativeBackend } = require('./javascript');
    const native = loadNativeBackend();
    if (!native) {
        console.log('  INFO: native addon could not be loaded in this runtime.');
        console.log('        (Expected under bun: N-API .node addons are not loadable;');
        console.log('         a real Node.js + `npm run build` is required to verify the');
        console.log('         end-to-end JS -> Rust round-trip.)');
        return;
    }
    try {
        native.enable();
        native.clearEvents();
        const tracer = new XplainitNodeTracer(native).enable();
        const add = tracer.wrap((a, b) => a + b, { name: 'add', file: 'native.js', line: 1 });
        const r = add(4, 5);
        const events = JSON.parse(native.getEvents());
        console.log(`  NATIVE OK: add(4,5)=${r}, addon recorded ${events.length} event(s).`);
        native.disable();
    } catch (err) {
        console.log('  NATIVE ERROR while exercising addon:', err && err.message);
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------
async function main() {
    console.log('======================================================================');
    console.log('XPLAINIT NODE.JS AUTOMATIC TRACING TEST - Phase 2.2 (mock backend)');
    console.log('======================================================================');

    testSimpleFunction();
    testRecursion();
    testException();
    testClassMethod();
    await testAsync();

    if (process.argv.includes('--native')) {
        tryNativePath();
    }

    console.log('\n======================================================================');
    console.log(`SUMMARY: ${passCount} passed, ${failCount} failed`);
    console.log('======================================================================');
    if (failCount > 0) {
        console.log('FAILURES:');
        for (const f of failures) console.log('  - ' + f);
        console.log('\nRESULT: FAIL');
        process.exit(1);
    }
    console.log('RESULT: PASS');
    process.exit(0);
}

main().catch((err) => {
    console.error('Unexpected test harness error:', err);
    process.exit(1);
});
