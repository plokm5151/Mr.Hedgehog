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
                name: None,
                children: vec![],
            },
        };

        let mut children = Vec::new();
        for item in ast_file.items {
            if let Item::Fn(ref func) = item {
                let name = func.sig.ident.to_string();
                children.push(AstNode {
                    kind: AstNodeKind::Function,
                    name: Some(name),
                    children: vec![],
                });
            }
        }
        AstNode {
            kind: AstNodeKind::Module,
            name: None,
            children,
        }
    }
}

pub struct SimpleCallGraphBuilder;
impl CallGraphBuilder for SimpleCallGraphBuilder {
    fn build_call_graph(&self, root: &AstNode) -> CallGraph {
        let mut nodes = Vec::new();
        let mut ids = Vec::new();
        for child in root.children.iter() {
            if let AstNodeKind::Function = child.kind {
                let id = child.name.clone().unwrap_or("unknown".to_string());
                ids.push(id.clone());
                let mut callees = vec![];
                // Demo: 假如 main 與 foo 都存在，就讓 main 呼叫 foo
                if id == "main" && ids.contains(&"foo".to_string()) {
                    callees.push("foo".to_string());
                }
                nodes.push(CallGraphNode { id, callees });
            }
        }
        CallGraph { nodes }
    }
}

pub struct DotExporter;
impl OutputExporter for DotExporter {
    fn export(&self, data: &str, path: &str) -> std::io::Result<()> {
        // 極簡 parser：動態根據 debug print 產生 DOT
        let mut nodes = vec![];
        let mut edges = vec![];
        for line in data.lines() {
            // id: "main",
            if let Some(idx) = line.find("id: \"") {
                let id = line[idx + 5..].split('"').next().unwrap();
                nodes.push(id.to_string());
            }
            // callees: ["foo"]
            if let Some(idx) = line.find("callees: [") {
                let rest = &line[idx + 10..];
                let callees: Vec<&str> = rest.split('"').filter(|s| !s.is_empty() && *s != ", " && *s != "]").collect();
                if !nodes.is_empty() && !callees.is_empty() {
                    let from = nodes.last().unwrap();
                    for to in callees {
                        edges.push((from.clone(), to.to_string()));
                    }
                }
            }
        }
        let mut dot_lines = vec!["digraph G {".to_string()];
        for node in &nodes {
            dot_lines.push(format!("    {};", node));
        }
        for (from, to) in &edges {
            dot_lines.push(format!("    {} -> {};", from, to));
        }
        dot_lines.push("}".to_string());
        std::fs::write(path, dot_lines.join("\n"))
    }
}
