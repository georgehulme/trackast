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

            // Extract function references passed to method calls like .to(), .service()
            // e.g., .route("/path", web::get().to(handler_func))
            if let Some(callee) = node.child(0) {
                if callee.kind() == "field_expression" {
                    // Get the method name
                    if let Some(field) = callee.child(callee.child_count() - 1) {
                        if field.kind() == "field" {
                            let method_name = &source[field.start_byte()..field.end_byte()];
                            // Check for common web framework methods
                            if matches!(method_name, "to" | "service" | "route" | "middleware" | "guard") {
                                // Extract identifier arguments
                                for i in 0..node.child_count() {
                                    if let Some(arg) = node.child(i) {
                                        if arg.kind() == "arguments" {
                                            for j in 0..arg.child_count() {
                                                if let Some(arg_child) = arg.child(j) {
                                                    if arg_child.kind() == "identifier" {
                                                        let name = &source[arg_child.start_byte()..arg_child.end_byte()];
                                                        calls.push(name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
        Self::extract_identifier_or_field_access_with_context(node, source, "")
    }

    /// Extract identifier from a node with context (handles simple identifiers and field access)
    fn extract_identifier_or_field_access_with_context(
        node: tree_sitter::Node,
        source: &str,
        impl_context: &str,
    ) -> Option<String> {
        match node.kind() {
            "identifier" => {
                let text = &source[node.start_byte()..node.end_byte()];
                Some(text.to_string())
            }
            "field_expression" => {
                // Check if this is a self.method() call
                if let Some(object) = node.child(0) {
                    let object_text = &source[object.start_byte()..object.end_byte()];
                    if object_text == "self" && !impl_context.is_empty() {
                        // This is self.method() - resolve to current impl context
                        if let Some(field) = node.child(2) { // field might be at index 2 (object, dot, field)
                            if field.kind() == "field_identifier" { // might be field_identifier not field
                                let method_name = &source[field.start_byte()..field.end_byte()];
                                return Some(format!("{}::{}", impl_context, method_name));
                            }
                        }
                    }
                }
                
                // Fallback: just extract the field name
                if let Some(child) = node.child(node.child_count() - 1) {
                    if child.kind() == "field_identifier" {
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
                // Extract calls from this function with impl context for resolution
                let mut calls = Vec::new();
                Self::extract_calls_from_function_with_context(node, source, &mut calls, impl_context);

                // Create function definition with impl context
                let sig = Signature::empty(); // Simplified for now
                let scoped_name = if impl_context.is_empty() {
                    func_name
                } else {
                    format!("{}::{}", impl_context, func_name)
                };
                let mut func_def = FunctionDef::new(scoped_name, sig, module.to_string());
                
                for call_name in calls {
                    // Determine if this is a local call that should be resolved within the module
                    let target_module = if call_name.contains("::") {
                        // For method calls like "MyStruct::method2", try to resolve within current module
                        Some(module.to_string())
                    } else {
                        // For simple function calls, we can't determine easily, leave as None (external)
                        // This could be enhanced with more sophisticated analysis
                        None
                    };
                    let call = FunctionCall::new(call_name, target_module, 0);
                    func_def.add_call(call);
                }

                ast.add_function(func_def);
            }
        }

        if node.kind() == "expression_statement" && impl_context.is_empty() {
            // Handle top-level expression statements like router setup
            // e.g., App::new().route("/path", handler_func) or app.service(handler)
            let mut calls = Vec::new();
            Self::extract_calls_recursive(node, source, &mut calls);
            
            if !calls.is_empty() {
                // Create a virtual module-level function to track these references
                let sig = Signature::empty();
                let mut func_def = FunctionDef::new("<module>".to_string(), sig, module.to_string());
                
                for call_name in calls {
                    let call = FunctionCall::new(call_name, None, 0);
                    func_def.add_call(call);
                }
                
                // Check if we already have a module-level function
                if let Some(existing) = ast.functions.iter_mut().find(|f| f.name == "<module>") {
                    // Add calls to existing module function
                    for call in &func_def.calls {
                        existing.add_call(call.clone());
                    }
                } else {
                    ast.add_function(func_def);
                }
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

    /// Extract calls within a single function with impl context for better resolution
    fn extract_calls_from_function_with_context(
        func_node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
        impl_context: &str,
    ) {
        for child in func_node.children(&mut func_node.walk()) {
            Self::extract_calls_recursive_with_context(child, source, calls, impl_context);
        }
    }

    /// Recursively find function calls with impl context for better resolution
    fn extract_calls_recursive_with_context(
        node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
        impl_context: &str,
    ) {
        if node.kind() == "call_expression" {
            if let Some(child) = node.child(0) {
                let call_name = Self::extract_identifier_or_field_access_with_context(child, source, impl_context);
                if let Some(name) = call_name {
                    calls.push(name);
                }
            }

            // Extract function references passed to method calls like .to(), .service()
            // e.g., .route("/path", web::get().to(handler_func))
            if let Some(callee) = node.child(0) {
                if callee.kind() == "field_expression" {
                    // Get the method name
                    if let Some(field) = callee.child(callee.child_count() - 1) {
                        if field.kind() == "field" {
                            let method_name = &source[field.start_byte()..field.end_byte()];
                            // Check for common web framework methods
                            if matches!(method_name, "to" | "service" | "route" | "middleware" | "guard") {
                                // Extract identifier arguments
                                for i in 0..node.child_count() {
                                    if let Some(arg) = node.child(i) {
                                        if arg.kind() == "arguments" {
                                            for j in 0..arg.child_count() {
                                                if let Some(arg_child) = arg.child(j) {
                                                    if arg_child.kind() == "identifier" {
                                                        let name = &source[arg_child.start_byte()..arg_child.end_byte()];
                                                        calls.push(name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            Self::extract_calls_recursive_with_context(child, source, calls, impl_context);
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
