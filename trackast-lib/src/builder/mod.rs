use std::collections::HashMap;
use crate::ast::{AbstractAST, FunctionDef};
use crate::function_id::FunctionId;
use crate::graph::{CallGraph, GraphNode, GraphEdge};
use crate::traversal::{dfs_traversal, TraversalResult};

/// Builder for constructing a call graph from ASTs
pub struct CallGraphBuilder {
    asts: Vec<AbstractAST>,
    functions_map: HashMap<FunctionId, FunctionDef>,
}

impl CallGraphBuilder {
    #[must_use] 
    pub fn new() -> Self {
        CallGraphBuilder {
            asts: vec![],
            functions_map: HashMap::new(),
        }
    }

    /// Add an abstract syntax tree to the builder
    ///
    /// # Errors
    ///
    /// Returns an error if a duplicate function ID is encountered.
    pub fn add_ast(&mut self, ast: AbstractAST) -> Result<(), String> {
        for func in &ast.functions {
            let fn_id = func.fn_id();
            if self.functions_map.contains_key(&fn_id) {
                return Err(format!("Duplicate function ID: {fn_id}"));
            }
            self.functions_map.insert(fn_id, func.clone());
        }
        self.asts.push(ast);
        Ok(())
    }

    /// Build the complete call graph
    ///
    /// # Errors
    ///
    /// Returns an error if graph construction fails.
    pub fn build(&self) -> Result<CallGraph, String> {
        let mut graph = CallGraph::new();

        // Add all nodes
        for (fn_id, func_def) in &self.functions_map {
            let node = GraphNode::internal(fn_id.clone(), func_def.clone());
            graph.insert_node(node)?;
        }

        // Add edges based on calls, marking unresolved calls as external
        for func_def in self.functions_map.values() {
            let from_id = func_def.fn_id();

            for call in &func_def.calls {
                // Try to resolve the call
                let to_id = if let Some(target_module) = &call.target_module {
                    crate::function_id::generate_id(target_module, &call.target_name, &crate::ast::Signature::empty())
                } else {
                    // Unresolved call - create external node
                    let external_id = FunctionId::new(format!(
                        "<external>::{}::{}",
                        call.target_name, "()"
                    ));
                    
                    // Add external node if it doesn't exist
                    if !graph.nodes.contains_key(&external_id) {
                        let external_func = FunctionDef::new(
                            call.target_name.clone(),
                            crate::ast::Signature::empty(),
                            "<external>".to_string(),
                        );
                        let external_node = GraphNode::external(external_id.clone(), external_func);
                        graph.insert_node(external_node)?;
                    }
                    
                    external_id
                };

                // Check if target exists, if not add it as external
                if !graph.nodes.contains_key(&to_id) && !to_id.as_str().starts_with("<external>") {
                    let external_func = FunctionDef::new(
                        call.target_name.clone(),
                        crate::ast::Signature::empty(),
                        "<external>".to_string(),
                    );
                    let external_node = GraphNode::external(to_id.clone(), external_func);
                    graph.insert_node(external_node)?;
                }

                // Add edge
                let edge = GraphEdge::new(from_id.clone(), to_id, call.line);
                graph.insert_edge(edge)?;
            }
        }

        Ok(graph)
    }

    /// Build graph and traverse from entry points
    ///
    /// # Errors
    ///
    /// Returns an error if graph construction fails or entry points are not found.
    pub fn build_from_entries(
        &self,
        entries: &[FunctionId],
    ) -> Result<(CallGraph, TraversalResult), String> {
        let graph = self.build()?;
        let mut result = TraversalResult::new();

        for entry in entries {
            if !graph.nodes.contains_key(entry) {
                return Err(format!("Entry point not found: {entry}"));
            }
            let entry_result = dfs_traversal(&graph, entry);
            result.merge(entry_result);
        }

        Ok((graph, result))
    }
}

impl Default for CallGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Signature, FunctionCall};

    #[test]
    fn test_builder_new() {
        let builder = CallGraphBuilder::new();
        assert_eq!(builder.asts.len(), 0);
        assert_eq!(builder.functions_map.len(), 0);
    }

    #[test]
    fn test_add_ast() {
        let mut builder = CallGraphBuilder::new();
        let mut ast = AbstractAST::new("root".to_string());
        let func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        ast.add_function(func);

        assert!(builder.add_ast(ast).is_ok());
        assert_eq!(builder.functions_map.len(), 1);
    }

    #[test]
    fn test_add_duplicate_function() {
        let mut builder = CallGraphBuilder::new();
        let mut ast1 = AbstractAST::new("root".to_string());
        let func1 = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        ast1.add_function(func1);
        builder.add_ast(ast1).unwrap();

        let mut ast2 = AbstractAST::new("root".to_string());
        let func2 = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        ast2.add_function(func2);

        assert!(builder.add_ast(ast2).is_err());
    }

    #[test]
    fn test_build_simple() {
        let mut builder = CallGraphBuilder::new();
        let mut ast = AbstractAST::new("root".to_string());
        let func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        ast.add_function(func);
        builder.add_ast(ast).unwrap();

        let graph = builder.build().unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_build_with_call() {
        let mut builder = CallGraphBuilder::new();
        let mut ast = AbstractAST::new("root".to_string());

        let mut main_func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        main_func.add_call(FunctionCall::new("helper".to_string(), Some("root".to_string()), 5));

        let helper_func = FunctionDef::new("helper".to_string(), Signature::empty(), "root".to_string());

        ast.add_function(main_func);
        ast.add_function(helper_func);
        builder.add_ast(ast).unwrap();

        let graph = builder.build().unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_build_with_external_call() {
        let mut builder = CallGraphBuilder::new();
        let mut ast = AbstractAST::new("root".to_string());

        let mut main_func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        main_func.add_call(FunctionCall::new("println".to_string(), None, 5));

        ast.add_function(main_func);
        builder.add_ast(ast).unwrap();

        let graph = builder.build().unwrap();
        assert_eq!(graph.node_count(), 2); // main + external println
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_build_from_entries() {
        let mut builder = CallGraphBuilder::new();
        let mut ast = AbstractAST::new("root".to_string());

        let mut main_func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        main_func.add_call(FunctionCall::new("helper".to_string(), Some("root".to_string()), 5));

        let helper_func = FunctionDef::new("helper".to_string(), Signature::empty(), "root".to_string());

        ast.add_function(main_func);
        ast.add_function(helper_func);
        builder.add_ast(ast).unwrap();

        let main_id = FunctionId::new("root::main::() -> ()".to_string());
        let (graph, result) = builder.build_from_entries(&[main_id]).unwrap();

        assert_eq!(graph.node_count(), 2);
        assert_eq!(result.reachable.len(), 2);
    }

    #[test]
    fn test_build_from_entries_nonexistent() {
        let builder = CallGraphBuilder::new();
        let missing_id = FunctionId::new("missing::()".to_string());
        assert!(builder.build_from_entries(&[missing_id]).is_err());
    }
}
