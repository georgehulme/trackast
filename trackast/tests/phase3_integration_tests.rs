use trackast::translator_trait::Translator;
use trackast::translator_factory::get_translator;
use trackast::language::Language;
use trackast_lib::builder::CallGraphBuilder;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn test_translate_python_simple() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(fixture_path("python/simple.py").to_str().unwrap(), None)
        .expect("Failed to translate Python file");

    assert!(!ast.module_path().is_empty());
    assert!(ast.functions.len() >= 2);
}

#[test]
fn test_translate_javascript_simple() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(fixture_path("javascript/simple.js").to_str().unwrap(), None)
        .expect("Failed to translate JavaScript file");

    assert!(!ast.module_path().is_empty());
    assert!(ast.functions.len() >= 2);
}

#[test]
fn test_translate_rust_simple() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(fixture_path("rust/simple.rs").to_str().unwrap(), Some("rust_test"))
        .expect("Failed to translate Rust file");

    assert_eq!(ast.module_path(), "rust_test");
    assert!(ast.functions.len() >= 2);
}

#[test]
fn test_build_python_callgraph() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(fixture_path("python/simple.py").to_str().unwrap(), None)
        .expect("Failed to translate");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    assert!(graph.node_count() > 0);
    assert!(graph.edge_count() > 0);
}

#[test]
fn test_build_javascript_callgraph() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(fixture_path("javascript/simple.js").to_str().unwrap(), None)
        .expect("Failed to translate");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    assert!(graph.node_count() > 0);
    assert!(graph.edge_count() > 0);
}

#[test]
fn test_build_rust_callgraph() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(fixture_path("rust/simple.rs").to_str().unwrap(), Some("test"))
        .expect("Failed to translate");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    assert!(graph.node_count() > 0);
    assert!(graph.edge_count() > 0);
}

#[test]
fn test_language_detection_python() {
    let language = Language::from_file_path("script.py").expect("Failed to detect Python");
    assert_eq!(language, Language::Python);
}

#[test]
fn test_language_detection_javascript() {
    let language = Language::from_file_path("app.js").expect("Failed to detect JavaScript");
    assert_eq!(language, Language::JavaScript);
}

#[test]
fn test_language_detection_rust() {
    let language = Language::from_file_path("main.rs").expect("Failed to detect Rust");
    assert_eq!(language, Language::Rust);
}

#[test]
fn test_translator_trait_duck_typing() {
    let rust = get_translator(Language::Rust);
    let python = get_translator(Language::Python);
    let javascript = get_translator(Language::JavaScript);

    // Verify all implement Translator trait
    let _: &dyn Translator = &*rust;
    let _: &dyn Translator = &*python;
    let _: &dyn Translator = &*javascript;
}

#[test]
fn test_multi_language_support() {
    let languages = vec![Language::Rust, Language::Python, Language::JavaScript];

    for lang in languages {
        let translator = get_translator(lang);
        let _: &dyn Translator = &*translator;
    }
}

#[test]
fn test_translate_python_with_classes() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(fixture_path("python/with_classes.py").to_str().unwrap(), None)
        .expect("Failed to translate Python file with classes");

    assert!(!ast.module_path().is_empty());
    // Should have 4 functions: Calculator.__init__, Calculator.add, Calculator.validate, Logger.__init__
    assert_eq!(ast.functions.len(), 4);
    
    // Verify that methods have unique names scoped to their class
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"Calculator.__init__"));
    assert!(func_names.contains(&"Calculator.add"));
    assert!(func_names.contains(&"Calculator.validate"));
    assert!(func_names.contains(&"Logger.__init__"));
    
    // Verify that self.validate() call in Calculator.add() is properly detected
    let calculator_add = ast.functions.iter().find(|f| f.name == "Calculator.add").unwrap();
    assert_eq!(calculator_add.calls.len(), 1, "Calculator.add should have exactly 1 call");
    
    let validate_call = &calculator_add.calls[0];
    assert_eq!(validate_call.target_name, "Calculator.validate", 
               "Calculator.add should call Calculator.validate");
    assert_eq!(validate_call.target_module, Some(ast.module_path().to_string()),
               "Calculator.validate call should be resolved to the same module");
}

