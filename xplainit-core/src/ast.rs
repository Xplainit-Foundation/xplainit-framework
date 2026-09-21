//! AST (Abstract Syntax Tree) parsing and source code mapping
//!
//! This module uses Tree-sitter to parse source code and map runtime events
//! to AST nodes, providing rich context for explanations.

use crate::{Language, Result, SourceLocation};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

/// Represents a parsed AST node
#[derive(Debug, Clone)]
pub struct AstNode {
    /// Node type (e.g., "function_definition", "if_statement")
    pub kind: String,

    /// Start position
    pub start: SourceLocation,

    /// End position
    pub end: SourceLocation,

    /// Source text
    pub text: String,

    /// Child nodes
    pub children: Vec<AstNode>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// AST parser for a specific language
pub struct AstParser {
    #[allow(dead_code)]
    language: Language,
    source_code: Option<String>,
    tree: Option<Tree>,
    parser: Parser,
}

impl AstParser {
    pub fn new(language: Language) -> Self {
        let mut parser = Parser::new();

        // Set language grammar - use the language() method from tree-sitter crates
        let lang_result = match language {
            Language::Python => tree_sitter_python::language(),
            Language::JavaScript => tree_sitter_javascript::language(),
            Language::Rust => tree_sitter_rust::language(),
            Language::C => tree_sitter_c::language(),
            Language::Cpp => tree_sitter_cpp::language(),
            _ => tree_sitter_python::language(), // Default to Python
        };

        parser
            .set_language(lang_result)
            .expect("Failed to set language");

        Self {
            language,
            source_code: None,
            tree: None,
            parser,
        }
    }

    /// Parse source code
    pub fn parse(&mut self, source: String) -> Result<()> {
        self.source_code = Some(source.clone());

        // Actual Tree-sitter parsing
        self.tree = self.parser.parse(&source, None);

        if self.tree.is_none() {
            return Err(crate::XplainitError::ParseError(
                "Failed to parse source code".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the root AST node
    pub fn root_node(&self) -> Option<AstNode> {
        let tree = self.tree.as_ref()?;
        let source = self.source_code.as_ref()?;
        Some(self.convert_node(tree.root_node(), source))
    }

    /// Convert tree-sitter Node to our AstNode
    fn convert_node(&self, node: Node, source: &str) -> AstNode {
        let start_point = node.start_position();
        let end_point = node.end_position();

        let start = SourceLocation {
            file: "".into(),
            line: start_point.row,
            column: start_point.column,
            offset: node.start_byte(),
        };

        let end = SourceLocation {
            file: "".into(),
            line: end_point.row,
            column: end_point.column,
            offset: node.end_byte(),
        };

        let text = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            children.push(self.convert_node(child, source));
        }

        let mut metadata = HashMap::new();

        // Extract function name if it's a function definition
        if node.kind().contains("function") || node.kind().contains("method") {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                    metadata.insert("name".to_string(), name.to_string());
                }
            }
        }

        AstNode {
            kind: node.kind().to_string(),
            start,
            end,
            text,
            children,
            metadata,
        }
    }

    /// Find AST node at a specific location
    pub fn find_node_at(&self, location: &SourceLocation) -> Option<AstNode> {
        let root = self.root_node()?;
        Self::find_node_recursive(&root, location)
    }

    fn find_node_recursive(node: &AstNode, location: &SourceLocation) -> Option<AstNode> {
        // Check if location is within this node
        if location.line >= node.start.line && location.line <= node.end.line {
            // Check children first (more specific)
            for child in &node.children {
                if let Some(found) = Self::find_node_recursive(child, location) {
                    return Some(found);
                }
            }
            // Return this node if no child matches
            return Some(node.clone());
        }
        None
    }

    /// Get surrounding context for a location
    pub fn get_context(
        &self,
        location: &SourceLocation,
        lines_before: usize,
        lines_after: usize,
    ) -> Option<String> {
        let source = self.source_code.as_ref()?;
        let lines: Vec<&str> = source.lines().collect();

        let start = location.line.saturating_sub(lines_before);
        let end = (location.line + lines_after).min(lines.len());

        Some(lines[start..end].join("\n"))
    }

    /// Get function name containing a location
    ///
    /// Walks the retained tree-sitter `Tree` directly: descends to the smallest
    /// node containing `location`, then walks upward via `node.parent()` until a
    /// function-like node for the current grammar is found, and reads its name.
    /// Returns `None` when the location is not inside any function/method.
    pub fn get_containing_function(&self, location: &SourceLocation) -> Option<String> {
        let tree = self.tree.as_ref()?;
        let source = self.source_code.as_ref()?;

        // Descend to the smallest named-or-unnamed node containing the location.
        let root = tree.root_node();
        let mut node = Self::smallest_node_at(root, location)?;

        // Walk up until we hit a named function-like node. Anonymous functions
        // (e.g. JS arrow functions / function expressions with no name) are
        // skipped so we resolve to the nearest *named* enclosing function.
        loop {
            if Self::is_function_node(&node) {
                if let Some(name) = Self::function_name(&node, source) {
                    return Some(name);
                }
            }
            match node.parent() {
                Some(parent) => node = parent,
                None => return None,
            }
        }
    }

    /// Find the smallest tree-sitter node whose byte/line range contains the
    /// given location.
    fn smallest_node_at<'a>(node: Node<'a>, location: &SourceLocation) -> Option<Node<'a>> {
        // The location must fall within this node's line range.
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        if location.line < start_line || location.line > end_line {
            return None;
        }

