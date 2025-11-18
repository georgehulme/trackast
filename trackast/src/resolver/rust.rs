use trackast_lib::ast::FunctionDef;

/// Try to resolve a function call to a local function definition
/// Returns (module, name) if found
#[must_use] 
pub fn resolve_call(
    call_name: &str,
    current_module: &str,
    all_functions: &[FunctionDef],
) -> Option<(String, String)> {
    // First, try to find in current module
    for func in all_functions {
        if func.name == call_name && func.module == current_module {
            return Some((func.module.clone(), func.name.clone()));
        }
    }

    // Then try parent modules
    let parts: Vec<&str> = current_module.split("::").collect();
    for i in (1..parts.len()).rev() {
        let parent_module = parts[0..i].join("::");
        for func in all_functions {
            if func.name == call_name && func.module == parent_module {
                return Some((func.module.clone(), func.name.clone()));
            }
        }
    }

    // Try root module
    for func in all_functions {
        if func.name == call_name && func.module.is_empty() {
            return Some((func.module.clone(), func.name.clone()));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use trackast_lib::ast::Signature;

    fn create_test_function(name: &str, module: &str) -> FunctionDef {
        FunctionDef::new(
            name.to_string(),
            Signature::empty(),
            module.to_string(),
        )
    }

    #[test]
    fn test_resolve_in_current_module() {
        let funcs = vec![
            create_test_function("helper", "root"),
            create_test_function("main", "root"),
        ];

        let result = resolve_call("helper", "root", &funcs);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ("root".to_string(), "helper".to_string()));
    }

    #[test]
    fn test_resolve_in_parent_module() {
        let funcs = vec![
            create_test_function("helper", "root"),
            create_test_function("main", "root::utils"),
        ];

        let result = resolve_call("helper", "root::utils", &funcs);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ("root".to_string(), "helper".to_string()));
    }

    #[test]
    fn test_resolve_not_found() {
        let funcs = vec![create_test_function("main", "root")];

        let result = resolve_call("missing", "root", &funcs);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_nested_module() {
        let funcs = vec![
            create_test_function("util", "root::nested::deep"),
            create_test_function("main", "root::nested::deep::deeper"),
        ];

        let result = resolve_call("util", "root::nested::deep::deeper", &funcs);
        assert!(result.is_some());
    }
}
