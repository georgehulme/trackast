use std::collections::{HashSet, VecDeque};
use crate::function_id::FunctionId;
use crate::graph::CallGraph;

/// Result of a graph traversal
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub reachable: HashSet<FunctionId>,
    pub visited_order: Vec<FunctionId>,
}

impl TraversalResult {
    #[must_use] 
    pub fn new() -> Self {
        TraversalResult {
            reachable: HashSet::new(),
            visited_order: vec![],
        }
    }

    pub fn add_node(&mut self, id: FunctionId) {
        if !self.reachable.contains(&id) {
            self.reachable.insert(id.clone());
            self.visited_order.push(id);
        }
    }

    pub fn merge(&mut self, other: TraversalResult) {
        for id in other.visited_order {
            if !self.reachable.contains(&id) {
                self.reachable.insert(id.clone());
                self.visited_order.push(id);
            }
        }
    }
}

impl Default for TraversalResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Depth-first search traversal starting from a single node
#[must_use] 
pub fn dfs_traversal(graph: &CallGraph, start: &FunctionId) -> TraversalResult {
    let mut result = TraversalResult::new();
    let mut stack = vec![start.clone()];
    let mut visited = HashSet::new();

    while let Some(current) = stack.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        result.add_node(current.clone());

        // Add all callees to stack
        for edge in graph.get_edges_from(&current) {
            if !visited.contains(&edge.to) {
                stack.push(edge.to.clone());
            }
        }
    }

    result
}

/// Breadth-first search traversal starting from a single node
#[must_use] 
pub fn bfs_traversal(graph: &CallGraph, start: &FunctionId) -> TraversalResult {
    let mut result = TraversalResult::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(start.clone());

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        result.add_node(current.clone());

        // Add all callees to queue
        for edge in graph.get_edges_from(&current) {
            if !visited.contains(&edge.to) {
                queue.push_back(edge.to.clone());
            }
        }
    }

    result
}

/// Traverse from multiple entry points
#[must_use] 
pub fn traversal_from_entries(
    graph: &CallGraph,
    entries: &[FunctionId],
) -> TraversalResult {
    let mut result = TraversalResult::new();

    for entry in entries {
        let entry_result = dfs_traversal(graph, entry);
        result.merge(entry_result);
    }

    result
}

/// Visitor trait for custom traversal logic
pub trait Visitor {
    fn visit(&mut self, node_id: &FunctionId);
}

/// DFS traversal with a visitor
pub fn dfs_with_visitor(
    graph: &CallGraph,
    start: &FunctionId,
    visitor: &mut dyn Visitor,
) -> TraversalResult {
    let mut result = TraversalResult::new();
    let mut stack = vec![start.clone()];
    let mut visited = HashSet::new();

    while let Some(current) = stack.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        result.add_node(current.clone());
        visitor.visit(&current);

        // Add all callees to stack
        for edge in graph.get_edges_from(&current) {
            if !visited.contains(&edge.to) {
                stack.push(edge.to.clone());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FunctionDef, Signature};
    use crate::graph::{GraphNode, GraphEdge};

    fn create_graph_with_edges() -> (CallGraph, FunctionId, FunctionId, FunctionId) {
        let mut graph = CallGraph::new();

        let id_a = FunctionId::new("a::()".to_string());
        let id_b = FunctionId::new("b::()".to_string());
        let id_c = FunctionId::new("c::()".to_string());

        let func_a = FunctionDef::new("a".to_string(), Signature::empty(), "root".to_string());
        let func_b = FunctionDef::new("b".to_string(), Signature::empty(), "root".to_string());
        let func_c = FunctionDef::new("c".to_string(), Signature::empty(), "root".to_string());

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
            .insert_edge(GraphEdge::new(id_a.clone(), id_b.clone(), 1))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_b.clone(), id_c.clone(), 2))
            .unwrap();

        (graph, id_a, id_b, id_c)
    }

    #[test]
    fn test_traversal_result_new() {
        let result = TraversalResult::new();
        assert_eq!(result.reachable.len(), 0);
        assert_eq!(result.visited_order.len(), 0);
    }

    #[test]
    fn test_traversal_result_add_node() {
        let mut result = TraversalResult::new();
        let id = FunctionId::new("a::()".to_string());
        result.add_node(id.clone());
        assert_eq!(result.reachable.len(), 1);
        assert_eq!(result.visited_order.len(), 1);
    }

    #[test]
    fn test_dfs_single_node() {
        let (graph, id_a, _, _) = create_graph_with_edges();
        let result = dfs_traversal(&graph, &id_a);
        assert_eq!(result.reachable.len(), 3);
        assert!(result.reachable.contains(&id_a));
    }

    #[test]
    fn test_dfs_from_middle() {
        let (graph, _, id_b, id_c) = create_graph_with_edges();
        let result = dfs_traversal(&graph, &id_b);
        assert_eq!(result.reachable.len(), 2);
        assert!(result.reachable.contains(&id_b));
        assert!(result.reachable.contains(&id_c));
    }

    #[test]
    fn test_bfs_single_node() {
        let (graph, id_a, _, _) = create_graph_with_edges();
        let result = bfs_traversal(&graph, &id_a);
        assert_eq!(result.reachable.len(), 3);
        assert!(result.reachable.contains(&id_a));
    }

    #[test]
    fn test_traversal_from_entries() {
        let (graph, id_a, id_b, _) = create_graph_with_edges();
        let entries = vec![id_a, id_b];
        let result = traversal_from_entries(&graph, &entries);
        assert_eq!(result.reachable.len(), 3);
    }

    #[test]
    fn test_dfs_with_cycle() {
        let mut graph = CallGraph::new();

        let id_a = FunctionId::new("a::()".to_string());
        let id_b = FunctionId::new("b::()".to_string());

        let func_a = FunctionDef::new("a".to_string(), Signature::empty(), "root".to_string());
        let func_b = FunctionDef::new("b".to_string(), Signature::empty(), "root".to_string());

        graph
            .insert_node(GraphNode::internal(id_a.clone(), func_a))
            .unwrap();
        graph
            .insert_node(GraphNode::internal(id_b.clone(), func_b))
            .unwrap();

        // Create cycle: a -> b -> a
        graph
            .insert_edge(GraphEdge::new(id_a.clone(), id_b.clone(), 1))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_b.clone(), id_a.clone(), 2))
            .unwrap();

        let result = dfs_traversal(&graph, &id_a);
        assert_eq!(result.reachable.len(), 2);
    }

    struct CountingVisitor {
        count: usize,
    }

    impl Visitor for CountingVisitor {
        fn visit(&mut self, _: &FunctionId) {
            self.count += 1;
        }
    }

    #[test]
    fn test_dfs_with_visitor() {
        let (graph, id_a, _, _) = create_graph_with_edges();
        let mut visitor = CountingVisitor { count: 0 };
        let result = dfs_with_visitor(&graph, &id_a, &mut visitor);
        assert_eq!(visitor.count, 3);
        assert_eq!(result.reachable.len(), 3);
    }
}
