// ...[與前面一樣的 import與read_all_rs_files略]...

use toml::Value as TomlValue;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input source files (can specify multiple)
    #[arg(short, long, required = false)]
    input: Vec<String>,

    /// Input source folders (can specify multiple)
    #[arg(short = 'd', long, required = false)]
    folder: Vec<String>,

    /// Analyze all crates in a Cargo workspace (auto-find all .rs)
    #[arg(long, required = false)]
    workspace: Option<String>,

    /// Output file path
    #[arg(short, long)]
    output: String,

    /// Output format (dot, json, text)
    #[arg(short, long, default_value = "dot")]
    format: String,
}

fn collect_rs_from_workspace(cargo_toml_path: &str) -> Vec<String> {
    let toml = std::fs::read_to_string(cargo_toml_path).unwrap();
    let value = toml.parse::<TomlValue>().unwrap();
    let mut all_rs = vec![];
    if let Some(workspace) = value.get("workspace") {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(path) = member.as_str() {
                    // 只收 src/**/*.rs
                    let src_dir = format!("{}/src", path.trim_end_matches('/'));
                    let mut content = read_all_rs_files(&src_dir);
                    if !content.is_empty() {
                        all_rs.push(content);
                    }
                }
            }
        }
    }
    all_rs
}

// ... read_all_rs_files 與 main() function ...
fn main() {
    let cli = Cli::parse();
    let mut src_code = String::new();

    // 1. input files
    for input_file in &cli.input {
        if let Ok(code) = fs::read_to_string(input_file) {
            src_code.push_str(&code);
            src_code.push('\n');
        }
    }

    // 2. folders
    for folder in &cli.folder {
        src_code.push_str(&read_all_rs_files(folder));
    }

    // 3. workspace
    if let Some(cargo_toml) = cli.workspace {
        for code in collect_rs_from_workspace(&cargo_toml) {
            src_code.push_str(&code);
        }
    }

    if src_code.trim().is_empty() {
        panic!("Please provide at least one --input <file> or --folder <dir> or --workspace <Cargo.toml>");
    }

    let usecase = AnalyzeUsecase {
        parser: &SynAstParser,
        callgraph_builder: &SimpleCallGraphBuilder,
        exporter: &DotExporter,
    };

    let result = usecase.run(&src_code, &cli.output);

    match result {
        Ok(_) => println!(
            "Analysis completed! Output written to {} (format: {})",
            cli.output, cli.format
        ),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