        // Prefer the most specific matching child.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = Self::smallest_node_at(child, location) {
                return Some(found);
            }
        }

        Some(node)
    }

    /// Whether a tree-sitter node represents a function/method definition for one
    /// of the supported grammars.
    fn is_function_node(node: &Node) -> bool {
        matches!(
            node.kind(),
            // Python
            "function_definition"
            // JavaScript / TypeScript
            | "function_declaration"
            | "function"
            | "function_expression"
            | "method_definition"
            | "generator_function"
            | "generator_function_declaration"
            | "arrow_function"
            // Rust
            | "function_item" // C / C++ share "function_definition"
        )
    }

    /// Extract the name of a function-like node. Handles the field-based grammars
    /// (Python/JS/Rust) directly and descends declarators for C/C++.
    fn function_name(node: &Node, source: &str) -> Option<String> {
        // Most grammars expose a "name" field directly.
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                return Some(name.to_string());
            }
        }

        // C/C++: the function name is nested inside the "declarator" field,
        // e.g. function_definition -> function_declarator -> identifier.
        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(name) = Self::name_from_declarator(&declarator, source) {
                return Some(name);
            }
        }

        None
    }

    /// Recursively descend a C/C++ declarator to find the function identifier.
    fn name_from_declarator(node: &Node, source: &str) -> Option<String> {
        match node.kind() {
            "identifier" | "field_identifier" | "type_identifier" => node
                .utf8_text(source.as_bytes())
                .ok()
                .map(|s| s.to_string()),
            _ => {
                // function_declarator / pointer_declarator / parenthesized_declarator
                // all nest the real declarator in the "declarator" field.
                if let Some(inner) = node.child_by_field_name("declarator") {
                    return Self::name_from_declarator(&inner, source);
                }
                // Fall back to scanning children for a nested declarator/identifier.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if let Some(name) = Self::name_from_declarator(&child, source) {
                        return Some(name);
                    }
                }
                None
            }
        }
    }
}

/// AST cache for multiple files
pub struct AstCache {
    parsers: HashMap<String, AstParser>,
    language: Language,
}

impl AstCache {
    pub fn new(language: Language) -> Self {
        Self {
            parsers: HashMap::new(),
            language,
        }
    }

