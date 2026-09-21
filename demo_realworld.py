#!/usr/bin/env python3
"""
Real-World Example: Web API Request Handler

Demonstrates how Xplainit can be used in a production-like scenario
to trace, debug, and document API request handling.
"""

import sys
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

from xplainit import Xplainit
from decorators import trace, profile, trace_class
import json
from datetime import datetime
from typing import Dict, Any, Optional

print("=" * 80)
print("REAL-WORLD EXAMPLE: API Request Handler")
print("=" * 80)

# Create backend
backend = Xplainit(enabled=True, verbosity="detailed")

# =============================================================================
# Simulated Database
# =============================================================================

DATABASE = {
    'users': {
        1: {'id': 1, 'name': 'Alice', 'email': 'alice@example.com', 'age': 30},
        2: {'id': 2, 'name': 'Bob', 'email': 'bob@example.com', 'age': 25},
        3: {'id': 3, 'name': 'Charlie', 'email': 'charlie@example.com', 'age': 35},
    },
    'posts': {
        1: {'id': 1, 'user_id': 1, 'title': 'Hello World', 'content': 'My first post!'},
        2: {'id': 2, 'user_id': 1, 'title': 'Python Tips', 'content': 'Some Python tips'},
        3: {'id': 3, 'user_id': 2, 'title': 'Rust is Great', 'content': 'Why I love Rust'},
    }
}

# =============================================================================
# Data Access Layer
# =============================================================================

@trace_class(backend=backend)
class UserRepository:
    """Repository for user data access."""
    
    @profile(backend=backend, name="find_user_by_id")
    def find_by_id(self, user_id: int) -> Optional[Dict[str, Any]]:
        """Find user by ID."""
        return DATABASE['users'].get(user_id)
    
    @profile(backend=backend, name="find_all_users")
    def find_all(self) -> list:
        """Get all users."""
        return list(DATABASE['users'].values())
    
    @profile(backend=backend, name="create_user")
    def create(self, name: str, email: str, age: int) -> Dict[str, Any]:
        """Create a new user."""
        user_id = max(DATABASE['users'].keys()) + 1
        user = {'id': user_id, 'name': name, 'email': email, 'age': age}
        DATABASE['users'][user_id] = user
        return user

@trace_class(backend=backend)
class PostRepository:
    """Repository for post data access."""
    
    @profile(backend=backend, name="find_posts_by_user")
    def find_by_user_id(self, user_id: int) -> list:
        """Find all posts by user ID."""
        return [p for p in DATABASE['posts'].values() if p['user_id'] == user_id]
    
    @profile(backend=backend, name="create_post")
    def create(self, user_id: int, title: str, content: str) -> Dict[str, Any]:
        """Create a new post."""
        post_id = max(DATABASE['posts'].keys()) + 1
        post = {'id': post_id, 'user_id': user_id, 'title': title, 'content': content}
        DATABASE['posts'][post_id] = post
        return post

# =============================================================================
# Business Logic Layer
# =============================================================================

@trace_class(backend=backend)
class UserService:
    """Service for user-related business logic."""
    
    def __init__(self):
        self.user_repo = UserRepository()
        self.post_repo = PostRepository()
    
    @profile(backend=backend, name="get_user_profile")
    def get_user_profile(self, user_id: int) -> Dict[str, Any]:
        """Get user profile with posts."""
        # Get user
        user = self.user_repo.find_by_id(user_id)
        if not user:
            raise ValueError(f"User not found: {user_id}")
        
        # Get user's posts
        posts = self.post_repo.find_by_user_id(user_id)
        
        # Build profile
        profile = {
            'user': user,
            'posts': posts,
            'post_count': len(posts),
            'retrieved_at': datetime.now().isoformat()
        }
        
        return profile
    
    @profile(backend=backend, name="create_user_with_post")
    def create_user_with_post(self, name: str, email: str, age: int, 
                               first_post_title: str, first_post_content: str) -> Dict[str, Any]:
        """Create a new user with their first post."""
        # Create user
        user = self.user_repo.create(name, email, age)
        
        # Create first post
        post = self.post_repo.create(user['id'], first_post_title, first_post_content)
        
        return {
            'user': user,
            'first_post': post,
            'created_at': datetime.now().isoformat()
        }

# =============================================================================
# API Layer
# =============================================================================

