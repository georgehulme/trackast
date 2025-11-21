use std::fmt;
use serde::Serialize;

/// Function signature with parameters and return type
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Signature {
    pub params: Vec<(String, String)>, // (name, type)
    pub return_type: String,
}

impl Signature {
    #[must_use] 
    pub fn new(params: Vec<(String, String)>, return_type: String) -> Self {
        Signature { params, return_type }
    }

    #[must_use] 
    pub fn empty() -> Self {
        Signature {
            params: vec![],
            return_type: "()".to_string(),
        }
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params_str = self
            .params
            .iter()
            .map(|(name, ty)| format!("{name}: {ty}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({}) -> {}", params_str, self.return_type)
    }
}

/// A function call within another function
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCall {
    pub target_name: String,
    pub target_module: Option<String>, // None = unresolved/external
    pub line: usize,
}

impl FunctionCall {
    #[must_use] 
    pub fn new(target_name: String, target_module: Option<String>, line: usize) -> Self {
        FunctionCall {
            target_name,
            target_module,
            line,
        }
    }
}

/// A function definition extracted from source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionDef {
    pub name: String,
    pub signature: Signature,
    pub calls: Vec<FunctionCall>,
    pub module: String,
}

impl FunctionDef {
    #[must_use] 
    pub fn new(name: String, signature: Signature, module: String) -> Self {
        FunctionDef {
            name,
            signature,
            calls: vec![],
            module,
        }
    }

    #[must_use] 
    pub fn with_calls(mut self, calls: Vec<FunctionCall>) -> Self {
        self.calls = calls;
        self
    }

    pub fn add_call(&mut self, call: FunctionCall) {
        self.calls.push(call);
    }

    #[must_use] 
    pub fn fn_id(&self) -> crate::function_id::FunctionId {
        crate::function_id::generate_id(&self.module, &self.name, &self.signature)
    }
}

/// Abstract syntax tree representation of code, language-independent
#[derive(Debug, Clone, Serialize)]
pub struct AbstractAST {
    pub functions: Vec<FunctionDef>,
    pub module_path: String,
}

impl AbstractAST {
    #[must_use] 
    pub fn new(module_path: String) -> Self {
        AbstractAST {
            functions: vec![],
            module_path,
        }
    }

    #[must_use] 
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn add_function(&mut self, func: FunctionDef) {
        self.functions.push(func);
    }

    #[must_use] 
    pub fn module_path(&self) -> &str {
        &self.module_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_display() {
        let sig = Signature::new(
            vec![("x".to_string(), "i32".to_string())],
            "String".to_string(),
        );
        assert_eq!(sig.to_string(), "(x: i32) -> String");
    }

    #[test]
    fn test_signature_empty() {
        let sig = Signature::empty();
        assert_eq!(sig.to_string(), "() -> ()");
    }

    #[test]
    fn test_signature_equality() {
        let sig1 = Signature::new(
            vec![("x".to_string(), "i32".to_string())],
            "String".to_string(),
        );
        let sig2 = Signature::new(
            vec![("x".to_string(), "i32".to_string())],
            "String".to_string(),
        );
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_function_call() {
        let call = FunctionCall::new("foo".to_string(), Some("mod".to_string()), 5);
        assert_eq!(call.target_name, "foo");
        assert_eq!(call.target_module, Some("mod".to_string()));
        assert_eq!(call.line, 5);
    }

    #[test]
    fn test_function_def() {
        let sig = Signature::empty();
        let func = FunctionDef::new("main".to_string(), sig, "root".to_string());
        assert_eq!(func.name, "main");
        assert_eq!(func.module, "root");
    }

    #[test]
    fn test_function_def_with_calls() {
        let sig = Signature::empty();
        let calls = vec![FunctionCall::new("helper".to_string(), None, 10)];
        let func = FunctionDef::new("main".to_string(), sig, "root".to_string())
            .with_calls(calls.clone());
        assert_eq!(func.calls, calls);
    }

    #[test]
    fn test_abstract_ast() {
        let mut ast = AbstractAST::new("mymod".to_string());
        assert_eq!(ast.module_path(), "mymod");

        let sig = Signature::empty();
        let func = FunctionDef::new("foo".to_string(), sig, "mymod".to_string());
        ast.add_function(func);

        assert_eq!(ast.functions.len(), 1);
        assert!(ast.get_function("foo").is_some());
        assert!(ast.get_function("bar").is_none());
    }
}
