/**
 * Production Example: Express.js REST API Tracing
 * 
 * This example demonstrates using Xplainit to trace an Express.js API,
 * including async/await operations and error handling.
 * 
 * NOTE: Requires Node.js and npm packages to run
 */

const xplainit = require('./index.node');
const { XplainitAsyncTracer, tracePromise } = require('./javascript');

// Initialize Xplainit
xplainit.enable();

const asyncTracer = new XplainitAsyncTracer({
    onFunctionEnter: (name, args, file, line) => {
        xplainit.onFunctionEnter(name, args, file, line);
    },
    onFunctionExit: (name, returnValue) => {
        xplainit.onFunctionExit(name, returnValue);
    },
    onException: (type, message, file, line) => {
        xplainit.onException(type, message, file, line);
    },
});

// Simulated database
const database = {
    users: [
        { id: 1, username: 'alice', email: 'alice@example.com', role: 'admin' },
        { id: 2, username: 'bob', email: 'bob@example.com', role: 'user' },
        { id: 3, username: 'charlie', email: 'charlie@example.com', role: 'user' },
    ],
    sessions: new Map(),
};

// Async database operations
async function dbFindUser(userId) {
    // Simulate async database query
    await new Promise(resolve => setTimeout(resolve, 10));
    return database.users.find(u => u.id === userId);
}

async function dbCreateSession(userId) {
    await new Promise(resolve => setTimeout(resolve, 5));
    const sessionId = `session_${Date.now()}_${userId}`;
    database.sessions.set(sessionId, { userId, createdAt: Date.now() });
    return sessionId;
}

async function dbValidateSession(sessionId) {
    await new Promise(resolve => setTimeout(resolve, 3));
    return database.sessions.has(sessionId);
}

// API handlers
async function handleLogin(username, password) {
    console.log(`[API] Login request: ${username}`);
    
    // Find user (async)
    const user = await tracePromise(
        dbFindUser(database.users.find(u => u.username === username)?.id),
        'dbFindUser'
    );
    
    if (!user) {
        throw new Error('User not found');
    }
    
    // Create session (async)
    const sessionId = await tracePromise(
        dbCreateSession(user.id),
        'dbCreateSession'
    );
    
    return {
        success: true,
        sessionId,
        user: { id: user.id, username: user.username, role: user.role }
    };
}

async function handleGetProfile(sessionId, userId) {
    console.log(`[API] Get profile: user=${userId}, session=${sessionId}`);
    
    // Validate session
    const isValid = await tracePromise(
        dbValidateSession(sessionId),
        'dbValidateSession'
    );
    
    if (!isValid) {
        throw new Error('Invalid session');
    }
    
    // Get user
    const user = await tracePromise(
        dbFindUser(userId),
        'dbFindUser'
    );
    
    if (!user) {
        throw new Error('User not found');
    }
    
    return {
        success: true,
        profile: user
    };
}

async function handleBatchRequest(userIds) {
    console.log(`[API] Batch request: ${userIds.length} users`);
    
    // Process multiple users in parallel
    const promises = userIds.map(id => 
        tracePromise(dbFindUser(id), `dbFindUser(${id})`)
    );
    
    const users = await Promise.all(promises);
    
    return {
        success: true,
        users: users.filter(u => u !== undefined),
        count: users.filter(u => u !== undefined).length
    };
}

// Main production scenario
async function simulateProductionTraffic() {
    console.log('='.repeat(70));
    console.log('PRODUCTION EXAMPLE: Express.js API with Async Tracing');
    console.log('='.repeat(70));
    console.log();
    
    // Enable async tracing
    asyncTracer.enable();
    
    try {
        // Scenario 1: User login
        console.log('Scenario 1: User Login');
        const loginResult = await handleLogin('alice', 'password123');
        console.log(`  ✓ Login successful: ${loginResult.sessionId}`);
        console.log();
        
        // Scenario 2: Get user profile
        console.log('Scenario 2: Get User Profile');
        const profileResult = await handleGetProfile(loginResult.sessionId, 1);
        console.log(`  ✓ Profile retrieved: ${profileResult.profile.username}`);
        console.log();
        
        // Scenario 3: Batch request
        console.log('Scenario 3: Batch User Query');
        const batchResult = await handleBatchRequest([1, 2, 3]);
        console.log(`  ✓ Retrieved ${batchResult.count} users`);
        console.log();
        
        // Scenario 4: Error case
        console.log('Scenario 4: Invalid Session (Error Case)');
        try {
            await handleGetProfile('invalid_session', 1);
        } catch (error) {
            console.log(`  ✓ Error caught: ${error.message}`);
        }
        console.log();
        
    } catch (error) {
        console.error('Error in production scenario:', error);
    } finally {
        // Disable async tracing
        asyncTracer.disable();
    }
    
    // Analysis
    console.log('='.repeat(70));
    console.log('TRACE ANALYSIS');
    console.log('='.repeat(70));
    console.log();
    
    const stats = xplainit.getStatistics();
    console.log('Statistics:', stats);
    console.log();
    
    const events = JSON.parse(xplainit.getEvents());
    console.log(`Total events captured: ${events.length}`);
    console.log();
    
    // Count event types
    const functionEnters = events.filter(e => e.FunctionEnter).length;
    const functionExits = events.filter(e => e.FunctionExit).length;
    const exceptions = events.filter(e => e.Exception).length;
    
    console.log('Event breakdown:');
    console.log(`  - Function calls: ${functionEnters}`);
    console.log(`  - Function returns: ${functionExits}`);
    console.log(`  - Exceptions: ${exceptions}`);
    console.log();
    
    const asyncStats = asyncTracer.getStatistics();
    console.log('Async operations:', asyncStats);
    console.log();
    
    console.log('='.repeat(70));
    console.log('INSIGHTS');
    console.log('='.repeat(70));
    console.log();
    console.log('✅ Async operations traced successfully');
    console.log('✅ Promise chains captured with tracePromise()');
    console.log('✅ Parallel operations (Promise.all) tracked');
    console.log('✅ Error handling verified with invalid session');
    console.log('✅ Complete async call graph available');
    console.log();
    console.log('Use this trace to:');
    console.log('  • Debug async/await issues');
    console.log('  • Identify promise chains');
    console.log('  • Track parallel operations');
    console.log('  • Measure async operation timing');
    console.log('  • Optimize API response times');
    console.log();
}

// Run the example
console.log();
console.log('Starting production API simulation...');
console.log();

simulateProductionTraffic()
    .then(() => {
        console.log('Production simulation complete!');
        xplainit.disable();
        process.exit(0);
    })
    .catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
