use crate::ports::{CallGraphBuilder, OutputExporter};
use crate::domain::callgraph::{CallGraph, CallGraphNode};
use syn::{File, Item, Expr, Stmt};
use std::collections::HashMap;
use std::fs;

pub struct DotExporter;
impl OutputExporter for DotExporter {
    fn export(&self, cg: &CallGraph, path: &str) -> std::io::Result<()> {
        let mut lines = vec!["digraph G {".to_string()];
        for node in &cg.nodes {
            lines.push(format!("    \"{}\";", node.id));
            for callee in &node.callees {
                lines.push(format!("    \"{}\" -> \"{}\";", node.id, callee));
            }
        }
        lines.push("}".to_string());
        fs::write(path, lines.join("\n"))
    }
}

pub struct SimpleCallGraphBuilder;
impl CallGraphBuilder for SimpleCallGraphBuilder {
    fn build_call_graph(&self, files: &[(String, String, String)]) -> CallGraph {
        let mut func_defs = vec![];
        // 收集所有 function 定義
        for (crate_name, path, code) in files {
            let ast_file: File = match syn::parse_file(code) {
                Ok(f) => f,
                Err(_) => continue,
            };
            for item in ast_file.items {
                if let Item::Fn(ref func) = item {
                    let name = func.sig.ident.to_string();
                    let mut callees = vec![];
                    visit_stmts(&func.block.stmts, &mut callees);
                    println!("[DEBUG][insert] fn={} crate={} path={} callees={:?}", name, crate_name, path, callees);
                    func_defs.push((name, crate_name.clone(), path.clone(), callees));
                }
            }
        }
        // 一次性建立全局 id_map
        let mut id_map: HashMap<String, (String, String, String)> = HashMap::new();
        for (name, crate_name, path, _) in &func_defs {
            let id = format!("{}@{}", name, crate_name);
            id_map.insert(id, (name.clone(), crate_name.clone(), path.clone()));
        }
        println!("[DEBUG][final id_map keys] {:?}", id_map.keys().collect::<Vec<_>>());

        // resolve callee id，這裡用全局 id_map
        let mut nodes = vec![];
        for (name, crate_name, _path, callees) in &func_defs {
            let id = format!("{}@{}", name, crate_name);
            if id == "main@crate1" {
                println!("[DEBUG] main@crate1 的 callees: {:?}", callees);
            }
            let callee_ids = callees.iter().filter_map(|callee_name| {
                if callee_name.contains("::") {
                    let parts: Vec<&str> = callee_name.split("::").collect();
                    if parts.len() == 2 {
                        let search_id = format!("{}@{}", parts[1], parts[0]);
                        println!("[DEBUG][resolve] parts: {:?} -> search_id={}", parts, search_id);
                        if id_map.contains_key(&search_id) {
                            Some(search_id)
                        } else {
                            println!("[DEBUG][resolve-miss] search_id={}  all_keys={:?}", search_id, id_map.keys().collect::<Vec<_>>() );
                            None
                        }
                    } else { None }
                } else {
                    let search_id = format!("{}@{}", callee_name, crate_name);
                    if id_map.contains_key(&search_id) {
                        Some(search_id)
                    } else {
                        println!("[DEBUG][resolve-miss] search_id={}  all_keys={:?}", search_id, id_map.keys().collect::<Vec<_>>() );
                        None
                    }
                }
            }).collect();
            nodes.push(CallGraphNode {
                id,
                callees: callee_ids,
            });
        }
        CallGraph { nodes }
    }
}

fn visit_stmts(stmts: &[Stmt], callees: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr, _) => visit_expr(expr, callees),
            _ => {}
        }
    }
}
fn visit_expr(expr: &Expr, callees: &mut Vec<String>) {
    match expr {
        Expr::Call(expr_call) => {
            if let Expr::Path(ref expr_path) = *expr_call.func {
                let segments: Vec<_> = expr_path.path.segments.iter().map(|s| s.ident.to_string()).collect();
                if !segments.is_empty() {
                    println!("[DEBUG] Detected call: {}", segments.join("::"));
                    callees.push(segments.join("::"));
                }
            }
            for arg in &expr_call.args {
                visit_expr(arg, callees);
            }
        }
        Expr::Block(expr_block) => visit_stmts(&expr_block.block.stmts, callees),
        _ => {}
    }
}
