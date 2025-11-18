use crate::graph::CallGraph;
use std::fmt::Write as _;

/// Generate Graphviz DOT format for the call graph
#[must_use] 
pub fn to_dot(graph: &CallGraph) -> String {
    let mut output = String::new();
    output.push_str("digraph CallGraph {\n");
    output.push_str("    rankdir=LR;\n");
    output.push_str("    node [shape=box];\n\n");

    // Add nodes
    for (id, node) in &graph.nodes {
        let style = if node.is_external {
            ", style=filled, fillcolor=lightgray"
        } else {
            ", style=filled, fillcolor=lightblue"
        };
        
        // Format label: replace :: with newline for readability
        let label = id.as_str().replace("::", "\n");
        let _ = writeln!(
            output,
            "    \"{}\" [label=\"{}\"{}];",
            id.as_str(),
            label,
            style
        );
    }

    output.push('\n');

    // Add edges with line number labels
    for edge in &graph.edges {
        let label = if edge.line > 0 {
            format!(", label=\"L{}\"", edge.line)
        } else {
            String::new()
        };
        
        let _ = writeln!(
            output,
            "    \"{}\" -> \"{}\"{};",
            edge.from.as_str(),
            edge.to.as_str(),
            label
        );
    }

    output.push_str("}\n");
    output
}

/// Write DOT format to a file
///
/// # Errors
///
/// Returns an I/O error if writing to the file fails.
pub fn to_dot_file(graph: &CallGraph, path: &str) -> std::io::Result<()> {
    std::fs::write(path, to_dot(graph))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FunctionDef, Signature};
    use crate::function_id::FunctionId;
    use crate::graph::{GraphNode, GraphEdge};

    #[test]
    fn test_to_dot_empty() {
        let graph = CallGraph::new();
        let dot = to_dot(&graph);
        assert!(dot.contains("digraph CallGraph"));
        assert!(dot.contains("rankdir=LR"));
    }

    #[test]
    fn test_to_dot_with_internal_node() {
        let mut graph = CallGraph::new();
        let id = FunctionId::new("root::main::() -> ()".to_string());
        let func = FunctionDef::new("main".to_string(), Signature::empty(), "root".to_string());
        let node = GraphNode::internal(id, func);
        graph.insert_node(node).unwrap();

        let dot = to_dot(&graph);
        assert!(dot.contains("root::main::() -> ()"));
        assert!(dot.contains("fillcolor=lightblue"));
    }

    #[test]
    fn test_to_dot_with_external_node() {
        let mut graph = CallGraph::new();
        let id = FunctionId::new("<external>::println::()".to_string());
        let func = FunctionDef::new("println".to_string(), Signature::empty(), "<external>".to_string());
        let node = GraphNode::external(id, func);
        graph.insert_node(node).unwrap();

        let dot = to_dot(&graph);
        assert!(dot.contains("fillcolor=lightgray"));
    }

    #[test]
    fn test_to_dot_with_edge() {
        let mut graph = CallGraph::new();

        let id1 = FunctionId::new("a::()".to_string());
        let id2 = FunctionId::new("b::()".to_string());

        let func1 = FunctionDef::new("a".to_string(), Signature::empty(), "root".to_string());
        let func2 = FunctionDef::new("b".to_string(), Signature::empty(), "root".to_string());

        graph.insert_node(GraphNode::internal(id1.clone(), func1)).unwrap();
        graph.insert_node(GraphNode::internal(id2.clone(), func2)).unwrap();
        graph.insert_edge(GraphEdge::new(id1, id2, 5)).unwrap();

        let dot = to_dot(&graph);
        assert!(dot.contains("->"));
        assert!(dot.contains("L5"));
    }

    #[test]
    fn test_to_dot_newline_formatting() {
        let mut graph = CallGraph::new();
        let id = FunctionId::new("my_crate::utils::helpers::process::() -> String".to_string());
        let func = FunctionDef::new("process".to_string(), Signature::empty(), "my_crate::utils::helpers".to_string());
        let node = GraphNode::internal(id, func);
        graph.insert_node(node).unwrap();

        let dot = to_dot(&graph);
        // Should contain newlines in the label
        assert!(dot.contains("my_crate\nutils\nhelpers\nprocess\n() -> String"));
    }

    #[test]
    fn test_to_dot_file() {
        let graph = CallGraph::new();
        let temp_file = "/tmp/test_callgraph.dot";
        
        let result = to_dot_file(&graph, temp_file);
        assert!(result.is_ok());
        
        if let Ok(contents) = std::fs::read_to_string(temp_file) {
            assert!(contents.contains("digraph CallGraph"));
        }
        
        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }
}
