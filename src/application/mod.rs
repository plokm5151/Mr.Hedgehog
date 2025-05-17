use crate::ports::{AstParser, CallGraphBuilder, OutputExporter};

pub struct AnalyzeUsecase<'a> {
    pub parser: &'a dyn AstParser,
    pub callgraph_builder: &'a dyn CallGraphBuilder,
    pub exporter: &'a dyn OutputExporter,
}

impl<'a> AnalyzeUsecase<'a> {
    pub fn run(&self, src: &str, export_path: &str) -> std::io::Result<()> {
        let _ast = self.parser.parse(src);            // 仍保留 AST 解析
        let cg   = self.callgraph_builder.build_call_graph(src);
        self.exporter.export(&cg, export_path)
    }
}
