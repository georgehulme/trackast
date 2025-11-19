use tree_sitter::Parser;
use trackast_lib::ast::{AbstractAST, FunctionDef, Signature, FunctionCall};

/// Translator for Rust source code to abstract AST
pub struct RustTranslator;

impl RustTranslator {
    /// Create a new Rust translator
    #[must_use] 
    pub fn new() -> Self {
        RustTranslator
    }

    /// Set up a parser for Rust
    ///
    /// # Errors
    ///
    /// Returns an error if the parser cannot be initialized or language set.
    pub fn setup_parser() -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::language();
        parser
            .set_language(language)
            .map_err(|_| "Failed to set Rust language".to_string())?;
        Ok(parser)
    }

    /// Parse source code and return the tree
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse_source(&self, source: &str) -> Result<tree_sitter::Tree, String> {
        let mut parser = Self::setup_parser()?;
        parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse source".to_string())
    }

    /// Query for function names in the tree
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or querying fails.
    pub fn query_function_names(&self, source: &str) -> Result<Vec<String>, String> {
        let tree = self.parse_source(source)?;
        let root = tree.root_node();
        let mut functions = Vec::new();

        Self::extract_functions_recursive(root, source, &mut functions);
        Ok(functions)
    }

    /// Recursively extract function names from AST
    fn extract_functions_recursive(
        node: tree_sitter::Node,
        source: &str,
        functions: &mut Vec<String>,
    ) {
        if node.kind() == "function_item" {
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    let name = &source[child.start_byte()..child.end_byte()];
                    functions.push(name.to_string());
                    break;
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            Self::extract_functions_recursive(child, source, functions);
        }
    }

    /// Extract module path from Rust source (simple version)
    ///
    /// # Errors
    ///
    /// This function currently always succeeds, but returns Result for consistency.
    pub fn extract_module_path(&self, source: &str, default_path: &str) -> Result<String, String> {
        let mut modules = Vec::new();
        
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("mod ") && trimmed.ends_with(';') {
                let mod_name = trimmed[4..trimmed.len() - 1].trim();
                modules.push(mod_name.to_string());
            } else if trimmed.starts_with("pub mod ") && trimmed.ends_with(';') {
                let mod_name = trimmed[8..trimmed.len() - 1].trim();
                modules.push(mod_name.to_string());
            }
        }

        if modules.is_empty() {
            Ok(default_path.to_string())
        } else {
            Ok(format!("{}::{}", default_path, modules.join("::")))
        }
    }

    /// Extract function calls from source code
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn extract_function_calls(&self, source: &str) -> Result<Vec<String>, String> {
        let tree = self.parse_source(source)?;
        let root = tree.root_node();
        let mut calls = Vec::new();

        Self::extract_calls_recursive(root, source, &mut calls);
        Ok(calls)
    }

    /// Recursively find function calls in the tree
    fn extract_calls_recursive(
        node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
    ) {
        if node.kind() == "call_expression" {
            if let Some(child) = node.child(0) {
                let call_name = Self::extract_identifier_or_field_access(child, source);
                if let Some(name) = call_name {
                    calls.push(name);
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            Self::extract_calls_recursive(child, source, calls);
        }
    }

    /// Extract identifier from a node (handles simple identifiers and field access)
    fn extract_identifier_or_field_access(
        node: tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        match node.kind() {
            "identifier" => {
                let text = &source[node.start_byte()..node.end_byte()];
                Some(text.to_string())
            }
            "field_expression" => {
                if let Some(child) = node.child(node.child_count() - 1) {
                    if child.kind() == "field" {
                        let text = &source[child.start_byte()..child.end_byte()];
                        return Some(text.to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Translate Rust source to abstract AST
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn translate(&self, source: &str, module_path: &str) -> Result<AbstractAST, String> {
        let tree = self.parse_source(source)?;
        let root = tree.root_node();
        let mut ast = AbstractAST::new(module_path.to_string());

        // Extract all functions and their calls
        Self::extract_ast_recursive(root, source, module_path, &mut ast, "");

        Ok(ast)
    }

    /// Recursively extract functions and build AST
    fn extract_ast_recursive(
        node: tree_sitter::Node,
        source: &str,
        module: &str,
        ast: &mut AbstractAST,
        impl_context: &str,
    ) {
        if node.kind() == "impl_item" {
            // Extract the type being implemented for
            let mut impl_type = String::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "type_identifier" || child.kind() == "identifier" {
                    impl_type = source[child.start_byte()..child.end_byte()].to_string();
                    break;
                }
            }

            // Recursively process children with impl context
            for child in node.children(&mut node.walk()) {
                Self::extract_ast_recursive(child, source, module, ast, &impl_type);
            }
            return;
        }

        if node.kind() == "function_item" {
            // Extract function name
            let mut func_name = String::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    func_name = source[child.start_byte()..child.end_byte()].to_string();
                    break;
                }
            }

            if !func_name.is_empty() {
                // Extract calls from this function
                let mut calls = Vec::new();
                Self::extract_calls_from_function(node, source, &mut calls);

                // Create function definition with impl context
                let sig = Signature::empty(); // Simplified for now
                let scoped_name = if impl_context.is_empty() {
                    func_name
                } else {
                    format!("{}::{}", impl_context, func_name)
                };
                let mut func_def = FunctionDef::new(scoped_name, sig, module.to_string());
                
                for call_name in calls {
                    let call = FunctionCall::new(call_name, None, 0);
                    func_def.add_call(call);
                }

                ast.add_function(func_def);
            }
        }

        for child in node.children(&mut node.walk()) {
            Self::extract_ast_recursive(child, source, module, ast, impl_context);
        }
    }

    /// Extract calls within a single function
    fn extract_calls_from_function(
        func_node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
    ) {
        for child in func_node.children(&mut func_node.walk()) {
            Self::extract_calls_recursive(child, source, calls);
        }
    }

    /// Translate a Rust file to abstract AST
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails.
    pub fn translate_file(&self, path: &str, module_path: &str) -> Result<AbstractAST, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        self.translate(&source, module_path)
    }
}

impl crate::translator_trait::Translator for RustTranslator {
    fn translate_file(&self, path: &str, module_path: Option<&str>) -> Result<AbstractAST, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let module = if let Some(m) = module_path {
            m.to_string()
        } else {
            std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("root")
                .to_string()
        };
        self.translate(&source, &module)
    }
}

impl Default for RustTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_translator_new() {
        let translator = RustTranslator::new();
        assert_eq!(std::mem::size_of_val(&translator), 0);
    }

    #[test]
    fn test_setup_parser() {
        let result = RustTranslator::setup_parser();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_rust() {
        let translator = RustTranslator::new();
        let source = "fn main() {}";
        let result = translator.parse_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_function_names() {
        let translator = RustTranslator::new();
        let source = "fn main() {}\nfn helper() {}";
        let names = translator.query_function_names(source).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"helper".to_string()));
    }

    #[test]
    fn test_translate_simple() {
        let translator = RustTranslator::new();
        let source = "fn main() {}\nfn helper() {}";
        let ast = translator.translate(source, "root").unwrap();
        assert_eq!(ast.module_path(), "root");
        assert!(ast.functions.len() >= 2);
    }

    #[test]
    fn test_translate_with_calls() {
        let translator = RustTranslator::new();
        let source = "fn main() { helper(); }\nfn helper() {}";
        let ast = translator.translate(source, "root").unwrap();
        let main_func = ast.get_function("main");
        assert!(main_func.is_some());
    }

    #[test]
    fn test_extract_module_path_empty() {
        let translator = RustTranslator::new();
        let source = "fn main() {}";
        let path = translator.extract_module_path(source, "root").unwrap();
        assert_eq!(path, "root");
    }

    #[test]
    fn test_extract_module_path_with_mod() {
        let translator = RustTranslator::new();
        let source = "mod helpers;\nfn main() {}";
        let path = translator.extract_module_path(source, "root").unwrap();
        assert_eq!(path, "root::helpers");
    }

    #[test]
    fn test_extract_function_calls_empty() {
        let translator = RustTranslator::new();
        let source = "fn main() {}";
        let calls = translator.extract_function_calls(source).unwrap();
        assert_eq!(calls.len(), 0);
    }
}
