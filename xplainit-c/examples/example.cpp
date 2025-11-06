/*
 * Xplainit C++ Example
 * 
 * Demonstrates using Xplainit from C++ code with RAII wrapper.
 */

#include <iostream>
#include <memory>
#include <string>
#include "../include/xplainit-c.h"

// RAII wrapper for XplainitHandle
class Xplainit {
private:
    XplainitHandle* handle_;

public:
    Xplainit() : handle_(xplainit_create()) {
        if (handle_ == nullptr) {
            throw std::runtime_error("Failed to create Xplainit handle");
        }
    }

    ~Xplainit() {
        if (handle_ != nullptr) {
            xplainit_free(handle_);
        }
    }

    // Delete copy constructor and assignment
    Xplainit(const Xplainit&) = delete;
    Xplainit& operator=(const Xplainit&) = delete;

    // Move constructor and assignment
    Xplainit(Xplainit&& other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    Xplainit& operator=(Xplainit&& other) noexcept {
        if (this != &other) {
            if (handle_ != nullptr) {
                xplainit_free(handle_);
            }
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    void enable() {
        if (!xplainit_enable(handle_)) {
            throw std::runtime_error("Failed to enable tracing");
        }
    }

    void disable() {
        if (!xplainit_disable(handle_)) {
            throw std::runtime_error("Failed to disable tracing");
        }
    }

    bool is_enabled() const {
        return xplainit_is_enabled(handle_) != 0;
    }

    std::string get_events() const {
        char* events = xplainit_get_events(handle_);
        if (events == nullptr) {
            return "[]";
        }
        std::string result(events);
        xplainit_free_string(events);
        return result;
    }

    void clear_events() {
        xplainit_clear_events(handle_);
    }

    struct Statistics {
        size_t total_events;
        size_t function_calls;
        size_t errors;
    };

    Statistics get_statistics() const {
        Statistics stats{};
        xplainit_get_statistics(
            handle_,
            &stats.total_events,
            &stats.function_calls,
            &stats.errors
        );
        return stats;
    }

    static std::string version() {
        return xplainit_version();
    }
};

int main() {
    try {
        std::cout << "Xplainit C++ Example\n";
        std::cout << "====================\n\n";

        // Create Xplainit instance (RAII - auto cleanup)
        std::cout << "Creating Xplainit runtime...\n";
        Xplainit tracer;

        // Get version
        std::cout << "Xplainit version: " << Xplainit::version() << "\n\n";

        // Enable tracing
        std::cout << "Enabling tracing...\n";
        tracer.enable();

        if (tracer.is_enabled()) {
            std::cout << "Tracing is enabled\n\n";
        }

        // Your C++ code would execute here and generate events
        // For demonstration, we'll just work with the empty event list

        // Get statistics
        std::cout << "Getting statistics...\n";
        auto stats = tracer.get_statistics();
        std::cout << "  Total events: " << stats.total_events << "\n";
        std::cout << "  Function calls: " << stats.function_calls << "\n";
        std::cout << "  Errors: " << stats.errors << "\n\n";

        // Get events as JSON
        std::cout << "Getting events...\n";
        std::string events = tracer.get_events();
        std::cout << "Events JSON: " << events << "\n\n";

        // Clear events
        std::cout << "Clearing events...\n";
        tracer.clear_events();

        // Disable tracing
        std::cout << "Disabling tracing...\n";
        tracer.disable();

        std::cout << "\nExample completed successfully!\n";
        return 0;

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
