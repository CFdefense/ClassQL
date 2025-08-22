use petgraph::dot::Dot;
use petgraph::graph::DiGraph;
use petgraph::stable_graph::NodeIndex;

use crate::compiler::parser::{Ast, TreeNode};
pub fn ast_to_dot(input_string: String, ast: &Ast) -> String {
    let mut graph: DiGraph<String, ()> = DiGraph::new();

    let ast_head_idx = graph.add_node("AST Head".to_string());

    if let Some(ref head_node) = ast.head {
        add_tree_nodes_recursive(input_string, &mut graph, ast_head_idx, head_node);
    }

    // Generate DOT output
    format!("{:?}", Dot::new(&graph)).replace("label = \"()\"", "label = \"\"")
}

/// Recursive helper function to add TreeNodes and their children to the graph.
fn add_tree_nodes_recursive(
    input_string: String,
    graph: &mut DiGraph<String, ()>,
    parent_idx: NodeIndex,
    tree_node: &TreeNode,
) {
    // Add the current TreeNode's NodeType as a node in the graph
    // if it has a lexeme add that or add the production name
    let current_node_label = match tree_node.lexical_token {
        Some(t) => format!(
            "{}=`{}`",
            tree_node.node_type,
            &input_string[t.get_start()..t.get_end()],
        ),
        None => tree_node.node_type.to_string(),
    };

    let current_idx = graph.add_node(current_node_label);

    graph.add_edge(parent_idx, current_idx, ());

    // Add the rest of them
    for child in &tree_node.children {
        add_tree_nodes_recursive(input_string.clone(), graph, current_idx, child);
    }
}
