use crate::translator_trait::Translator;
use crate::translators::{RustTranslator, PythonTranslator, JavaScriptTranslator};
use crate::language::Language;

/// Factory for creating translators based on language
#[must_use] 
pub fn get_translator(language: Language) -> Box<dyn Translator> {
    match language {
        Language::Rust => Box::new(RustTranslator::new()),
        Language::Python => Box::new(PythonTranslator::new()),
        Language::JavaScript => Box::new(JavaScriptTranslator::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_translator_rust() {
        let translator = get_translator(Language::Rust);
        let _: &dyn Translator = &*translator;
    }

    #[test]
    fn test_get_translator_python() {
        let translator = get_translator(Language::Python);
        let _: &dyn Translator = &*translator;
    }

    #[test]
    fn test_get_translator_javascript() {
        let translator = get_translator(Language::JavaScript);
        let _: &dyn Translator = &*translator;
    }
}
