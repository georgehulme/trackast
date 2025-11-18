use crate::translator_factory::get_translator;
use crate::language::Language;
use trackast_lib::ast::AbstractAST;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Module loader that recursively discovers and loads all imported modules
pub struct ModuleLoader {
    root_path: PathBuf,
    language: Language,
    loaded_modules: HashSet<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader for a given root path and language
    pub fn new(root_path: impl AsRef<Path>, language: Language) -> Self {
        ModuleLoader {
            root_path: root_path.as_ref().to_path_buf(),
            language,
            loaded_modules: HashSet::new(),
        }
    }

    /// Load all modules recursively starting from entry point
    ///
    /// # Errors
    ///
    /// Returns an error if the entry point does not exist or if translation fails.
    pub fn load_all(&mut self, entry_point: &str) -> Result<AbstractAST, String> {
        // If entry_point is an absolute path, use it directly
        let entry_path = if std::path::Path::new(entry_point).is_absolute() {
            std::path::PathBuf::from(entry_point)
        } else {
            // Try to resolve from root_path
            let path = self.root_path.join(entry_point);
            if path.exists() {
                path
            } else {
                // Try entry_point as-is
                std::path::PathBuf::from(entry_point)
            }
        };

        if !entry_path.exists() {
            return Err(format!(
                "Entry point does not exist: {}",
                entry_path.display()
            ));
        }

        self.load_recursively(&entry_path)
    }

    /// Recursively load a file and all its dependencies
    fn load_recursively(&mut self, path: &PathBuf) -> Result<AbstractAST, String> {
        if self.loaded_modules.contains(path) {
            return Ok(AbstractAST::new("already_loaded".to_string()));
        }

        self.loaded_modules.insert(path.clone());

        let translator = get_translator(self.language);
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let ast = translator.translate_file(path.to_str().unwrap(), Some(module_name))?;

        // Extract imports from this file
        let imports = self.extract_imports(path)?;

        // Recursively load each imported module
        let mut combined_ast = ast.clone();
        for import_path in imports {
            if let Ok(resolved_path) = self.resolve_path(&import_path) {
                if !self.loaded_modules.contains(&resolved_path) {
                    if let Ok(imported_ast) = self.load_recursively(&resolved_path) {
                        // Merge ASTs
                        for func in imported_ast.functions {
                            combined_ast.add_function(func);
                        }
                    } else {
                        // External or non-existent module, skip silently
                    }
                }
            }
        }

        Ok(combined_ast)
    }

    /// Extract import statements from a source file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if extraction fails.
    pub fn extract_imports_from_file(&self, path: &Path) -> Result<Vec<String>, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;

