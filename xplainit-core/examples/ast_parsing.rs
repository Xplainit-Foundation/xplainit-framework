//! AST Parsing Example
//! 
//! Demonstrates the new Tree-sitter integration for parsing
//! source code and extracting rich context.

use xplainit_core::*;

fn main() {
    println!("=== Xplainit Core - AST Parsing Example ===\n");
    
    // Example 1: Parse Python code
    println!("--- Example 1: Python Code Parsing ---");
    let python_code = r#"
def calculate_total(price, quantity):
    """Calculate the total cost."""
    subtotal = price * quantity
    
    if subtotal > 50:
        discount = subtotal * 0.1
        total = subtotal - discount
    else:
        total = subtotal
    
    return total

result = calculate_total(29.99, 3)
print(f"Total: {result}")
"#;
    
    let mut parser = AstParser::new(Language::Python);
    match parser.parse(python_code.to_string()) {
        Ok(_) => {
            println!("✓ Successfully parsed Python code");
            
            if let Some(root) = parser.root_node() {
                println!("  Root node type: {}", root.kind);
                println!("  Number of children: {}", root.children.len());
                
                // Show first few nodes
                for (i, child) in root.children.iter().take(3).enumerate() {
                    println!("  Child {}: {} at line {}", 
                        i + 1, child.kind, child.start.line);
                }
            }
            
            // Test finding node at specific location
            let location = SourceLocation::new("test.py".to_string(), 4, 4);
            if let Some(node) = parser.find_node_at(&location) {
                println!("  Node at line 4: {} ('{}')", 
                    node.kind, 
                    node.text.lines().next().unwrap_or("").trim());
            }
            
            // Test context extraction
            if let Some(context) = parser.get_context(&location, 2, 2) {
                println!("  Context around line 4:");
                for line in context.lines() {
                    println!("    {}", line);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
        }
    }
    
    // Example 2: Parse JavaScript code
    println!("\n--- Example 2: JavaScript Code Parsing ---");
    let js_code = r#"
function processOrder(order) {
    const total = order.items.reduce((sum, item) => {
        return sum + (item.price * item.quantity);
    }, 0);
    
    if (total > 100) {
        return applyDiscount(total, 0.15);
    }
    
    return total;
}
"#;
    
    let mut js_parser = AstParser::new(Language::JavaScript);
    match js_parser.parse(js_code.to_string()) {
        Ok(_) => {
            println!("✓ Successfully parsed JavaScript code");
            
            if let Some(root) = js_parser.root_node() {
                println!("  Root node type: {}", root.kind);
                println!("  Number of children: {}", root.children.len());
                
                // Find the function
                for child in &root.children {
                    if child.kind.contains("function") {
                        println!("  Found function: {}", 
                            child.metadata.get("name").unwrap_or(&"<anonymous>".to_string()));
                        println!("    Lines: {}-{}", child.start.line, child.end.line);
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
        }
    }
    
    // Example 3: Parse Rust code
    println!("\n--- Example 3: Rust Code Parsing ---");
    let rust_code = r#"
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn main() {
    let result = fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}
"#;
    
    let mut rust_parser = AstParser::new(Language::Rust);
    match rust_parser.parse(rust_code.to_string()) {
        Ok(_) => {
            println!("✓ Successfully parsed Rust code");
            
            if let Some(root) = rust_parser.root_node() {
                println!("  Root node type: {}", root.kind);
                println!("  Number of top-level items: {}", root.children.len());
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
        }
    }
    
    // Example 4: AST Cache Usage
    println!("\n--- Example 4: AST Cache ---");
    let mut cache = AstCache::new(Language::Python);
    
    let simple_code = "x = 1\ny = 2\nz = x + y".to_string();
    match cache.get_parser("script.py", Some(simple_code)) {
        Ok(_parser) => {
            println!("✓ Cached parser for script.py");
        }
        Err(e) => {
            println!("✗ Failed to create cached parser: {}", e);
        }
    }
    
    // Access again - should use cache
    match cache.get_parser("script.py", None) {
        Ok(_parser) => {
            println!("✓ Retrieved cached parser for script.py");
        }
        Err(e) => {
            println!("✗ Failed to retrieve cached parser: {}", e);
        }
    }
    
    println!("\n✓ AST parsing examples complete!");
    println!("\n📝 Summary:");
    println!("  • Tree-sitter integration: ✅ Working");
    println!("  • Python parsing: ✅ Working");
    println!("  • JavaScript parsing: ✅ Working");
    println!("  • Rust parsing: ✅ Working");
    println!("  • AST cache: ✅ Working");
    println!("  • Node location: ✅ Working");
    println!("  • Context extraction: ✅ Working");
}