    /// Get or create parser for a file
    pub fn get_parser(
        &mut self,
        file_path: &str,
        source: Option<String>,
    ) -> Result<&mut AstParser> {
        if !self.parsers.contains_key(file_path) {
            let mut parser = AstParser::new(self.language);
            if let Some(src) = source {
                parser.parse(src)?;
            }
            self.parsers.insert(file_path.to_string(), parser);
        }

        Ok(self.parsers.get_mut(file_path).unwrap())
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.parsers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_parser_creation() {
        let parser = AstParser::new(Language::Python);
        assert!(parser.source_code.is_none());
        assert!(parser.root_node().is_none());
    }

    #[test]
    fn test_ast_parser_parse() {
        let mut parser = AstParser::new(Language::Python);
        let source = "def hello():\n    print('world')".to_string();

        parser.parse(source.clone()).unwrap();
        assert!(parser.source_code.is_some());
        assert!(parser.root_node().is_some());
    }

    #[test]
    fn test_ast_cache() {
        let mut cache = AstCache::new(Language::Python);

        let source = "x = 1".to_string();
        let parser = cache.get_parser("test.py", Some(source)).unwrap();

        assert!(parser.source_code.is_some());
    }

    #[test]
    fn test_get_context() {
        let mut parser = AstParser::new(Language::Python);
        let source = "line1\nline2\nline3\nline4\nline5".to_string();
        parser.parse(source).unwrap();

        let location = SourceLocation {
            file: "test.py".into(),
            line: 2,
            column: 0,
            offset: 0,
        };

        let context = parser.get_context(&location, 1, 1);
        assert!(context.is_some());
    }

    fn loc(line: usize) -> SourceLocation {
        SourceLocation {
            file: "test".into(),
            // Tree-sitter uses 0-based rows.
            line,
            column: 0,
            offset: 0,
        }
    }

    #[test]
    fn test_get_containing_function_python() {
        let mut parser = AstParser::new(Language::Python);
        // Line 0: `def foo():`
        // Line 1: `    x = 1`
        parser.parse("def foo():\n    x = 1".to_string()).unwrap();

        assert_eq!(
            parser.get_containing_function(&loc(1)),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_get_containing_function_javascript() {
        let mut parser = AstParser::new(Language::JavaScript);
        // Line 0: `function bar() {`
        // Line 1: `  var y = 2;`
        // Line 2: `}`
        parser
            .parse("function bar() {\n  var y = 2;\n}".to_string())
            .unwrap();

        assert_eq!(
            parser.get_containing_function(&loc(1)),
            Some("bar".to_string())
        );
    }

    #[test]
    fn test_get_containing_function_javascript_method() {
        let mut parser = AstParser::new(Language::JavaScript);
        // A method inside a class should resolve to the method name.
        let source = "class C {\n  greet() {\n    return 1;\n  }\n}".to_string();
        parser.parse(source).unwrap();

        assert_eq!(
            parser.get_containing_function(&loc(2)),
            Some("greet".to_string())
        );
    }

    #[test]
    fn test_get_containing_function_rust() {
        let mut parser = AstParser::new(Language::Rust);
        // Line 0: `fn baz() {`
        // Line 1: `    let z = 3;`
        // Line 2: `}`
        parser
            .parse("fn baz() {\n    let z = 3;\n}".to_string())
            .unwrap();

        assert_eq!(
            parser.get_containing_function(&loc(1)),
            Some("baz".to_string())
        );
    }

    #[test]
    fn test_get_containing_function_c() {
        let mut parser = AstParser::new(Language::C);
        // Line 0: `int add(int a, int b) {`
        // Line 1: `    return a + b;`
        // Line 2: `}`
        parser
            .parse("int add(int a, int b) {\n    return a + b;\n}".to_string())
            .unwrap();

        assert_eq!(
            parser.get_containing_function(&loc(1)),
            Some("add".to_string())
        );
    }

    #[test]
    fn test_get_containing_function_outside_returns_none() {
        let mut parser = AstParser::new(Language::Python);
        // Line 0: `x = 1` (module-level, not inside any function)
        // Line 1: `def foo():`
        // Line 2: `    y = 2`
        parser
            .parse("x = 1\ndef foo():\n    y = 2".to_string())
            .unwrap();

        assert_eq!(parser.get_containing_function(&loc(0)), None);
    }
}
