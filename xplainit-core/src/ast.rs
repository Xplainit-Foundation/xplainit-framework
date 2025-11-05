//! AST (Abstract Syntax Tree) parsing and source code mapping
//!
//! This module uses Tree-sitter to parse source code and map runtime events
//! to AST nodes, providing rich context for explanations.

use crate::{Result, Language, SourceLocation};
use std::collections::HashMap;

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
    _language: Language,
    source_code: Option<String>,
    root_node: Option<AstNode>,
}

impl AstParser {
    pub fn new(language: Language) -> Self {
        Self {
            _language: language,
            source_code: None,
            root_node: None,
        }
    }
    
    /// Parse source code
    pub fn parse(&mut self, source: String) -> Result<()> {
        self.source_code = Some(source.clone());
        
        // TODO: Actual Tree-sitter parsing
        // For now, create a stub root node
        self.root_node = Some(AstNode {
            kind: "module".into(),
            start: SourceLocation {
                file: "".into(),
                line: 0,
                column: 0,
                offset: 0,
            },
            end: SourceLocation {
                file: "".into(),
                line: 0,
                column: 0,
                offset: source.len(),
            },
            text: source.clone(),
            children: Vec::new(),
            metadata: HashMap::new(),
        });
        
        Ok(())
    }
    
    /// Find AST node at a specific location
    pub fn find_node_at(&self, location: &SourceLocation) -> Option<&AstNode> {
        self.root_node.as_ref().and_then(|root| {
            Self::find_node_recursive(root, location)
        })
    }
    
    fn find_node_recursive<'a>(node: &'a AstNode, location: &SourceLocation) -> Option<&'a AstNode> {
        // Check if location is within this node
        if location.line >= node.start.line && location.line <= node.end.line {
            // Check children first (more specific)
            for child in &node.children {
                if let Some(found) = Self::find_node_recursive(child, location) {
                    return Some(found);
                }
            }
            // Return this node if no child matches
            return Some(node);
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
        // TODO: Implement proper parent tracking
        if node.kind.contains("function") {
            node.metadata.get("name").cloned()
        } else {
            None
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
        assert!(parser.root_node.is_none());
    }

    #[test]
    fn test_ast_parser_parse() {
        let mut parser = AstParser::new(Language::Python);
        let source = "def hello():\n    print('world')".to_string();
        
        parser.parse(source.clone()).unwrap();
        assert!(parser.source_code.is_some());
        assert!(parser.root_node.is_some());
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