#[test]
fn test_python_class_method_call_graph_resolution() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(fixture_path("python/with_classes.py").to_str().unwrap(), None)
        .expect("Failed to translate Python file with classes");

    // Build the call graph to verify end-to-end resolution
    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    // Find the Calculator.add and Calculator.validate nodes
    let add_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator.add"))
        .expect("Should find Calculator.add function");
    
    let validate_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator.validate"))
        .expect("Should find Calculator.validate function");

    // Verify there's an edge from Calculator.add to Calculator.validate
    let edges_from_add: Vec<_> = graph.edges.iter()
        .filter(|edge| edge.from == *add_id)
        .collect();
    
    assert_eq!(edges_from_add.len(), 1, "Calculator.add should have exactly 1 outgoing edge");
    assert_eq!(edges_from_add[0].to, *validate_id, 
               "Calculator.add should have an edge to Calculator.validate");
}

#[test]
fn test_translate_rust_with_impl() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(fixture_path("rust/with_impl.rs").to_str().unwrap(), Some("rust_impl_test"))
        .expect("Failed to translate Rust file with impl blocks");

    assert_eq!(ast.module_path(), "rust_impl_test");
    // Should have 4 functions: Calculator::new, Calculator::add, Calculator::validate, Logger::new
    assert_eq!(ast.functions.len(), 4);
    
    // Verify that methods have unique names scoped to their impl type
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"Calculator::new"));
    assert!(func_names.contains(&"Calculator::add"));
    assert!(func_names.contains(&"Calculator::validate"));
    assert!(func_names.contains(&"Logger::new"));
    
    // Verify that self.validate() call in Calculator::add() is properly detected
    let calculator_add = ast.functions.iter().find(|f| f.name == "Calculator::add").unwrap();
    assert_eq!(calculator_add.calls.len(), 1, "Calculator::add should have exactly 1 call");
    
    let validate_call = &calculator_add.calls[0];
    assert_eq!(validate_call.target_name, "Calculator::validate", 
               "Calculator::add should call Calculator::validate");
    assert_eq!(validate_call.target_module, Some(ast.module_path().to_string()),
               "Calculator::validate call should be resolved to the same module");
}

#[test]
fn test_rust_impl_method_call_graph_resolution() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(fixture_path("rust/with_impl.rs").to_str().unwrap(), Some("rust_impl_test"))
        .expect("Failed to translate Rust file with impl blocks");

    // Build the call graph to verify end-to-end resolution
    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    // Find the Calculator::add and Calculator::validate nodes
    let add_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator::add"))
        .expect("Should find Calculator::add function");
    
    let validate_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator::validate"))
        .expect("Should find Calculator::validate function");

    // Verify there's an edge from Calculator::add to Calculator::validate
    let edges_from_add: Vec<_> = graph.edges.iter()
        .filter(|edge| edge.from == *add_id)
        .collect();
    
    assert_eq!(edges_from_add.len(), 1, "Calculator::add should have exactly 1 outgoing edge");
    assert_eq!(edges_from_add[0].to, *validate_id, 
               "Calculator::add should have an edge to Calculator::validate");
}

#[test]
fn test_translate_javascript_with_classes() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(fixture_path("javascript/with_classes.js").to_str().unwrap(), None)
        .expect("Failed to translate JavaScript file with classes");

    assert!(!ast.module_path().is_empty());
    // Should have 5 functions: Calculator.constructor, Calculator.add, Calculator.validate, Logger.constructor, Logger.log
    assert_eq!(ast.functions.len(), 5);
    
    // Verify that methods have unique names scoped to their class
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"Calculator.constructor"));
    assert!(func_names.contains(&"Calculator.add"));
    assert!(func_names.contains(&"Calculator.validate"));
    assert!(func_names.contains(&"Logger.constructor"));
    assert!(func_names.contains(&"Logger.log"));
    
    // Verify that this.validate() call in Calculator.add() is properly detected
    let calculator_add = ast.functions.iter().find(|f| f.name == "Calculator.add").unwrap();
    assert_eq!(calculator_add.calls.len(), 1, "Calculator.add should have exactly 1 call");
    
    let validate_call = &calculator_add.calls[0];
    assert_eq!(validate_call.target_name, "Calculator.validate", 
               "Calculator.add should call Calculator.validate");
    assert_eq!(validate_call.target_module, Some(ast.module_path().to_string()),
               "Calculator.validate call should be resolved to the same module");
}

