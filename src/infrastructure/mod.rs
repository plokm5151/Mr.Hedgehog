use crate::domain::ast::{AstNode, AstNodeKind};
use crate::domain::callgraph::*;
use crate::ports::{AstParser, CallGraphBuilder, OutputExporter};
use syn::{File, Item, Expr, Stmt};
use std::collections::HashMap;

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
    fn build_call_graph(&self, src: &str) -> CallGraph {
        let ast_file: File = syn::parse_file(src).unwrap();
        let mut callees_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_funcs = vec![];

        for item in ast_file.items {
            if let Item::Fn(ref func) = item {
                let name = func.sig.ident.to_string();
                all_funcs.push(name.clone());
                let mut callees = vec![];
                visit_stmts(&func.block.stmts, &mut callees);
                println!("[Debug] fn: {}, callees: {:?}", name, callees); // ← debug
                callees_map.insert(name, callees);
            }
        }

        let nodes = all_funcs.iter().map(|f| CallGraphNode {
            id: f.clone(),
            callees: callees_map.remove(f).unwrap_or_default(),
        }).collect();
        CallGraph { nodes }
    }
}

fn visit_stmts(stmts: &[Stmt], callees: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr, _) => {
                visit_expr(expr, callees);
            }
            _ => {}
        }
    }
}

fn visit_expr(expr: &Expr, callees: &mut Vec<String>) {
    match expr {
        Expr::Call(expr_call) => {
            if let Expr::Path(ref expr_path) = *expr_call.func {
                if let Some(ident) = expr_path.path.get_ident() {
                    callees.push(ident.to_string());
                }
            }
            for arg in &expr_call.args {
                visit_expr(arg, callees);
            }
        }
        Expr::If(expr_if) => {
            visit_expr(&expr_if.cond, callees);
            visit_stmts(&expr_if.then_branch.stmts, callees);
            if let Some((_, else_branch)) = &expr_if.else_branch {
                visit_expr(else_branch, callees);
            }
        }
        Expr::Block(expr_block) => {
            visit_stmts(&expr_block.block.stmts, callees);
        }
        Expr::While(expr_while) => {
            visit_expr(&expr_while.cond, callees);
            visit_stmts(&expr_while.body.stmts, callees);
        }
        Expr::ForLoop(expr_for) => {
            visit_expr(&expr_for.expr, callees);
            visit_stmts(&expr_for.body.stmts, callees);
        }
        Expr::Match(expr_match) => {
            visit_expr(&expr_match.expr, callees);
            for arm in &expr_match.arms {
                visit_expr(&arm.body, callees);
            }
        }
        _ => {}
    }
}

pub struct DotExporter;
impl OutputExporter for DotExporter {
    fn export(&self, graph: &CallGraph, path: &str) -> std::io::Result<()> {
        let mut dot_lines = vec!["digraph G {".to_string()];
        for node in &graph.nodes {
            dot_lines.push(format!("    {};", node.id));
            for callee in &node.callees {
                dot_lines.push(format!("    {} -> {};", node.id, callee));
            }
        }
        dot_lines.push("}".to_string());
        std::fs::write(path, dot_lines.join("\n"))
    }
}
