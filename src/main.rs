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
}

fn collect_rs(dir:&str, crate_name:&str)->Vec<(String,String,String)>{
    fn walk(p:&Path, c:&str, out:&mut Vec<(String,String,String)>){
        if p.ends_with("target")||p.ends_with(".git"){return;}
        if let Ok(rd)=fs::read_dir(p){
            for e in rd.flatten(){
                let path=e.path();
                if path.is_dir(){walk(&path,c,out);}
                else if path.extension().map(|x|x=="rs").unwrap_or(false){
                    if let Ok(src)=fs::read_to_string(&path){
                        out.push((c.to_string(),path.display().to_string(),src));
                    }
                }
            }
        }
    }
    let mut v=Vec::new(); walk(Path::new(dir),crate_name,&mut v); v
}

fn parse_ws(ws:&str)->Vec<(String,String)>{
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

fn main(){
    let cli=Cli::parse();
    let mut files=Vec::<(String,String,String)>::new();

    // single files
    for f in &cli.input{
        if let Ok(src)=fs::read_to_string(f){
            files.push(("main".into(),f.clone(),src));
        }
    }
    // folders
    for d in &cli.folder{ files.extend(collect_rs(d,"main")); }
    // workspace
    if let Some(ws)=&cli.workspace{
        for (c,src_dir) in parse_ws(ws){
            files.extend(collect_rs(&src_dir,&c));
        }
    }
    if files.is_empty(){panic!("No input provided");}

    // ── 2. **唯一一次** 建圖 ─────────────────
    let cg_builder=SimpleCallGraphBuilder{};
    let callgraph=cg_builder.build_call_graph(&files);

    // debug
    println!("\n==== [DEBUG nodes] ====");
    for n in &callgraph.nodes{println!("{} -> {:?}",n.id,n.callees);}
    println!("========================");

    // ── 3. trace from main@ ──────────────────
    let mut map=HashMap::new(); for n in &callgraph.nodes{map.insert(n.id.clone(),n);}
    let entry=callgraph.nodes.iter()
        .find(|n|n.id.starts_with("main@"))
        .map(|n|n.id.clone())
        .unwrap_or_else(||{eprintln!("WARN: no main() found");"".into()});

    fn dfs(id:&str,map:&HashMap<String,&tracecraft::domain::callgraph::CallGraphNode>,
           vis:&mut HashSet<String>,trace:&mut Vec<String>){
        if !vis.insert(id.into()){return;}
        trace.push(id.into());
        if let Some(n)=map.get(id){for c in &n.callees{dfs(c,map,vis,trace);}}
    }
    let mut vis=HashSet::new(); let mut trace=Vec::new();
    if !entry.is_empty(){ dfs(&entry,&map,&mut vis,&mut trace); }

    println!("\n=== Call-flow ===");
    for (i,id) in trace.iter().enumerate(){println!("{}. {}",i+1,id);}
    println!("=================\n");

    // ── 4. export dot ────────────────────────
    let exporter=DotExporter{};
    exporter.export(&callgraph,&cli.output).unwrap();
    println!("Graph saved to {}",cli.output);
}
