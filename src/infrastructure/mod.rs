use std::collections::HashMap;
use crate::domain::callgraph::{CallGraph, CallGraphNode};
use crate::ports::{CallGraphBuilder, OutputExporter};
use syn::{Item, Stmt, Expr, ImplItem, Type, Pat};

type SymbolTable = HashMap<String, String>;
struct ImplInfo { type_name: String, methods: Vec<String> }

pub struct DotExporter;
impl OutputExporter for DotExporter {
    fn export(&self, cg: &CallGraph, path: &str) -> std::io::Result<()> {
        let mut out = vec!["digraph G {".into()];
        for n in &cg.nodes {
            let lbl = n.label.clone().unwrap_or_else(|| n.id.clone());
            out.push(format!("    \"{}\" [label=\"{}\"];", n.id, lbl.replace('"', "\\\"")));
            for c in &n.callees {
                out.push(format!("    \"{}\" -> \"{}\";", n.id, c));
            }
        }
        out.push("}".into());
        std::fs::write(path, out.join("\n"))
    }
}

pub struct SimpleCallGraphBuilder;
impl CallGraphBuilder for SimpleCallGraphBuilder {
    fn build_call_graph(&self, files: &[(String,String,String)]) -> CallGraph {
        let mut impls = Vec::<ImplInfo>::new();
        for (_,_,code) in files {
            if let Ok(ast) = syn::parse_file(code) {
                for it in ast.items {
                    if let Item::Impl(imp) = it {
                        if let Type::Path(tp) = &*imp.self_ty {
                            let ty = tp.path.segments.last().unwrap().ident.to_string();
                            let mut meth = Vec::new();
                            for ii in &imp.items { if let ImplItem::Fn(f)=ii { meth.push(f.sig.ident.to_string()); } }
                            impls.push(ImplInfo{type_name:ty,methods:meth});
                        }
                    }
                }
            }
        }

        let mut defs = Vec::new();
        for (cr, path, code) in files {
            if let Ok(ast)=syn::parse_file(code){
                for it in ast.items {
                    match it {
                        Item::Fn(f) => {
                            let mut callees=Vec::new();
                            visit_stmts(&f.block.stmts,&mut callees,&impls,&mut HashMap::new());
                            let lbl=Some(format!("{}:{}",path,f.sig.ident.span().start().line));
                            defs.push((f.sig.ident.to_string(),cr.clone(),path.clone(),callees,lbl));
                        }
                        Item::Impl(imp) => if let Type::Path(tp)=&*imp.self_ty {
                            let ty=tp.path.segments.last().unwrap().ident.to_string();
                            for ii in imp.items {
                                if let ImplItem::Fn(m)=ii{
                                    let mut callees=Vec::new();
                                    visit_stmts(&m.block.stmts,&mut callees,&impls,&mut HashMap::new());
                                    let lbl=Some(format!("{}:{}",path,m.sig.ident.span().start().line));
                                    defs.push((format!("{}::{}",ty,m.sig.ident),cr.clone(),path.clone(),callees,lbl));
                                }
                            }
                        }
                        _=>{}
                    }
                }
            }
        }

        let mut map=HashMap::new();
        for (n,c,p,_,_) in &defs { map.insert(format!("{}@{}",n,c),(n.clone(),c.clone(),p.clone())); }
        let nodes=defs.into_iter().map(|(n,c,_,cal,lbl)|{
            let id=format!("{}@{}",n,&c);
            let edges=cal.into_iter().map(|t|format!("{}@{}",t,&c)).collect();
            CallGraphNode{id,callees:edges,label:lbl}
        }).collect();
        CallGraph{nodes}
    }
}

fn visit_stmts(st:&[Stmt],cal:&mut Vec<String>,impls:&[ImplInfo],sym:&mut SymbolTable){
    for s in st{
        match s{
            Stmt::Local(l)=>{
                if let Pat::Ident(pi)=&l.pat{
                    if let Some(init)=&l.init{
                        if let Expr::Path(p)=&*init.expr{
                            sym.insert(pi.ident.to_string(),p.path.segments.last().unwrap().ident.to_string());
                        }
                    }
                }
            }
            Stmt::Expr(e,_)=>visit_expr(e,cal,impls,sym),
            _=>{}
        }
    }
}

fn visit_expr(e:&Expr,cal:&mut Vec<String>,impls:&[ImplInfo],sym:&SymbolTable){
    match e{
        Expr::Call(c)=>{
            if let Expr::Path(p)=&*c.func{
                cal.push(p.path.segments.iter().map(|s|s.ident.to_string()).collect::<Vec<_>>().join("::"));
            }
            for a in &c.args{visit_expr(a,cal,impls,sym);}
        }
        Expr::MethodCall(mc)=>{
            let m=mc.method.to_string();
            let recv_ty=if let Expr::Path(p)=&*mc.receiver{
                sym.get(&p.path.segments.last().unwrap().ident.to_string()).cloned()
            }else{None};
            if let Some(rt)=recv_ty{
                if impls.iter().any(|i|i.type_name==rt && i.methods.contains(&m)){
                    cal.push(format!("{}::{}",rt,m));
                }else{cal.push(format!("{}::{}",rt,m));}
            }else{cal.push(m.clone());}
            for a in &mc.args{visit_expr(a,cal,impls,sym);}
            visit_expr(&mc.receiver,cal,impls,sym);
        }
        Expr::Block(b)=>visit_stmts(&b.block.stmts,cal,impls,&mut sym.clone()),
        Expr::If(i)=>{
            visit_expr(&i.cond,cal,impls,sym);
            visit_stmts(&i.then_branch.stmts,cal,impls,&mut sym.clone());
            if let Some((_,e2))=&i.else_branch{visit_expr(e2,cal,impls,sym);}
        }
        Expr::Match(m)=>{
            visit_expr(&m.expr,cal,impls,sym);
            for a in &m.arms{visit_expr(&a.body,cal,impls,sym);}
        }
        _=>{}
    }
}
