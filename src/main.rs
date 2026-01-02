use clap::Parser;
use std::fs;
use std::path::Path;
use std::collections::{HashMap, HashSet};

use tracecraft::infrastructure::{SimpleCallGraphBuilder, DotExporter};
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

fn collect_rs(dir:&str, crate_name:&str)->Vec<(String,String,String)> {
    fn walk(p:&Path, c:&str, out:&mut Vec<(String,String,String)>) {
        if p.ends_with("target")||p.ends_with(".git"){return;}
        if let Ok(rd)=fs::read_dir(p) {
            for e in rd.flatten() {
                let path=e.path();
                if path.is_dir(){walk(&path,c,out);}
                else if path.extension().map(|x|x=="rs").unwrap_or(false) {
                    if let Ok(src)=fs::read_to_string(&path) {
                        out.push((c.to_string(),path.display().to_string(),src));
                    }
                }
            }
        }
    }
    let mut v=Vec::new(); walk(Path::new(dir),crate_name,&mut v); v
}

fn parse_ws(ws:&str)->Vec<(String,String)> {
    let root=Path::new(ws).parent().unwrap();
    let toml=fs::read_to_string(ws).expect("read Cargo.toml");
    let v:toml::Value=toml::from_str(&toml).expect("toml");
    v["workspace"]["members"].as_array().expect("members")
      .iter()
      .map(|m|{
          let s=m.as_str().unwrap();
          let crate_dir=root.join(s);
          let crate_name=s.split('/').last().unwrap();
          (crate_name.to_string(),crate_dir.join("src").display().to_string())
      }).collect()
}

fn main() {
    let cli=Cli::parse();

    if cli.debug {
        println!("[DEBUG] Config: {:?}", cli);
    }

    let mut files=Vec::<(String,String,String)>::new();

    // single files
    for f in &cli.input {
        if let Ok(src)=fs::read_to_string(f) {
            files.push(("main".into(),f.clone(),src));
        }
    }
    // folders
    for d in &cli.folder { files.extend(collect_rs(d,"main")); }
    // workspace
    if let Some(ws)=&cli.workspace {
        for (c,src_dir) in parse_ws(ws) {
            files.extend(collect_rs(&src_dir,&c));
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

