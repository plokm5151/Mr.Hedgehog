use clap::Parser;
use std::fs;
use std::collections::{HashMap, HashSet};

use tracecraft::infrastructure::{SimpleCallGraphBuilder, DotExporter};
use tracecraft::infrastructure::project_loader::ProjectLoader;
use tracecraft::ports::{CallGraphBuilder, OutputExporter};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// single .rs file(s)
    #[arg(short, long)]
    input: Vec<String>,

    /// folder(s) (recursively collect *.rs)
    #[arg(short='d', long)]
    folder: Vec<String>,

    /// Cargo workspace Cargo.toml
    #[arg(long)]
    workspace: Option<String>,

    /// output path
    #[arg(short, long)]
    output: String,

    /// output format (ignored for now)
    #[arg(short, long, default_value="dot")]
    format: String,

    /// 反向查詢（查詢所有能呼叫到此 function 的所有路徑，例 Type::func@crate）
    #[arg(long)]
    reverse: Option<String>,

    /// 展開 main 到所有葉節點的完整呼叫路徑
    #[arg(long)]
    expand_paths: bool,

    /// 分支 event 摘要模式（if/match 分支遇到相同 event 只記一次，不重複展開）
    #[arg(long)]
    branch_summary: bool,

    /// Enable debug output
    #[arg(long, short='D')]
    debug: bool,
}

fn main() {
    let cli=Cli::parse();

    if cli.debug {
        println!("[DEBUG] Config: {:?}", cli);
    }

    let mut files=Vec::<(String,String,String)>::new();

    // workspace (primary method)
    if let Some(ws) = &cli.workspace {
        match ProjectLoader::load_workspace(ws) {
            Ok(loaded_files) => {
                println!("Loaded {} files from workspace", loaded_files.len());
                files.extend(loaded_files);
            },
            Err(e) => panic!("Failed to load workspace: {:?}", e),
        }
    } else {
        // Legacy/Fallback handling could go here. 
        // For now, if no workspace is provided but input is, we panic (based on user request to focus on workspace)
        // Or we could implement a quick fallback if needed.
        if !cli.input.is_empty() || !cli.folder.is_empty() {
             panic!("Legacy input/folder mode is momentarily disabled during refactor. Please use --workspace.");
        }
    }

    if files.is_empty(){panic!("No input provided");}

    // ── 2. **唯一一次** 建圖 ─────────────────
    let cg_builder=SimpleCallGraphBuilder{};
    let callgraph=cg_builder.build_call_graph(&files);

    // for quick lookup
    let mut map=HashMap::new(); for n in &callgraph.nodes{map.insert(n.id.clone(),n);}
    let entry=callgraph.nodes.iter()
        .find(|n|n.id.starts_with("main@"))
        .map(|n|n.id.clone())
        .unwrap_or_else(||{eprintln!("WARN: no main() found");"".into()});

    // ── reverse call查詢 ──────────────────────
    if let Some(ref target_id) = cli.reverse {
        println!("=== Reverse call tracing: {} ===", target_id);
        // 1. 構建 caller_map: callee_id → Vec<caller_id>
        let mut caller_map: HashMap<String, Vec<String>> = HashMap::new();
        for node in &callgraph.nodes {
            for callee in &node.callees {
                caller_map.entry(callee.clone()).or_default().push(node.id.clone());
            }
        }

        // 2. BFS/DFS 搜尋所有從 main@... 到 target_id 的完整呼叫路徑
        let mut all_paths: Vec<Vec<String>> = vec![];
        let mut stack = vec![(vec![entry.clone()], entry.clone())]; // (目前路徑, 當前節點)

        while let Some((path, node_id)) = stack.pop() {
            if node_id == *target_id {
                all_paths.push(path.clone());
                continue;
            }
            // 找 callee
            if let Some(n) = map.get(&node_id) {
                for callee in &n.callees {
                    if !path.contains(callee) { // 防止循環
                        let mut new_path = path.clone();
                        new_path.push(callee.clone());
                        stack.push((new_path, callee.clone()));
                    }
                }
            }
        }
        if all_paths.is_empty() {
            println!("找不到任何路徑從 main 到 {}", target_id);
        } else {
            for (i, path) in all_paths.iter().enumerate() {
                println!("路徑 {}:", i+1);
                for seg in path {
                    println!("  {}", seg);
                }
            }
        }
        return;
    }

    // ── 3. trace from main，分支摘要模式 ──────────────────
    if cli.debug {
        println!("\n==== [DEBUG nodes] ====");
        for n in &callgraph.nodes{println!("{} -> {:?}",n.id,n.callees);}
        println!("========================");
    }

    let mut all_paths: Vec<Vec<String>> = Vec::new();
    if !entry.is_empty() && cli.expand_paths {
        fn dfs_expand(
            node_id: &str,
            map: &HashMap<String,&tracecraft::domain::callgraph::CallGraphNode>,
            path: &mut Vec<String>,
            all_paths: &mut Vec<Vec<String>>,
            depth: usize,
            branch_summary: bool,
            branch_event_set: &mut HashSet<String>
        ) {
            if depth > 128 { return; }
            if let Some(n) = map.get(node_id) {
                if n.callees.is_empty() {
                    all_paths.push(path.clone());
                } else {
                    for callee in &n.callees {
                        // 判斷是否為分支 event
                        let is_branch_event = callee.starts_with("if(") || callee.starts_with("match(") || callee.starts_with("match_arm");
                        if branch_summary && is_branch_event {
                            if branch_event_set.contains(callee) { continue; }
                            branch_event_set.insert(callee.clone());
                        }
                        if path.contains(callee) { continue; }
                        path.push(callee.clone());
                        dfs_expand(callee, map, path, all_paths, depth+1, branch_summary, branch_event_set);
                        path.pop();
                        if branch_summary && is_branch_event {
                            branch_event_set.remove(callee);
                        }
                    }
                }
            }
        }
        let mut path = vec![entry.clone()];
        let mut branch_event_set = HashSet::new();
        dfs_expand(&entry, &map, &mut path, &mut all_paths, 0, cli.branch_summary, &mut branch_event_set);

        println!("\n=== All call paths from entry (main) ===");
        for (i, p) in all_paths.iter().enumerate() {
            println!("Path {}:", i+1);
            for seg in p {
                println!("  {}", seg);
            }
            println!();
        }
    }

    // ── 4. export dot ────────────────────────
    let exporter=DotExporter{};
    exporter.export(&callgraph,&cli.output).unwrap();
    println!("Graph saved to {}",cli.output);
}

