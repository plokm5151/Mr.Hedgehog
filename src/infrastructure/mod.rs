// Infrastructure implementations for TraceCraft.

use crate::domain::ast::{AstNode, AstNodeKind};
use crate::domain::callgraph::*;
use crate::ports::{AstParser, CallGraphBuilder, OutputExporter};
use syn::{File, Item};

pub struct SynAstParser;
impl AstParser for SynAstParser {
    fn parse(&self, src: &str) -> AstNode {
        let ast_file: File = match syn::parse_file(src) {
            Ok(file) => file,
            Err(_) => return AstNode {
                kind: AstNodeKind::Module,
                children: vec![],
            },
        };

        let mut children = Vec::new();
        for item in ast_file.items {
            if let Item::Fn(_func) = item {
                children.push(AstNode {
                    kind: AstNodeKind::Function,
                    children: vec![],
                });
            }
        }
        AstNode {
            kind: AstNodeKind::Module,
            children,
        }
    }
}

pub struct SimpleCallGraphBuilder;
impl CallGraphBuilder for SimpleCallGraphBuilder {
    fn build_call_graph(&self, _root: &AstNode) -> CallGraph {
        CallGraph { nodes: vec![] }
    }
}

pub struct DotExporter;
impl OutputExporter for DotExporter {
    fn export(&self, data: &str, path: &str) -> std::io::Result<()> {
        std::fs::write(path, data)
    }
}
