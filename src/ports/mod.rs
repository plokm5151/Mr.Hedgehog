use crate::domain::ast::AstNode;
use crate::domain::callgraph::CallGraph;

pub trait AstParser {
    fn parse(&self, src: &str) -> AstNode;
}

pub trait CallGraphBuilder {
    fn build_call_graph(&self, src: &str) -> CallGraph;
}

pub trait OutputExporter {
    fn export(&self, graph: &CallGraph, path: &str) -> std::io::Result<()>;
}