@trace_class(backend=backend)
class APIHandler:
    """Handler for API requests."""
    
    def __init__(self):
        self.user_service = UserService()
    
    @trace(backend=backend)
    def handle_get_user_profile(self, user_id: int) -> Dict[str, Any]:
        """Handle GET /users/{id}/profile request."""
        try:
            profile = self.user_service.get_user_profile(user_id)
            return {
                'status': 'success',
                'data': profile,
                'error': None
            }
        except ValueError as e:
            return {
                'status': 'error',
                'data': None,
                'error': str(e)
            }
        except Exception as e:
            return {
                'status': 'error',
                'data': None,
                'error': f'Internal server error: {str(e)}'
            }
    
    @trace(backend=backend)
    def handle_create_user(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Handle POST /users request."""
        try:
            # Validate required fields
            required_fields = ['name', 'email', 'age', 'first_post_title', 'first_post_content']
            for field in required_fields:
                if field not in request_data:
                    raise ValueError(f"Missing required field: {field}")
            
            # Create user with post
            result = self.user_service.create_user_with_post(
                request_data['name'],
                request_data['email'],
                request_data['age'],
                request_data['first_post_title'],
                request_data['first_post_content']
            )
            
            return {
                'status': 'success',
                'data': result,
                'error': None
            }
        except ValueError as e:
            return {
                'status': 'error',
                'data': None,
                'error': str(e)
            }
        except Exception as e:
            return {
                'status': 'error',
                'data': None,
                'error': f'Internal server error: {str(e)}'
            }

# =============================================================================
# Example Usage
# =============================================================================

# Initialize API handler
api = APIHandler()

print("\n--- Example 1: Get User Profile ---")
response1 = api.handle_get_user_profile(1)
print(f"Response: {json.dumps(response1, indent=2, default=str)}")

# Get execution trace
print("\n--- Execution Trace (simplified) ---")
backend.set_verbosity("brief")
print(backend.get_explanations())

# Clear for next request
backend.clear()

print("\n" + "=" * 80)
print("--- Example 2: Create New User ---")

new_user_request = {
    'name': 'Diana',
    'email': 'diana@example.com',
    'age': 28,
    'first_post_title': 'My First Day',
    'first_post_content': 'Hello everyone! This is my first post.'
}

response2 = api.handle_create_user(new_user_request)
print(f"Response: {json.dumps(response2, indent=2, default=str)}")

# Get detailed execution trace
print("\n--- Detailed Execution Trace ---")
backend.set_verbosity("detailed")
explanations = backend.get_explanations()
# Show first 1000 characters
print(explanations[:1000] + "...")

# Get statistics
events = json.loads(backend.get_events())
print(f"\n📊 Total events captured: {len(events)}")

# Count different event types
function_calls = sum(1 for e in events if 'FunctionEnter' in e)
function_returns = sum(1 for e in events if 'FunctionExit' in e)
print(f"   Function calls: {function_calls}")
print(f"   Function returns: {function_returns}")

# Clear for next request
backend.clear()

print("\n" + "=" * 80)
print("--- Example 3: Error Handling ---")

# Try to get non-existent user
response3 = api.handle_get_user_profile(999)
print(f"Response: {json.dumps(response3, indent=2)}")

# Show error in trace
print("\n--- Error Trace ---")
backend.set_verbosity("detailed")
print(backend.get_explanations())

# Clear for next request
backend.clear()

print("\n" + "=" * 80)
print("--- Example 4: Validation Error ---")

# Try to create user with missing fields
invalid_request = {
    'name': 'Eve'
    # Missing required fields!
}

response4 = api.handle_create_user(invalid_request)
print(f"Response: {json.dumps(response4, indent=2)}")

print("\n--- Validation Error Trace ---")
backend.set_verbosity("detailed")
print(backend.get_explanations())

# =============================================================================
# Summary
# =============================================================================

print("\n" + "=" * 80)
print("REAL-WORLD USAGE DEMONSTRATED")
print("=" * 80)
print()
print("✅ Traced API request handling")
print("✅ Traced business logic execution")
print("✅ Traced data access operations")
print("✅ Profiled method execution time")
print("✅ Captured error handling")
print("✅ Tracked validation errors")
print()
print("Use Cases:")
print("  • Debug API requests in production")
print("  • Profile performance bottlenecks")
print("  • Document API behavior")
print("  • Monitor business logic flow")
print("  • Track error scenarios")
print()
print("=" * 80)