#[test]
fn test_javascript_class_method_call_graph_resolution() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(fixture_path("javascript/with_classes.js").to_str().unwrap(), None)
        .expect("Failed to translate JavaScript file with classes");

    // Build the call graph to verify end-to-end resolution
    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    // Find the Calculator.add and Calculator.validate nodes
    let add_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator.add"))
        .expect("Should find Calculator.add function");
    
    let validate_id = graph.nodes.keys()
        .find(|id| id.as_str().contains("Calculator.validate"))
        .expect("Should find Calculator.validate function");

    // Verify there's an edge from Calculator.add to Calculator.validate
    let edges_from_add: Vec<_> = graph.edges.iter()
        .filter(|edge| edge.from == *add_id)
        .collect();
    
    assert_eq!(edges_from_add.len(), 1, "Calculator.add should have exactly 1 outgoing edge");
    assert_eq!(edges_from_add[0].to, *validate_id, 
               "Calculator.add should have an edge to Calculator.validate");
}

#[test]
fn test_translate_javascript_express_server() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(fixture_path("javascript/express_server.js").to_str().unwrap(), None)
        .expect("Failed to translate JavaScript Express server");

    assert!(!ast.module_path().is_empty());
    // Should have at least 4 functions: handleGetUsers, handleCreateUser, validateUser, errorHandler
    assert!(ast.functions.len() >= 4);
    
    // Verify we have all named functions
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"handleGetUsers"));
    assert!(func_names.contains(&"handleCreateUser"));
    assert!(func_names.contains(&"validateUser"));
    assert!(func_names.contains(&"errorHandler"));
    
    // Verify that routes and exports create edges - check that the <module> function
    // has calls to the route handlers
    let module_func = ast.functions.iter().find(|f| f.name == "<module>");
    assert!(module_func.is_some(), "Expected to find a <module> function tracking route handlers");
    
    if let Some(module_f) = module_func {
        // Should have calls to handleGetUsers, handleCreateUser, errorHandler
        let call_targets: Vec<&str> = module_f.calls.iter().map(|c| c.target_name.as_str()).collect();
        assert!(call_targets.contains(&"handleGetUsers"));
        assert!(call_targets.contains(&"handleCreateUser"));
        assert!(call_targets.contains(&"errorHandler"));
    }
}

#[test]
fn test_translate_python_flask_app() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(fixture_path("python/flask_app.py").to_str().unwrap(), None)
        .expect("Failed to translate Python Flask app");

    assert!(!ast.module_path().is_empty());
    // Should have at least 5 functions: handle_get_users, validate_user, error_handler, get_users, create_user
    assert!(ast.functions.len() >= 5);
    
    // Verify we have all named functions
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"handle_get_users"));
    assert!(func_names.contains(&"validate_user"));
    assert!(func_names.contains(&"error_handler"));
    assert!(func_names.contains(&"get_users"));
    assert!(func_names.contains(&"create_user"));
    
    // Verify that module-level registrations are tracked with a <module> function
    let module_func = ast.functions.iter().find(|f| f.name == "<module>");
    assert!(module_func.is_some(), "Expected to find a <module> function tracking Flask registrations");
}

#[test]
fn test_translate_rust_actix_app() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(fixture_path("rust/actix_app.rs").to_str().unwrap(), Some("actix_test"))
        .expect("Failed to translate Rust Actix app");

    assert_eq!(ast.module_path(), "actix_test");
    // Should have at least 4 functions: get_users, create_user, validate_user, error_handler
    assert!(ast.functions.len() >= 4);
    
    // Verify we have all named functions
    let func_names: Vec<&str> = ast.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"get_users"));
    assert!(func_names.contains(&"create_user"));
    assert!(func_names.contains(&"validate_user"));
    assert!(func_names.contains(&"error_handler"));
}
