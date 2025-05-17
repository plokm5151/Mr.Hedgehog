use crate::domain::ast::AstNode;
use crate::domain::callgraph::CallGraph;

// AST Parser: Parse source code string to AST node tree
pub trait AstParser {
    fn parse(&self, src: &str) -> AstNode;
}

// CallGraph Builder: Build call graph from AST node
pub trait CallGraphBuilder {
    fn build_call_graph(&self, root: &AstNode) -> CallGraph;
}

// OutputExporter: Export call graph to disk in specific format
pub trait OutputExporter {
    fn export(&self, graph: &CallGraph, path: &str) -> std::io::Result<()>;
}
