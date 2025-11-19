use tree_sitter::Parser;
use trackast_lib::ast::{AbstractAST, FunctionDef, Signature, FunctionCall};

/// Translator for JavaScript/TypeScript source code to abstract AST
pub struct JavaScriptTranslator;

impl JavaScriptTranslator {
    /// Create a new JavaScript translator
    #[must_use] 
    pub fn new() -> Self {
        JavaScriptTranslator
    }

    /// Set up a parser for JavaScript
    ///
    /// # Errors
    ///
    /// Returns an error if the parser cannot be initialized or language set.
    pub fn setup_parser() -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_javascript::language();
        parser
            .set_language(language)
            .map_err(|_| "Failed to set JavaScript language".to_string())?;
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
        match node.kind() {
            "function_declaration" | "function" => {
                // Find the identifier child
                for child in node.children(&mut node.walk()) {
                    if child.kind() == "identifier" {
                        let name = &source[child.start_byte()..child.end_byte()];
                        functions.push(name.to_string());
                        break;
                    }
                }
            }
            _ => {}
        }

        for child in node.children(&mut node.walk()) {
            Self::extract_functions_recursive(child, source, functions);
        }
    }

    /// Extract module path from JavaScript file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file path is invalid.
    pub fn extract_module_path(&self, file_path: &str) -> Result<String, String> {
        // Convert file path to module path (e.g., utils/helpers.js -> utils::helpers)
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
        // Look for call_expression nodes
        if node.kind() == "call_expression" {
            // The function being called is the first child
            if let Some(child) = node.child(0) {
                if child.kind() == "identifier" {
                    let name = &source[child.start_byte()..child.end_byte()];
                    calls.push(name.to_string());
                }
            }

            // Also extract function references passed as arguments to method calls
            // e.g., app.get('/path', handleGet) -> extract handleGet
            // e.g., app.use(middleware) -> extract middleware
            if let Some(callee) = node.child(0) {
                if callee.kind() == "member_expression" {
                    // Extract the method name to check if it's a known Express method
                    let mut is_express_method = false;
                    if let Some(property) = callee.child(callee.child_count() - 1) {
                        if property.kind() == "property_identifier" {
                            let method_name = &source[property.start_byte()..property.end_byte()];
                            // Check for Express route methods
                            is_express_method = matches!(method_name, "get" | "post" | "put" | "delete" | "patch" | "use" | "all");
                        }
                    }

                    if is_express_method {
                        // Extract identifiers from arguments
                        for i in 0..node.child_count() {
                            if let Some(arg) = node.child(i) {
                                if arg.kind() == "arguments" {
                                    for j in 0..arg.child_count() {
                                        if let Some(arg_child) = arg.child(j) {
                                            if arg_child.kind() == "identifier" {
                                                let name = &source[arg_child.start_byte()..arg_child.end_byte()];
                                                // Filter out common Express middleware parameter names
                                                if name != "req" && name != "res" && name != "next" && name != "err" {
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

        // Extract module.exports references
        if node.kind() == "assignment_expression" {
            // Look for module.exports = something pattern
            let mut is_module_exports = false;

            for child in node.children(&mut node.walk()) {
                if child.kind() == "member_expression" {
                    // Check if it's module.exports
                    let text = &source[child.start_byte()..child.end_byte()];
                    if text == "module.exports" {
                        is_module_exports = true;
                    }
                }
            }

            if is_module_exports {
                // Find the right side of the assignment
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Skip the assignment operator and member_expression
                        if child.kind() != "=" && child.kind() != "member_expression" {
                            if child.kind() == "identifier" {
                                let name = &source[child.start_byte()..child.end_byte()];
                                calls.push(name.to_string());
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

    /// Translate JavaScript source to abstract AST
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
        if node.kind() == "class_declaration" || node.kind() == "class" {
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

        if node.kind() == "function_declaration" || node.kind() == "function" {
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

                // Create function definition with class context
                let sig = Signature::empty();
                let scoped_name = if class_context.is_empty() {
                    func_name
                } else {
                    format!("{}.{}", class_context, func_name)
                };
                let mut func_def = FunctionDef::new(scoped_name, sig, module.to_string());
                
                for call_name in calls {
                    let call = FunctionCall::new(call_name, None, 0);
                    func_def.add_call(call);
                }

                ast.add_function(func_def);
            }
        } else if node.kind() == "variable_declaration" && class_context.is_empty() {
            // Handle const/let/var with arrow functions or function expressions
            // e.g., const handler = () => {...}; or const handler = function() {...};
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declarator" {
                        let mut var_name = String::new();
                        let mut func_body_node: Option<tree_sitter::Node> = None;

                        let decl_child_count = child.child_count();
                        for j in 0..decl_child_count {
                            if let Some(decl_child) = child.child(j) {
                                if decl_child.kind() == "identifier" && var_name.is_empty() {
                                    var_name = source[decl_child.start_byte()..decl_child.end_byte()].to_string();
                                }
                                // Check if it's an arrow function
                                if decl_child.kind() == "arrow_function" {
                                    func_body_node = Some(decl_child);
                                }
                            }
                        }

                        if !var_name.is_empty() && func_body_node.is_some() {
                            if let Some(func_node) = func_body_node {
                                // Extract calls from this function
                                let mut calls = Vec::new();
                                Self::extract_calls_from_function(func_node, source, &mut calls);

                                // Create function definition
                                let sig = Signature::empty();
                                let mut func_def = FunctionDef::new(var_name, sig, module.to_string());
                                
                                for call_name in calls {
                                    let call = FunctionCall::new(call_name, None, 0);
                                    func_def.add_call(call);
                                }

                                ast.add_function(func_def);
                            }
                        }
                    }
                }
            }
        } else if node.kind() == "method_definition" {
            // Handle JavaScript class methods
            let mut func_name = String::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "property_identifier" {
                    func_name = source[child.start_byte()..child.end_byte()].to_string();
                    break;
                }
            }

            if !func_name.is_empty() {
                // Extract calls from this method
                let mut calls = Vec::new();
                Self::extract_calls_from_function(node, source, &mut calls);

                // Create function definition with class context
                let sig = Signature::empty();
                let scoped_name = format!("{}.{}", class_context, func_name);
                let mut func_def = FunctionDef::new(scoped_name, sig, module.to_string());
                
                for call_name in calls {
                    let call = FunctionCall::new(call_name, None, 0);
                    func_def.add_call(call);
                }

                ast.add_function(func_def);
            }
        } else if node.kind() == "expression_statement" && class_context.is_empty() {
            // Handle top-level expression statements like app.get() or module.exports
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
            Self::extract_ast_recursive(child, source, module, ast, class_context);
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

    /// Translate a JavaScript file to abstract AST
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

impl crate::translator_trait::Translator for JavaScriptTranslator {
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

impl Default for JavaScriptTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_translator_new() {
        let translator = JavaScriptTranslator::new();
        assert_eq!(std::mem::size_of_val(&translator), 0);
    }

    #[test]
    fn test_setup_parser() {
        let result = JavaScriptTranslator::setup_parser();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_javascript() {
        let translator = JavaScriptTranslator::new();
        let source = "function main() {}";
        let result = translator.parse_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_function_names() {
        let translator = JavaScriptTranslator::new();
        let source = "function main() {}\nfunction helper() {}";
        let names = translator.query_function_names(source).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"helper".to_string()));
    }

    #[test]
    fn test_extract_module_path() {
        let translator = JavaScriptTranslator::new();
        let path = "utils/helpers.js";
        let module = translator.extract_module_path(path).unwrap();
        assert_eq!(module, "utils::helpers");
    }

    #[test]
    fn test_translate_simple() {
        let translator = JavaScriptTranslator::new();
        let source = "function main() {}\nfunction helper() {}";
        let ast = translator.translate(source, "mymod").unwrap();
        assert_eq!(ast.module_path(), "mymod");
        assert!(ast.functions.len() >= 2);
    }
}
