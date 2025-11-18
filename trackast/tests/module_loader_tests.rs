use trackast::module_loader::ModuleLoader;
use trackast::language::Language;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures")
}

#[test]
fn test_module_loader_create() {
    let _loader = ModuleLoader::new(fixture_dir(), Language::Rust);
    // Verify it was created successfully (no panic)
}

#[test]
fn test_extract_rust_imports_from_file() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Rust);
    let path = fixture_dir().join("rust/with_imports.rs");
    let imports = loader.extract_imports_from_file(&path).expect("Failed to extract imports");
    assert!(!imports.is_empty());
    assert!(imports.iter().any(|i| i.contains("helper")));
}

#[test]
fn test_extract_python_imports_from_file() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Python);
    let path = fixture_dir().join("python/with_imports.py");
    let imports = loader.extract_imports_from_file(&path).expect("Failed to extract imports");
    assert!(!imports.is_empty());
    assert!(imports.iter().any(|i| i.contains("helper")));
}

#[test]
fn test_extract_js_imports_from_file() {
    let loader = ModuleLoader::new(fixture_dir(), Language::JavaScript);
    let path = fixture_dir().join("javascript/with_imports.js");
    let imports = loader.extract_imports_from_file(&path).expect("Failed to extract imports");
    assert!(!imports.is_empty());
}

#[test]
fn test_rust_imports_parsing() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Rust);
    let source = "use std::fs;\nuse mymodule::submodule;\nuse crate::other;";
    let imports = loader.extract_rust_imports(source).unwrap();
    assert!(imports.contains(&"mymodule".to_string()));
    assert!(!imports.iter().any(|i| i.contains("std")));
    assert!(!imports.iter().any(|i| i.contains("crate")));
}

#[test]
fn test_python_imports_parsing() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Python);
    let source = "import os\nfrom mymodule import func\nimport numpy";
    let imports = loader.extract_python_imports(source).unwrap();
    assert!(imports.contains(&"mymodule".to_string()));
}

#[test]
fn test_js_imports_parsing() {
    let loader = ModuleLoader::new(fixture_dir(), Language::JavaScript);
    let source = "import x from 'mymodule';\nconst y = require('other');";
    let imports = loader.extract_js_imports(source).unwrap();
    assert!(imports.contains(&"mymodule".to_string()));
    assert!(imports.contains(&"other".to_string()));
}

#[test]
fn test_module_loader_filters_external_imports() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Rust);
    let source = "use std::fs;\nuse mylib::core;";
    let imports = loader.extract_rust_imports(source).unwrap();
    
    // Should have mylib but not std
    assert!(imports.iter().any(|i| i == "mylib"));
    assert!(!imports.iter().any(|i| i == "std"));
}

#[test]
fn test_module_loader_multiple_imports() {
    let loader = ModuleLoader::new(fixture_dir(), Language::Python);
    let source = "from helper import process\nfrom utils import format\nfrom external import lib";
    let imports = loader.extract_python_imports(source).unwrap();
    assert_eq!(imports.len(), 3);
}
