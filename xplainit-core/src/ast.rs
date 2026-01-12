//! AST (Abstract Syntax Tree) parsing and source code mapping
//!
//! This module uses Tree-sitter to parse source code and map runtime events
//! to AST nodes, providing rich context for explanations.

use crate::{Result, Language, SourceLocation};
use std::collections::HashMap;
use tree_sitter::{Parser, Tree, Node};

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
        
        parser.set_language(lang_result).expect("Failed to set language");
        
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
                "Failed to parse source code".to_string()
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
        
        let text = node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .to_string();
        
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
    pub fn get_context(&self, location: &SourceLocation, lines_before: usize, lines_after: usize) -> Option<String> {
        let source = self.source_code.as_ref()?;
        let lines: Vec<&str> = source.lines().collect();
        
        let start = location.line.saturating_sub(lines_before);
        let end = (location.line + lines_after).min(lines.len());
        
        Some(lines[start..end].join("\n"))
    }
    
    /// Get function name containing a location
    pub fn get_containing_function(&self, location: &SourceLocation) -> Option<String> {
        let node = self.find_node_at(location)?;
        
        // Walk up to find function definition
        self.find_function_in_hierarchy(&node)
    }
    
    fn find_function_in_hierarchy(&self, node: &AstNode) -> Option<String> {
        // Check if this node is a function
        if node.kind.contains("function") || node.kind.contains("method") {
            return node.metadata.get("name").cloned();
        }
        
        // For now, we don't track parents, so we search the tree
        // This is a simplified version - a production implementation would maintain parent links
        None
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
    pub fn get_parser(&mut self, file_path: &str, source: Option<String>) -> Result<&mut AstParser> {
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
}
