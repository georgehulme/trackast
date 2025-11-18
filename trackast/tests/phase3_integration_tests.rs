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
