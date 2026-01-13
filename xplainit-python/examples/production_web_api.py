"""
Production Example: Web API Request Analyzer

This example demonstrates using Xplainit to analyze a Flask web API,
automatically tracing all request handlers and database queries.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import xplainit
from python.tracer import XplainitTracer
from python.decorators import trace, profile
import time
import json


# Simulated database
DATABASE = {
    'users': [
        {'id': 1, 'name': 'Alice', 'email': 'alice@example.com', 'age': 30},
        {'id': 2, 'name': 'Bob', 'email': 'bob@example.com', 'age': 25},
        {'id': 3, 'name': 'Charlie', 'email': 'charlie@example.com', 'age': 35},
    ],
    'orders': [
        {'id': 101, 'user_id': 1, 'product': 'Laptop', 'amount': 1200},
        {'id': 102, 'user_id': 1, 'product': 'Mouse', 'amount': 25},
        {'id': 103, 'user_id': 2, 'product': 'Keyboard', 'amount': 75},
    ]
}


# Initialize Xplainit
backend = xplainit.Xplainit(enabled=True, verbosity="normal")


# Database layer with decorators for selective tracing
@trace(backend=backend)
def db_query_users():
    """Simulate database query for users"""
    time.sleep(0.01)  # Simulate query time
    return DATABASE['users']


@trace(backend=backend)
def db_query_user_by_id(user_id):
    """Simulate database query for specific user"""
    time.sleep(0.01)
    return next((u for u in DATABASE['users'] if u['id'] == user_id), None)


@profile(backend=backend)
def db_query_orders_by_user(user_id):
    """Simulate database query for user's orders"""
    time.sleep(0.02)  # Slower query
    return [o for o in DATABASE['orders'] if o['user_id'] == user_id]


# Business logic layer
def calculate_user_total_spent(user_id):
    """Calculate total amount spent by user"""
    orders = db_query_orders_by_user(user_id)
    return sum(order['amount'] for order in orders)


def get_user_profile(user_id):
    """Get complete user profile with spending"""
    user = db_query_user_by_id(user_id)
    if not user:
        return None
    
    total_spent = calculate_user_total_spent(user_id)
    
    return {
        **user,
        'total_spent': total_spent,
        'orders_count': len(db_query_orders_by_user(user_id))
    }


# API handlers (simulated)
def handle_get_users():
    """GET /api/users endpoint"""
    users = db_query_users()
    return {'users': users, 'count': len(users)}


def handle_get_user(user_id):
    """GET /api/users/:id endpoint"""
    profile = get_user_profile(user_id)
    if not profile:
        return {'error': 'User not found'}, 404
    return {'user': profile}, 200


# Production scenario: Process multiple API requests
def simulate_api_traffic():
    """Simulate a series of API requests"""
    print("=" * 70)
    print("PRODUCTION EXAMPLE: Web API Request Analyzer")
    print("=" * 70)
    print()
    
    # Create tracer with automatic tracing
    tracer = XplainitTracer(backend, trace_lines=False, capture_locals=False)
    
    print("Starting API traffic simulation...")
    print()
    
    with tracer:
        # Request 1: List all users
        print("Request 1: GET /api/users")
        result1 = handle_get_users()
        print(f"  Response: {result1['count']} users")
        print()
        
        # Request 2: Get user profile
        print("Request 2: GET /api/users/1")
        result2, status = handle_get_user(1)
        print(f"  Response: {result2['user']['name']}, spent ${result2['user']['total_spent']}")
        print()
        
        # Request 3: Get another user
        print("Request 3: GET /api/users/2")
        result3, status = handle_get_user(2)
        print(f"  Response: {result3['user']['name']}, spent ${result3['user']['total_spent']}")
        print()
        
        # Request 4: Non-existent user
        print("Request 4: GET /api/users/999")
        result4, status = handle_get_user(999)
        print(f"  Response: {status} - {result4.get('error', 'OK')}")
        print()
    
    print("API traffic simulation complete!")
    print()
    
    # Analyze the captured trace
    print("=" * 70)
    print("TRACE ANALYSIS")
    print("=" * 70)
    print()
    
    # Get statistics
    stats = backend.get_stats()
    print(f"Total events captured: {stats}")
    print()
    
    # Get events
    events = json.loads(backend.get_events())
    print(f"Event breakdown:")
    
    # Count event types
    function_enters = sum(1 for e in events if 'FunctionEnter' in e)
    function_exits = sum(1 for e in events if 'FunctionExit' in e)
    exceptions = sum(1 for e in events if 'Exception' in e)
    
    print(f"  - Function calls: {function_enters}")
    print(f"  - Function returns: {function_exits}")
    print(f"  - Exceptions: {exceptions}")
    print()
    
    # Show function call frequency
    function_names = {}
    for event in events:
        if 'FunctionEnter' in event:
            name = event['FunctionEnter']['name']
            function_names[name] = function_names.get(name, 0) + 1
    
    print("Most called functions:")
    for name, count in sorted(function_names.items(), key=lambda x: x[1], reverse=True)[:5]:
        print(f"  - {name}: {count} calls")
    print()
    
    # Get explanation
    explanation = backend.get_last_explanation()
    print("Natural language explanation:")
    print(f"  {explanation}")
    print()
    
    print("=" * 70)
    print("INSIGHTS")
    print("=" * 70)
    print()
    print("✅ Automatic tracing captured all API request handling")
    print("✅ Database queries tracked with @trace decorator")
    print("✅ Performance profiling on slow queries with @profile")
    print("✅ Complete call graph available for debugging")
    print("✅ Zero instrumentation in business logic!")
    print()
    print("This trace can be used to:")
    print("  • Identify slow database queries")
    print("  • Detect N+1 query problems")
    print("  • Understand request flow")
    print("  • Debug production issues")
    print("  • Optimize performance bottlenecks")
    print()


if __name__ == '__main__':
    simulate_api_traffic()
