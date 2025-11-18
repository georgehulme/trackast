use std::fmt;
use std::hash::{Hash, Hasher};
use crate::ast::Signature;

/// Unique identifier for a function: `module::name::signature`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionId(String);

impl FunctionId {
    #[must_use] 
    pub fn new(id: String) -> Self {
        FunctionId(id)
    }

    #[must_use] 
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for FunctionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Generate a unique function ID from module, name, and signature
#[must_use] 
pub fn generate_id(module: &str, name: &str, signature: &Signature) -> FunctionId {
    let sig_str = signature.to_string();
    let id = format!("{module}::{name}::{sig_str}");
    FunctionId::new(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_id_display() {
        let id = FunctionId::new("mod::func::()".to_string());
        assert_eq!(id.to_string(), "mod::func::()");
    }

    #[test]
    fn test_function_id_as_str() {
        let id = FunctionId::new("mod::func::()".to_string());
        assert_eq!(id.as_str(), "mod::func::()");
    }

    #[test]
    fn test_function_id_equality() {
        let id1 = FunctionId::new("mod::func::()".to_string());
        let id2 = FunctionId::new("mod::func::()".to_string());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_function_id_hash() {
        use std::collections::hash_map::DefaultHasher;
        let id1 = FunctionId::new("mod::func::()".to_string());
        let id2 = FunctionId::new("mod::func::()".to_string());
        
        let mut hasher1 = DefaultHasher::new();
        id1.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        
        let mut hasher2 = DefaultHasher::new();
        id2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_id_simple() {
        let sig = Signature::empty();
        let id = generate_id("root", "main", &sig);
        assert_eq!(id.to_string(), "root::main::() -> ()");
    }

    #[test]
    fn test_generate_id_with_params() {
        let sig = Signature::new(
            vec![("x".to_string(), "i32".to_string())],
            "String".to_string(),
        );
        let id = generate_id("my_crate", "parse", &sig);
        assert_eq!(id.to_string(), "my_crate::parse::(x: i32) -> String");
    }

    #[test]
    fn test_generate_id_nested_module() {
        let sig = Signature::empty();
        let id = generate_id("my_crate::utils::helpers", "process", &sig);
        assert_eq!(id.to_string(), "my_crate::utils::helpers::process::() -> ()");
    }

    #[test]
    fn test_generate_id_generics() {
        let sig = Signature::new(
            vec![("item".to_string(), "T".to_string())],
            "Option<T>".to_string(),
        );
        let id = generate_id("std::vec", "push", &sig);
        assert_eq!(id.to_string(), "std::vec::push::(item: T) -> Option<T>");
    }
}
