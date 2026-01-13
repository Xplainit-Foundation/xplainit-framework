/**
 * Xplainit Node.js CPU Profiler-Based Tracer
 * 
 * Alternative approach using V8 CPU Profiler
 * More efficient than Inspector API for production use
 */

const inspector = require('inspector');
const fs = require('fs');

class XplainitProfilerTracer {
    constructor(rustBackend) {
        this.rustBackend = rustBackend;
        this.session = null;
        this.profiling = false;
        this.profileData = null;
    }

    /**
     * Start CPU profiling
     */
    start() {
        if (this.profiling) {
            console.warn('Profiler already running');
            return;
        }

        this.session = new inspector.Session();
        this.session.connect();

        this.session.post('Profiler.enable', () => {
            this.session.post('Profiler.start', () => {
                this.profiling = true;
                console.log('Xplainit profiler started');
            });
        });
    }

    /**
     * Stop CPU profiling and extract trace
     */
    stop(callback) {
        if (!this.profiling) {
            console.warn('Profiler not running');
            return;
        }

        this.session.post('Profiler.stop', (err, { profile }) => {
            this.profiling = false;

            if (err) {
                console.error('Failed to stop profiler:', err);
                callback(err);
                return;
            }

            this.profileData = profile;
            this.processProfile(profile);

            if (callback) callback(null, profile);
        });

        this.session.disconnect();
        this.session = null;
    }

    /**
     * Process CPU profile and extract function calls
     */
    processProfile(profile) {
        if (!profile || !profile.nodes) {
            console.warn('Invalid profile data');
            return;
        }

        const functionCalls = [];

        // Parse profile nodes
        for (const node of profile.nodes) {
            const { callFrame } = node;
            
            if (callFrame && callFrame.functionName) {
                functionCalls.push({
                    name: callFrame.functionName,
                    file: callFrame.url,
                    line: callFrame.lineNumber,
                    column: callFrame.columnNumber,
                    hitCount: node.hitCount || 0,
                });
            }
        }

        // Send to Rust backend
        if (this.rustBackend && this.rustBackend.process_profile_data) {
            try {
                this.rustBackend.process_profile_data(JSON.stringify(functionCalls));
            } catch (error) {
                console.error('Error sending profile data to Rust:', error);
            }
        }

        console.log(`Processed ${functionCalls.length} function calls from profile`);
    }

    /**
     * Save profile to file for analysis
     */
    saveProfile(filename) {
        if (!this.profileData) {
            console.warn('No profile data available');
            return;
        }

        const json = JSON.stringify(this.profileData, null, 2);
        fs.writeFileSync(filename, json);
        console.log(`Profile saved to ${filename}`);
    }

    /**
     * Get collected function calls
     */
    getFunctionCalls() {
        if (!this.profileData || !this.profileData.nodes) {
            return [];
        }

        return this.profileData.nodes
            .filter(node => node.callFrame && node.callFrame.functionName)
            .map(node => ({
                name: node.callFrame.functionName,
                file: node.callFrame.url,
                line: node.callFrame.lineNumber,
            }));
    }
}

module.exports = { XplainitProfilerTracer };
