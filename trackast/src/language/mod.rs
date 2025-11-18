use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}

impl Language {
    #[must_use] 
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "js" | "ts" | "jsx" | "tsx" => Some(Language::JavaScript),
            _ => None,
        }
    }

    #[must_use] 
    pub fn from_file_path(path: &str) -> Option<Self> {
        let path = Path::new(path);
        let ext = path.extension()
            .and_then(|e| e.to_str())?;
        Self::from_extension(ext)
    }

    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_extension_rust() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
    }

    #[test]
    fn test_from_extension_python() {
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
    }

    #[test]
    fn test_from_extension_javascript() {
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("ts"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("jsx"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::JavaScript));
    }

    #[test]
    fn test_from_extension_unknown() {
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_from_file_path() {
        assert_eq!(Language::from_file_path("main.rs"), Some(Language::Rust));
        assert_eq!(Language::from_file_path("script.py"), Some(Language::Python));
        assert_eq!(Language::from_file_path("app.js"), Some(Language::JavaScript));
        assert_eq!(Language::from_file_path("utils/helpers.ts"), Some(Language::JavaScript));
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Language::Rust.as_str(), "Rust");
        assert_eq!(Language::Python.as_str(), "Python");
        assert_eq!(Language::JavaScript.as_str(), "JavaScript");
    }
}
