"""
Production Example: Data Processing Pipeline

This example demonstrates using Xplainit to trace a data processing pipeline,
showing how to debug ETL (Extract, Transform, Load) workflows.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import xplainit
from python.tracer import XplainitTracer
from python.decorators import trace, trace_recursive
import json
import time


# Initialize Xplainit
backend = xplainit.Xplainit(enabled=True, verbosity="detailed")


# Data extraction layer
@trace(backend=backend)
def extract_from_csv(filename):
    """Extract data from CSV file (simulated)"""
    time.sleep(0.05)
    # Simulated data
    return [
        {'id': 1, 'name': 'Product A', 'price': '100.50', 'category': 'electronics'},
        {'id': 2, 'name': 'Product B', 'price': 'invalid', 'category': 'books'},
        {'id': 3, 'name': 'Product C', 'price': '45.99', 'category': 'electronics'},
        {'id': 4, 'name': 'Product D', 'price': '12.00', 'category': 'clothing'},
    ]


@trace(backend=backend)
def extract_from_api(endpoint):
    """Extract data from external API (simulated)"""
    time.sleep(0.1)
    # Simulated API response
    return {
        'categories': {
            'electronics': {'tax_rate': 0.15, 'shipping': 5.00},
            'books': {'tax_rate': 0.08, 'shipping': 3.00},
            'clothing': {'tax_rate': 0.10, 'shipping': 4.00},
        }
    }


# Data transformation layer
def parse_price(price_string):
    """Parse price string to float"""
    try:
        return float(price_string)
    except (ValueError, TypeError):
        return None


@trace(backend=backend)
def transform_product(product, category_data):
    """Transform a single product record"""
    # Parse price
    price = parse_price(product['price'])
    if price is None:
        raise ValueError(f"Invalid price for product {product['id']}")
    
    # Get category info
    category = product['category']
    category_info = category_data['categories'].get(category, {})
    
    # Calculate totals
    tax = price * category_info.get('tax_rate', 0)
    shipping = category_info.get('shipping', 0)
    total = price + tax + shipping
    
    return {
        'id': product['id'],
        'name': product['name'],
        'base_price': price,
        'tax': round(tax, 2),
        'shipping': round(shipping, 2),
        'total': round(total, 2),
        'category': category
    }


@trace(backend=backend)
def transform_batch(products, category_data):
    """Transform a batch of products"""
    transformed = []
    errors = []
    
    for product in products:
        try:
            result = transform_product(product, category_data)
            transformed.append(result)
        except Exception as e:
            errors.append({
                'product_id': product.get('id'),
                'error': str(e)
            })
    
    return transformed, errors


# Data loading layer
@trace(backend=backend)
def load_to_database(records):
    """Load transformed records to database (simulated)"""
    time.sleep(0.03)
    print(f"  [DB] Inserted {len(records)} records")
    return len(records)


# Orchestration
@trace(backend=backend)
def run_etl_pipeline():
    """Main ETL pipeline orchestrator"""
    print("Starting ETL pipeline...")
    
    # Extract
    print("  [1/3] Extracting data...")
    products = extract_from_csv('products.csv')
    category_data = extract_from_api('/categories')
    print(f"  Extracted {len(products)} products")
    
    # Transform
    print("  [2/3] Transforming data...")
    transformed, errors = transform_batch(products, category_data)
    print(f"  Transformed {len(transformed)} products")
    if errors:
        print(f"  Warning: {len(errors)} errors occurred")
        for error in errors:
            print(f"    - Product {error['product_id']}: {error['error']}")
    
    # Load
    print("  [3/3] Loading data...")
    loaded_count = load_to_database(transformed)
    print(f"  Loaded {loaded_count} records")
    
    return {
        'extracted': len(products),
        'transformed': len(transformed),
        'loaded': loaded_count,
        'errors': len(errors)
    }


def main():
    """Run the production example"""
    print("=" * 70)
    print("PRODUCTION EXAMPLE: Data Processing Pipeline")
    print("=" * 70)
    print()
    
    # Run ETL with automatic tracing
    tracer = XplainitTracer(backend)
    
    with tracer:
        result = run_etl_pipeline()
    
    print()
    print("Pipeline complete!")
    print(f"Summary: {result}")
    print()
    
    # Analyze the trace
    print("=" * 70)
    print("TRACE ANALYSIS")
    print("=" * 70)
    print()
    
    # Get events
    events = json.loads(backend.get_events())
    
    # Find exceptions
    exceptions = [e for e in events if 'Exception' in e]
    if exceptions:
        print(f"Exceptions caught: {len(exceptions)}")
        for exc in exceptions:
            exc_data = exc['Exception']
            print(f"  - {exc_data['error_type']}: {exc_data['message']}")
            print(f"    at {exc_data['location']['file']}:{exc_data['location']['line']}")
        print()
    
    # Show function call flow
    print("Function call flow:")
    depth = 0
    for event in events[:20]:  # Show first 20 events
        if 'FunctionEnter' in event:
            name = event['FunctionEnter']['name']
            args = event['FunctionEnter'].get('args', {})
            print("  " * depth + f"→ {name}({len(args)} args)")
            depth += 1
        elif 'FunctionExit' in event:
            depth = max(0, depth - 1)
    print()
    
    # Get explanation
    explanation = backend.get_last_explanation()
    print("Natural language explanation:")
    print(f"  {explanation}")
    print()
    
    print("=" * 70)
    print("DEBUGGING INSIGHTS")
    print("=" * 70)
    print()
    print("The trace reveals:")
    print("  ✅ Product B has invalid price data")
    print("  ✅ Pipeline continues despite errors (error handling works)")
    print("  ✅ Complete data flow from extract → transform → load")
    print("  ✅ Timing information for each stage")
    print()
    print("Use this trace to:")
    print("  • Debug data quality issues")
    print("  • Identify bottlenecks in ETL process")
    print("  • Verify error handling logic")
    print("  • Understand data transformations")
    print("  • Optimize pipeline performance")
    print()


if __name__ == '__main__':
    main()
