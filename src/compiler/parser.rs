/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType as NodeType};

pub struct AST {
    head: TreeNode,
}

struct TreeNode {
    children: Vec<TreeNode>,
    node_type: NodeType,
    node_content: String,
}

pub fn parse(ast: &AST, tokens: Vec<Token>) {
    
}