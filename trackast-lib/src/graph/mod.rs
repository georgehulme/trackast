use std::collections::HashMap;
use crate::function_id::FunctionId;
use crate::ast::FunctionDef;

/// Node in the call graph representing a function
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: FunctionId,
    pub is_external: bool,
    pub metadata: FunctionDef,
}

impl GraphNode {
    #[must_use] 
    pub fn new(id: FunctionId, metadata: FunctionDef, is_external: bool) -> Self {
        GraphNode {
            id,
            is_external,
            metadata,
        }
    }

    #[must_use] 
    pub fn internal(id: FunctionId, metadata: FunctionDef) -> Self {
        GraphNode {
            id,
            is_external: false,
            metadata,
        }
    }

    #[must_use] 
    pub fn external(id: FunctionId, metadata: FunctionDef) -> Self {
        GraphNode {
            id,
            is_external: true,
            metadata,
        }
    }
}

/// Edge in the call graph representing a function call
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: FunctionId,
    pub to: FunctionId,
    pub line: usize,
}

impl GraphEdge {
    #[must_use] 
    pub fn new(from: FunctionId, to: FunctionId, line: usize) -> Self {
        GraphEdge { from, to, line }
    }
}

/// Call dependency graph
#[derive(Debug, Clone)]
pub struct CallGraph {
    pub nodes: HashMap<FunctionId, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl CallGraph {
    #[must_use] 
    pub fn new() -> Self {
        CallGraph {
            nodes: HashMap::new(),
            edges: vec![],
        }
    }

    /// Insert a node into the graph
    ///
    /// # Errors
    ///
    /// Returns an error if the node already exists in the graph.
    pub fn insert_node(&mut self, node: GraphNode) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("Node already exists: {}", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Insert an edge into the graph
    ///
    /// # Errors
    ///
    /// Returns an error if the from or to node does not exist.
    pub fn insert_edge(&mut self, edge: GraphEdge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.from) {
            return Err(format!("From node does not exist: {}", edge.from));
        }
        if !self.nodes.contains_key(&edge.to) {
            return Err(format!("To node does not exist: {}", edge.to));
        }
        self.edges.push(edge);
        Ok(())
    }

    /// Get a node by ID
    #[must_use] 
    pub fn get_node(&self, id: &FunctionId) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    /// Get all edges originating from a node
    #[must_use] 
    pub fn get_edges_from(&self, id: &FunctionId) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == *id).collect()
    }

    /// Get all edges pointing to a node
    #[must_use] 
    pub fn get_edges_to(&self, id: &FunctionId) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == *id).collect()
    }

    #[must_use] 
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use] 
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Signature;

    fn create_test_node(id: &str) -> (FunctionId, GraphNode) {
        let fn_id = FunctionId::new(id.to_string());
        let func_def = FunctionDef::new("test".to_string(), Signature::empty(), "test".to_string());
        let node = GraphNode::internal(fn_id.clone(), func_def);
        (fn_id, node)
    }

    #[test]
    fn test_graph_node_internal() {
        let (id, node) = create_test_node("test::func::()");
        assert!(!node.is_external);
        assert_eq!(node.id, id);
    }

    #[test]
    fn test_graph_node_external() {
        let (id, _) = create_test_node("test::func::()");
        let func_def = FunctionDef::new("test".to_string(), Signature::empty(), "test".to_string());
        let node = GraphNode::external(id.clone(), func_def);
        assert!(node.is_external);
        assert_eq!(node.id, id);
    }

    #[test]
    fn test_graph_edge() {
        let id1 = FunctionId::new("a::()".to_string());
        let id2 = FunctionId::new("b::()".to_string());
        let edge = GraphEdge::new(id1, id2, 5);
        assert_eq!(edge.line, 5);
    }

    #[test]
    fn test_call_graph_new() {
        let graph = CallGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_insert_node() {
        let mut graph = CallGraph::new();
        let (_, node) = create_test_node("a::()");
        assert!(graph.insert_node(node).is_ok());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_insert_duplicate_node() {
        let mut graph = CallGraph::new();
        let (_, node) = create_test_node("a::()");
        graph.insert_node(node.clone()).unwrap();
        assert!(graph.insert_node(node).is_err());
    }

    #[test]
    fn test_insert_edge() {
        let mut graph = CallGraph::new();
        let (id1, node1) = create_test_node("a::()");
        let (id2, node2) = create_test_node("b::()");
        
        graph.insert_node(node1).unwrap();
        graph.insert_node(node2).unwrap();
        
        let edge = GraphEdge::new(id1, id2, 5);
        assert!(graph.insert_edge(edge).is_ok());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_insert_edge_missing_from() {
        let mut graph = CallGraph::new();
        let (id1, node2) = create_test_node("b::()");
        let (id2, _) = create_test_node("c::()");
        
        graph.insert_node(node2).unwrap();
        
        let edge = GraphEdge::new(id1, id2, 5);
        assert!(graph.insert_edge(edge).is_err());
    }

    #[test]
    fn test_get_node() {
        let mut graph = CallGraph::new();
        let (id, node) = create_test_node("a::()");
        graph.insert_node(node).unwrap();
        
        assert!(graph.get_node(&id).is_some());
    }

    #[test]
    fn test_get_edges_from() {
        let mut graph = CallGraph::new();
        let (id1, node1) = create_test_node("a::()");
        let (id2, node2) = create_test_node("b::()");
        let (id3, node3) = create_test_node("c::()");
        
        graph.insert_node(node1).unwrap();
        graph.insert_node(node2).unwrap();
        graph.insert_node(node3).unwrap();
        
        graph.insert_edge(GraphEdge::new(id1.clone(), id2, 5)).unwrap();
        graph.insert_edge(GraphEdge::new(id1.clone(), id3, 10)).unwrap();
        
        let edges = graph.get_edges_from(&id1);
        assert_eq!(edges.len(), 2);
    }
}
