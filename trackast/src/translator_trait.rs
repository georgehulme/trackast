use trackast_lib::ast::AbstractAST;

/// Trait for language-specific translators
pub trait Translator {
    /// Translate a source file to an abstract AST
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    fn translate_file(&self, path: &str, module_path: Option<&str>) -> Result<AbstractAST, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translators::{RustTranslator, PythonTranslator, JavaScriptTranslator};

    #[test]
    fn test_rust_translator_implements_trait() {
        let translator = RustTranslator::new();
        let _: &dyn Translator = &translator;
    }

    #[test]
    fn test_python_translator_implements_trait() {
        let translator = PythonTranslator::new();
        let _: &dyn Translator = &translator;
    }

    #[test]
    fn test_javascript_translator_implements_trait() {
        let translator = JavaScriptTranslator::new();
        let _: &dyn Translator = &translator;
    }
}
