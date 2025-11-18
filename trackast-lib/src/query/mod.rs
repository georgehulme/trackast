use std::collections::HashSet;
use crate::function_id::FunctionId;
use crate::graph::CallGraph;
use crate::traversal::dfs_traversal;

/// Query interface for call graph analysis
pub trait GraphQuery {
    /// Get all functions reachable from the given function
    ///
    /// # Errors
    ///
    /// Returns an error if the function is not found in the graph.
    fn reachable_from(&self, id: &FunctionId) -> Result<HashSet<FunctionId>, String>;
    fn direct_callers(&self, id: &FunctionId) -> Vec<FunctionId>;
    fn direct_callees(&self, id: &FunctionId) -> Vec<FunctionId>;
    fn get_function(&self, id: &FunctionId) -> Option<&crate::graph::GraphNode>;
    fn external_calls(&self) -> Vec<&crate::graph::GraphEdge>;
}

impl GraphQuery for CallGraph {
    /// Get all functions reachable from the given function
    fn reachable_from(&self, id: &FunctionId) -> Result<HashSet<FunctionId>, String> {
        if !self.nodes.contains_key(id) {
            return Err(format!("Function not found: {id}"));
        }
        let result = dfs_traversal(self, id);
        Ok(result.reachable)
    }

    /// Get all functions that directly call the given function
    fn direct_callers(&self, id: &FunctionId) -> Vec<FunctionId> {
        self.get_edges_to(id)
            .iter()
            .map(|e| e.from.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get all functions directly called by the given function
    fn direct_callees(&self, id: &FunctionId) -> Vec<FunctionId> {
        self.get_edges_from(id)
            .iter()
            .map(|e| e.to.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get a node by ID
    fn get_function(&self, id: &FunctionId) -> Option<&crate::graph::GraphNode> {
        self.get_node(id)
    }

    /// Get all edges pointing to external nodes
    fn external_calls(&self) -> Vec<&crate::graph::GraphEdge> {
        self.edges
            .iter()
            .filter(|e| {
                self.nodes
                    .get(&e.to)
                    .is_some_and(|n| n.is_external)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FunctionDef, Signature};
    use crate::graph::{GraphNode, GraphEdge};

    fn create_test_graph() -> CallGraph {
        let mut graph = CallGraph::new();

        let id_a = FunctionId::new("a::()".to_string());
        let id_b = FunctionId::new("b::()".to_string());
        let id_c = FunctionId::new("c::()".to_string());
        let id_ext = FunctionId::new("<external>::ext::()".to_string());

        let func_a = FunctionDef::new("a".to_string(), Signature::empty(), "root".to_string());
        let func_b = FunctionDef::new("b".to_string(), Signature::empty(), "root".to_string());
        let func_c = FunctionDef::new("c".to_string(), Signature::empty(), "root".to_string());
        let func_ext = FunctionDef::new("ext".to_string(), Signature::empty(), "ext".to_string());

        graph
            .insert_node(GraphNode::internal(id_a.clone(), func_a))
            .unwrap();
        graph
            .insert_node(GraphNode::internal(id_b.clone(), func_b))
            .unwrap();
        graph
            .insert_node(GraphNode::internal(id_c.clone(), func_c))
            .unwrap();
        graph
            .insert_node(GraphNode::external(id_ext.clone(), func_ext))
            .unwrap();

        graph
            .insert_edge(GraphEdge::new(id_a.clone(), id_b.clone(), 1))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_b.clone(), id_c.clone(), 2))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_c.clone(), id_ext.clone(), 3))
            .unwrap();

        graph
    }

    #[test]
    fn test_reachable_from_start() {
        let graph = create_test_graph();
        let id_a = FunctionId::new("a::()".to_string());
        let reachable = graph.reachable_from(&id_a).unwrap();
        assert_eq!(reachable.len(), 4); // a, b, c, ext
    }

    #[test]
    fn test_reachable_from_middle() {
        let graph = create_test_graph();
        let id_b = FunctionId::new("b::()".to_string());
        let reachable = graph.reachable_from(&id_b).unwrap();
        assert_eq!(reachable.len(), 3); // b, c, ext
    }

    #[test]
    fn test_reachable_from_nonexistent() {
        let graph = create_test_graph();
        let id_missing = FunctionId::new("missing::()".to_string());
        assert!(graph.reachable_from(&id_missing).is_err());
    }

    #[test]
    fn test_direct_callers() {
        let graph = create_test_graph();
        let id_b = FunctionId::new("b::()".to_string());
        let callers = graph.direct_callers(&id_b);
        assert_eq!(callers.len(), 1);
        assert!(callers.contains(&FunctionId::new("a::()".to_string())));
    }

    #[test]
    fn test_direct_callees() {
        let graph = create_test_graph();
        let id_a = FunctionId::new("a::()".to_string());
        let callees = graph.direct_callees(&id_a);
        assert_eq!(callees.len(), 1);
        assert!(callees.contains(&FunctionId::new("b::()".to_string())));
    }

    #[test]
    fn test_get_function() {
        let graph = create_test_graph();
        let id_a = FunctionId::new("a::()".to_string());
        let node = graph.get_function(&id_a);
        assert!(node.is_some());
    }

    #[test]
    fn test_external_calls() {
        let graph = create_test_graph();
        let external = graph.external_calls();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].to, FunctionId::new("<external>::ext::()".to_string()));
    }
}
