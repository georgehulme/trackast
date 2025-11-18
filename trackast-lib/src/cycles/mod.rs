use std::collections::{HashSet, VecDeque};
use crate::function_id::FunctionId;
use crate::graph::CallGraph;

/// Represents a cycle in the call graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    pub nodes: Vec<FunctionId>,
}

impl Cycle {
    #[must_use] 
    pub fn new(nodes: Vec<FunctionId>) -> Self {
        Cycle { nodes }
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Find all cycles in the call graph using BFS-based cycle detection
#[must_use] 
pub fn find_cycles(graph: &CallGraph) -> Vec<Cycle> {
    let mut cycles = Vec::new();
    let mut visited_global = HashSet::new();

    for start_node in graph.nodes.keys() {
        if visited_global.contains(start_node) {
            continue;
        }

        // Check for self-cycles
        for edge in graph.get_edges_from(start_node) {
            if edge.to == *start_node {
                cycles.push(Cycle::new(vec![start_node.clone()]));
            }
        }

        // BFS to find paths from start_node back to itself (length > 1)
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        
        queue.push_back((start_node.clone(), vec![start_node.clone()]));
        visited.insert(start_node.clone());

        while let Some((current, path)) = queue.pop_front() {
            for edge in graph.get_edges_from(&current) {
                if edge.to == *start_node && path.len() > 1 {
                    // Found a cycle back to start
                    cycles.push(Cycle::new(path.clone()));
                } else if !visited.contains(&edge.to) && path.len() < graph.nodes.len() {
                    visited.insert(edge.to.clone());
                    let mut new_path = path.clone();
                    new_path.push(edge.to.clone());
                    queue.push_back((edge.to.clone(), new_path));
                }
            }
        }

        visited_global.insert(start_node.clone());
    }

    // Remove duplicate cycles
    cycles.sort_by_key(|c| c.nodes.clone());
    cycles.dedup();
    cycles
}

/// Check if the graph has any cycles
#[must_use] 
pub fn has_cycles(graph: &CallGraph) -> bool {
    !find_cycles(graph).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FunctionDef, Signature};
    use crate::graph::{GraphNode, GraphEdge};

    #[test]
    fn test_cycle_creation() {
        let cycle = Cycle::new(vec![
            FunctionId::new("a::()".to_string()),
            FunctionId::new("b::()".to_string()),
        ]);
        assert_eq!(cycle.len(), 2);
        assert!(!cycle.is_empty());
    }

    #[test]
    fn test_no_cycles() {
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
        graph
            .insert_edge(GraphEdge::new(id_a, id_b, 1))
            .unwrap();

        let cycles = find_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_self_cycle() {
        let mut graph = CallGraph::new();

        let id_a = FunctionId::new("a::()".to_string());
        let func_a = FunctionDef::new("a".to_string(), Signature::empty(), "root".to_string());

        graph
            .insert_node(GraphNode::internal(id_a.clone(), func_a))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_a.clone(), id_a.clone(), 1))
            .unwrap();

        let cycles = find_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_two_node_cycle() {
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

        let cycles = find_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_three_node_cycle() {
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

        // Create cycle: a -> b -> c -> a
        graph
            .insert_edge(GraphEdge::new(id_a.clone(), id_b.clone(), 1))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_b.clone(), id_c.clone(), 2))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_c.clone(), id_a.clone(), 3))
            .unwrap();

        let cycles = find_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_has_cycles() {
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
        graph
            .insert_edge(GraphEdge::new(id_a.clone(), id_b.clone(), 1))
            .unwrap();
        graph
            .insert_edge(GraphEdge::new(id_b.clone(), id_a.clone(), 2))
            .unwrap();

        assert!(has_cycles(&graph));
    }
}
