use trackast::translator_factory::get_translator;
use trackast::language::Language;
use trackast_lib::builder::CallGraphBuilder;
use trackast_lib::function_id::FunctionId;
use trackast_lib::traversal::traversal_from_entries;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/entry_point")
        .join(name)
}

/// Helper to build a graph and test entry point resolution
/// Returns (`reachable_count`, `reachable_ids`)
fn test_entry_point_with_module(
    fixture: &str,
    language: Language,
    module: &str,
    entry_point_func: &str,
    _expect_reachable_min: usize,
) -> (usize, Vec<String>) {
    let translator = get_translator(language);
    let ast = translator
        .translate_file(fixture_path(fixture).to_str().unwrap(), Some(module))
        .expect("Failed to translate file");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    // Manually resolve entry point (fuzzy match)
    let matching: Vec<FunctionId> = graph
        .nodes
        .keys()
        .filter(|id| {
            let id_str = id.as_str();
            let id_parts: Vec<&str> = id_str.splitn(3, "::").collect();
            if id_parts.len() >= 2 {
                id_parts[0] == module && id_parts[1] == entry_point_func
            } else {
                false
            }
        })
        .cloned()
        .collect();

    assert!(
        !matching.is_empty(),
        "Entry point {}::{} not found. Available: {:?}",
        module,
        entry_point_func,
        graph
            .nodes
            .keys()
            .take(5)
            .map(trackast_lib::function_id::FunctionId::as_str)
            .collect::<Vec<_>>()
    );

    let traversal = traversal_from_entries(&graph, &matching);
    let reachable_count = traversal.reachable.len();
    
    let reachable_ids: Vec<String> = traversal
        .reachable
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    (reachable_count, reachable_ids)
}

#[test]
fn test_rust_entry_point_with_modules() {
    let (reachable_count, reachable_ids) = test_entry_point_with_module(
        "rust/main.rs",
        Language::Rust,
        "app",
        "main_entry",
        4,
    );

    assert!(
        reachable_count >= 3,
        "Expected at least 3 reachable functions, got {reachable_count}"
    );

    let reachable_str = reachable_ids.join(",");

    assert!(
        !reachable_str.contains("unused_function"),
        "unused_function should not be reachable"
    );
    assert!(
        !reachable_str.contains("another_unused"),
        "another_unused should not be reachable"
    );

    assert!(
        reachable_str.contains("main_entry"),
        "main_entry should be reachable"
    );
}

#[test]
fn test_python_entry_point_with_modules() {
    let (reachable_count, reachable_ids) = test_entry_point_with_module(
        "python/main.py",
        Language::Python,
        "app",
        "main_entry",
        4,
    );

    assert!(
        reachable_count >= 3,
        "Expected at least 3 reachable functions, got {reachable_count}"
    );

    let reachable_str = reachable_ids.join(",");

    assert!(
        !reachable_str.contains("unused_function"),
        "unused_function should not be reachable"
    );
    assert!(
        !reachable_str.contains("another_unused"),
        "another_unused should not be reachable"
    );

    assert!(
        reachable_str.contains("main_entry"),
        "main_entry should be reachable"
    );
}

#[test]
fn test_javascript_entry_point_with_modules() {
    let (reachable_count, reachable_ids) = test_entry_point_with_module(
        "javascript/main.js",
        Language::JavaScript,
        "app",
        "mainEntry",
        4,
    );

    assert!(
        reachable_count >= 3,
        "Expected at least 3 reachable functions, got {reachable_count}"
    );

    let reachable_str = reachable_ids.join(",");

    assert!(
        !reachable_str.contains("unusedFunction"),
        "unusedFunction should not be reachable"
    );
    assert!(
        !reachable_str.contains("anotherUnused"),
        "anotherUnused should not be reachable"
    );

    assert!(
        reachable_str.contains("mainEntry"),
        "mainEntry should be reachable"
    );
}

#[test]
fn test_rust_entry_point_called_functions_included() {
    let (_reachable_count, reachable_ids) = test_entry_point_with_module(
        "rust/main.rs",
        Language::Rust,
        "app",
        "main_entry",
        4,
    );

    let reachable_str = reachable_ids.join("|");

    assert!(reachable_str.contains("main_entry"), "main_entry must be reachable");

    assert!(
        reachable_str.contains("process_data") || reachable_str.contains("<external>"),
        "process_data or external functions must be reachable"
    );
}

#[test]
fn test_python_entry_point_called_functions_included() {
    let (reachable_count, reachable_ids) = test_entry_point_with_module(
        "python/main.py",
        Language::Python,
        "app",
        "main_entry",
        4,
    );

    let reachable_str = reachable_ids.join("|");

    assert!(reachable_str.contains("main_entry"), "main_entry must be reachable");

    assert!(
        reachable_count >= 3,
        "Should reach through the call chain"
    );
}

#[test]
fn test_javascript_entry_point_called_functions_included() {
    let (reachable_count, reachable_ids) = test_entry_point_with_module(
        "javascript/main.js",
        Language::JavaScript,
        "app",
        "mainEntry",
        4,
    );

    let reachable_str = reachable_ids.join("|");

    assert!(reachable_str.contains("mainEntry"), "mainEntry must be reachable");

    assert!(
        reachable_count >= 3,
        "Should reach through the call chain"
    );
}

#[test]
fn test_rust_full_graph_without_entry_points() {
    let translator = get_translator(Language::Rust);
    let ast = translator
        .translate_file(
            fixture_path("rust/main.rs").to_str().unwrap(),
            Some("app"),
        )
        .expect("Failed to translate Rust file");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    let all_functions: Vec<String> = graph
        .nodes
        .keys()
        .map(std::string::ToString::to_string)
        .collect();

    let all_str = all_functions.join(",");

    // The main.rs file declares these functions
    assert!(
        all_str.contains("unused_function"),
        "Full graph should include unused_function"
    );

    // Note: another_unused is in utils.rs which won't be loaded without --discover
    // When loading without discovery, we only get functions from the single file
    assert!(
        graph.node_count() >= 4,
        "Full graph should have at least 4 nodes (from main.rs)"
    );
}

#[test]
fn test_python_full_graph_without_entry_points() {
    let translator = get_translator(Language::Python);
    let ast = translator
        .translate_file(
            fixture_path("python/main.py").to_str().unwrap(),
            Some("app"),
        )
        .expect("Failed to translate Python file");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    let all_functions: Vec<String> = graph
        .nodes
        .keys()
        .map(std::string::ToString::to_string)
        .collect();

    let all_str = all_functions.join(",");

    assert!(
        all_str.contains("unused_function"),
        "Full graph should include unused_function"
    );
}

#[test]
fn test_javascript_full_graph_without_entry_points() {
    let translator = get_translator(Language::JavaScript);
    let ast = translator
        .translate_file(
            fixture_path("javascript/main.js").to_str().unwrap(),
            Some("app"),
        )
        .expect("Failed to translate JavaScript file");

    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast).expect("Failed to add AST");
    let graph = builder.build().expect("Failed to build graph");

    let all_functions: Vec<String> = graph
        .nodes
        .keys()
        .map(std::string::ToString::to_string)
        .collect();

    let all_str = all_functions.join(",");

    assert!(
        all_str.contains("unusedFunction"),
        "Full graph should include unusedFunction"
    );
}