        match self.language {
            Language::Rust => self.extract_rust_imports(&source),
            Language::Python => self.extract_python_imports(&source),
            Language::JavaScript => self.extract_js_imports(&source),
        }
    }

    /// Extract import statements from a source file (internal)
    fn extract_imports(&self, path: &Path) -> Result<Vec<String>, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;

        match self.language {
            Language::Rust => self.extract_rust_imports(&source),
            Language::Python => self.extract_python_imports(&source),
            Language::JavaScript => self.extract_js_imports(&source),
        }
    }

    /// Extract Rust imports (use statements)
    ///
    /// # Errors
    ///
    /// This function currently always succeeds, but returns Result for consistency.
    pub fn extract_rust_imports(&self, source: &str) -> Result<Vec<String>, String> {
        let mut imports = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                // Simple parsing: extract module path
                let after_use = trimmed.strip_prefix("use ").unwrap_or("");
                let path = after_use.split('{').next().unwrap_or("").trim();
                let path = path.split("::").next().unwrap_or("");

                if !path.is_empty() && path != "std" && path != "crate" {
                    imports.push(path.to_string());
                }
            }
        }

        Ok(imports)
    }

    /// Extract Python imports
    ///
    /// # Errors
    ///
    /// This function currently always succeeds, but returns Result for consistency.
    pub fn extract_python_imports(&self, source: &str) -> Result<Vec<String>, String> {
        let mut imports = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                let after_import = trimmed.strip_prefix("import ").unwrap_or("");
                let module = after_import.split(',').next().unwrap_or("").trim();
                if !module.is_empty() && !module.starts_with('.') {
                    imports.push(module.to_string());
                }
            } else if trimmed.starts_with("from ") && trimmed.contains(" import ") {
                if let Some(module) = trimmed.strip_prefix("from ") {
                    if let Some(module) = module.split(" import ").next() {
                        let module = module.trim();
                        if !module.is_empty() && !module.starts_with('.') {
                            imports.push(module.to_string());
                        }
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Extract JavaScript imports
    ///
    /// # Errors
    ///
    /// This function currently always succeeds, but returns Result for consistency.
    pub fn extract_js_imports(&self, source: &str) -> Result<Vec<String>, String> {
        let mut imports = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();
            
            // Handle: import x from 'path' or import x from "path"
            if trimmed.starts_with("import ") {
                if let Some(from_idx) = trimmed.find(" from ") {
                    let rest = &trimmed[from_idx + 6..];
                    if let Some(start) = rest.find('\'') {
                        if let Some(end) = rest[start + 1..].find('\'') {
                            let path = &rest[start + 1..start + 1 + end];
                            if !path.starts_with('.') {
                                imports.push(path.to_string());
                            } else if path.starts_with("./") {
                                // Local import like './helper.js'
                                imports.push(path.strip_prefix("./").unwrap_or(path).to_string());
                            }
                        }
                    } else if let Some(start) = rest.find('"') {
                        if let Some(end) = rest[start + 1..].find('"') {
                            let path = &rest[start + 1..start + 1 + end];
                            if !path.starts_with('.') {
                                imports.push(path.to_string());
                            } else if path.starts_with("./") {
                                imports.push(path.strip_prefix("./").unwrap_or(path).to_string());
                            }
                        }
                    }
                }
            }
            
            // Handle: require('path') or require("path") - anywhere in the line
            if trimmed.contains("require(") {
                if let Some(start) = trimmed.find("require(") {
                    let rest = &trimmed[start + 8..];
                    if let Some(quote_start) = rest.find('\'') {
                        if let Some(quote_end) = rest[quote_start + 1..].find('\'') {
                            let path = &rest[quote_start + 1..quote_start + 1 + quote_end];
                            if !path.starts_with('.') {
                                imports.push(path.to_string());
                            }
                        }
                    } else if let Some(quote_start) = rest.find('"') {
                        if let Some(quote_end) = rest[quote_start + 1..].find('"') {
                            let path = &rest[quote_start + 1..quote_start + 1 + quote_end];
                            if !path.starts_with('.') {
                                imports.push(path.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Resolve an import path to an actual file
    fn resolve_path(&self, import_path: &str) -> Result<PathBuf, String> {
        let extensions = match self.language {
            Language::Rust => vec!["rs"],
            Language::Python => vec!["py"],
            Language::JavaScript => vec!["js", "ts", "jsx", "tsx"],
        };

        // Try different resolution strategies
        for ext in extensions {
            // Strategy 1: Direct file with extension
            let path1 = self.root_path.join(format!("{import_path}.{ext}"));
            if path1.exists() {
                return Ok(path1);
            }

            // Strategy 2: Module directory with __init__.py or mod.rs
            let init_file = match self.language {
                Language::Rust => "mod.rs",
                Language::Python => "__init__.py",
                Language::JavaScript => "index.js",
            };
            let path2 = self.root_path.join(import_path).join(init_file);
            if path2.exists() {
                return Ok(path2);
            }

            // Strategy 3: Sibling directory
            let path3 = self.root_path.join(import_path).with_extension(ext);
            if path3.exists() {
                return Ok(path3);
            }
        }

        Err(format!("Could not resolve import: {import_path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_new() {
        let loader = ModuleLoader::new(".", Language::Rust);
        assert_eq!(loader.root_path, PathBuf::from("."));
        assert_eq!(loader.loaded_modules.len(), 0);
    }

    #[test]
    fn test_extract_rust_imports() {
        let loader = ModuleLoader::new(".", Language::Rust);
        let source = "use std::fs;\nuse mymodule::submodule;\nuse crate::other;";
        let imports = loader.extract_rust_imports(source).unwrap();
        assert!(imports.contains(&"mymodule".to_string()));
    }

    #[test]
    fn test_extract_python_imports() {
        let loader = ModuleLoader::new(".", Language::Python);
        let source = "import os\nfrom mymodule import func\nimport numpy";
        let imports = loader.extract_python_imports(source).unwrap();
        assert!(imports.contains(&"mymodule".to_string()));
    }

    #[test]
    fn test_extract_js_imports() {
        let loader = ModuleLoader::new(".", Language::JavaScript);
        let source = "import x from 'mymodule';\nconst y = require('other');";
        let imports = loader.extract_js_imports(source).unwrap();
        assert!(imports.contains(&"mymodule".to_string()));
    }

    #[test]
    fn test_language_specific_loaders() {
        let _rust = ModuleLoader::new(".", Language::Rust);
        let _python = ModuleLoader::new(".", Language::Python);
        let _js = ModuleLoader::new(".", Language::JavaScript);
    }
}
