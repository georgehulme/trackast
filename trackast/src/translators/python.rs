use tree_sitter::Parser;
use trackast_lib::ast::{AbstractAST, FunctionDef, Signature, FunctionCall};

/// Translator for Python source code to abstract AST
pub struct PythonTranslator;

impl PythonTranslator {
    /// Create a new Python translator
    #[must_use] 
    pub fn new() -> Self {
        PythonTranslator
    }

    /// Set up a parser for Python
    ///
    /// # Errors
    ///
    /// Returns an error if the parser cannot be initialized or language set.
    pub fn setup_parser() -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::language();
        parser
            .set_language(language)
            .map_err(|_| "Failed to set Python language".to_string())?;
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
        if node.kind() == "function_definition" {
            // Find the identifier child
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

    /// Extract module path from Python file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file path is invalid.
    pub fn extract_module_path(&self, file_path: &str) -> Result<String, String> {
        // Convert file path to module path (e.g., utils/helpers.py -> utils::helpers)
        let path = std::path::Path::new(file_path);
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file path")?;
        
        let parent = path.parent()
            .and_then(|p| p.to_str());
        
        if let Some(parent) = parent {
            if parent != "." && !parent.is_empty() {
                Ok(format!("{}::{}", parent.replace('/', "::"), stem))
            } else {
                Ok(stem.to_string())
            }
        } else {
            Ok(stem.to_string())
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
        // Look for call nodes
        if node.kind() == "call" {
            // The function being called is the first child
            if let Some(child) = node.child(0) {
                match child.kind() {
                    "identifier" => {
                        // Direct function call: function_name()
                        let name = &source[child.start_byte()..child.end_byte()];
                        calls.push(name.to_string());
                    }
                    "attribute" => {
                        // Attribute access call: obj.method() or self.method()
                        Self::extract_attribute_call(child, source, calls);
                    }
                    _ => {}
                }
            }

            // Extract function references passed to method calls like app.add_url_rule()
            // e.g., app.add_url_rule('/users', view_func=get_users)
            // e.g., app.register_error_handler(500, error_handler)
            if let Some(callee) = node.child(0) {
                if callee.kind() == "attribute" {
                    // Get the method name
                    let callee_text = &source[callee.start_byte()..callee.end_byte()];
                    // Check for common Flask/Django methods
                    if callee_text.ends_with(".add_url_rule") 
                        || callee_text.ends_with(".register_error_handler")
                        || callee_text.ends_with(".register_blueprint")
                        || callee_text.ends_with(".before_request")
                        || callee_text.ends_with(".after_request") {
                        // Extract identifier arguments (function references)
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

        for child in node.children(&mut node.walk()) {
            Self::extract_calls_recursive(child, source, calls);
        }
    }

    /// Extract method name from attribute access calls
    fn extract_attribute_call(
        attribute_node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
    ) {
        // Handle attribute access patterns: obj.method() or self.method()
        // The attribute node should have structure: object "." attribute
        let mut object_name = None;
        let mut method_name = None;
        
        for child in attribute_node.children(&mut attribute_node.walk()) {
            match child.kind() {
                "identifier" => {
                    if object_name.is_none() {
                        // First identifier is the object (e.g., "self", "obj", etc.)
                        object_name = Some(&source[child.start_byte()..child.end_byte()]);
                    } else {
                        // Second identifier is the method name
                        method_name = Some(&source[child.start_byte()..child.end_byte()]);
                    }
                }
                _ => {}
            }
        }

        if let (Some(obj), Some(method)) = (object_name, method_name) {
            if obj == "self" {
                // For self.method() calls, just use the method name
                // It will be resolved to the current class context
                calls.push(method.to_string());
            } else {
                // For other object method calls (e.g., obj.method()), 
                // we can't easily resolve the type, so just record the method name
                // This could be enhanced with more sophisticated type tracking
                calls.push(method.to_string());
            }
        }
    }

    /// Translate Python source to abstract AST
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
        class_context: &str,
    ) {
        if node.kind() == "class_definition" {
            // Extract class name
            let mut class_name = String::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    class_name = source[child.start_byte()..child.end_byte()].to_string();
                    break;
                }
            }

            // Recursively process children with class context
            for child in node.children(&mut node.walk()) {
                Self::extract_ast_recursive(child, source, module, ast, &class_name);
            }
            return;
        }

        if node.kind() == "function_definition" {
            // Extract function name
            let mut func_name = String::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    func_name = source[child.start_byte()..child.end_byte()].to_string();
                    break;
                }
            }

            if !func_name.is_empty() {
                // Extract calls from this function with class context for resolution
                let mut calls = Vec::new();
                Self::extract_calls_from_function_with_context(node, source, &mut calls, class_context);

                // Create function definition with class context
                let sig = Signature::empty(); // Python has no explicit type signatures
                let scoped_name = if class_context.is_empty() {
                    func_name
                } else {
                    format!("{}.{}", class_context, func_name)
                };
                let mut func_def = FunctionDef::new(scoped_name, sig, module.to_string());
                
                for call_name in calls {
                    // Determine if this is a local call that should be resolved within the module
                    let target_module = if call_name.contains('.') {
                        // For method calls like "MyClass.method2", try to resolve within current module
                        Some(module.to_string())
                    } else {
                        // For simple function calls, leave as None (external)
                        None
                    };
                    let call = FunctionCall::new(call_name, target_module, 0);
                    func_def.add_call(call);
                }

                ast.add_function(func_def);
            }
        }

        if node.kind() == "expression_statement" && class_context.is_empty() {
            // Handle top-level expression statements like app.add_url_rule()
            let mut calls = Vec::new();
            Self::extract_calls_recursive(node, source, &mut calls);
            
            if !calls.is_empty() {
                // Create a virtual module-level function to track these references
                let sig = Signature::empty();
                let mut func_def = FunctionDef::new("<module>".to_string(), sig, module.to_string());
                
                for call_name in calls {
                    // Determine if this is a local call that should be resolved within the module
                    let target_module = if call_name.contains('.') {
                        // For method calls like "MyClass.method2", try to resolve within current module
                        Some(module.to_string())
                    } else {
                        // For simple function calls, leave as None (external)
                        None
                    };
                    let call = FunctionCall::new(call_name, target_module, 0);
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
            Self::extract_ast_recursive(child, source, module, ast, class_context);
        }
    }


    /// Extract calls within a single function with class context for better resolution
    fn extract_calls_from_function_with_context(
        func_node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
        class_context: &str,
    ) {
        for child in func_node.children(&mut func_node.walk()) {
            Self::extract_calls_recursive_with_context(child, source, calls, class_context);
        }
    }

    /// Recursively find function calls with class context for better resolution
    fn extract_calls_recursive_with_context(
        node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
        class_context: &str,
    ) {
        // Look for call nodes
        if node.kind() == "call" {
            // The function being called is the first child
            if let Some(child) = node.child(0) {
                match child.kind() {
                    "identifier" => {
                        // Direct function call: function_name()
                        let name = &source[child.start_byte()..child.end_byte()];
                        calls.push(name.to_string());
                    }
                    "attribute" => {
                        // Attribute access call: obj.method() or self.method()
                        Self::extract_attribute_call_with_context(child, source, calls, class_context);
                    }
                    _ => {}
                }
            }

            // Extract function references passed to method calls like app.add_url_rule()
            // e.g., app.add_url_rule('/users', view_func=get_users)
            // e.g., app.register_error_handler(500, error_handler)
            if let Some(callee) = node.child(0) {
                if callee.kind() == "attribute" {
                    // Get the method name
                    let callee_text = &source[callee.start_byte()..callee.end_byte()];
                    // Check for common Flask/Django methods
                    if callee_text.ends_with(".add_url_rule") 
                        || callee_text.ends_with(".register_error_handler")
                        || callee_text.ends_with(".register_blueprint")
                        || callee_text.ends_with(".before_request")
                        || callee_text.ends_with(".after_request") {
                        // Extract identifier arguments (function references)
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

        for child in node.children(&mut node.walk()) {
            Self::extract_calls_recursive_with_context(child, source, calls, class_context);
        }
    }

    /// Extract method name from attribute access calls with class context
    fn extract_attribute_call_with_context(
        attribute_node: tree_sitter::Node,
        source: &str,
        calls: &mut Vec<String>,
        class_context: &str,
    ) {
        // Handle attribute access patterns: obj.method() or self.method()
        // The attribute node should have structure: object "." attribute
        let mut object_name = None;
        let mut method_name = None;
        
        for child in attribute_node.children(&mut attribute_node.walk()) {
            match child.kind() {
                "identifier" => {
                    if object_name.is_none() {
                        // First identifier is the object (e.g., "self", "obj", etc.)
                        object_name = Some(&source[child.start_byte()..child.end_byte()]);
                    } else {
                        // Second identifier is the method name
                        method_name = Some(&source[child.start_byte()..child.end_byte()]);
                    }
                }
                _ => {}
            }
        }

        if let (Some(obj), Some(method)) = (object_name, method_name) {
            if obj == "self" && !class_context.is_empty() {
                // For self.method() calls, resolve to the current class context
                let resolved_method = format!("{}.{}", class_context, method);
                calls.push(resolved_method);
            } else {
                // For other object method calls (e.g., obj.method()), 
                // we can't easily resolve the type, so just record the method name
                // This could be enhanced with more sophisticated type tracking
                calls.push(method.to_string());
            }
        }
    }

    /// Translate a Python file to abstract AST
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails.
    pub fn translate_file(&self, path: &str, module_path: Option<&str>) -> Result<AbstractAST, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        
        let module = if let Some(m) = module_path {
            m.to_string()
        } else {
            self.extract_module_path(path)?
        };
        
        self.translate(&source, &module)
    }
}

impl crate::translator_trait::Translator for PythonTranslator {
    fn translate_file(&self, path: &str, module_path: Option<&str>) -> Result<AbstractAST, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let module = if let Some(m) = module_path {
            m.to_string()
        } else {
            self.extract_module_path(path)?
        };
        self.translate(&source, &module)
    }
}

impl Default for PythonTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_translator_new() {
        let translator = PythonTranslator::new();
        assert_eq!(std::mem::size_of_val(&translator), 0);
    }

    #[test]
    fn test_setup_parser() {
        let result = PythonTranslator::setup_parser();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_python() {
        let translator = PythonTranslator::new();
        let source = "def main():\n    pass";
        let result = translator.parse_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_function_names() {
        let translator = PythonTranslator::new();
        let source = "def main():\n    pass\ndef helper():\n    pass";
        let names = translator.query_function_names(source).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"helper".to_string()));
    }

    #[test]
    fn test_extract_module_path() {
        let translator = PythonTranslator::new();
        let path = "utils/helpers.py";
        let module = translator.extract_module_path(path).unwrap();
        assert_eq!(module, "utils::helpers");
    }

    #[test]
    fn test_translate_simple() {
        let translator = PythonTranslator::new();
        let source = "def main():\n    pass\ndef helper():\n    pass";
        let ast = translator.translate(source, "mymod").unwrap();
        assert_eq!(ast.module_path(), "mymod");
        assert!(ast.functions.len() >= 2);
    }
}
