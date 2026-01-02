# Mr. Hedgehog 🦔

A static analysis tool for multi-crate workspaces, enabling comprehensive call graph, AST, and dependency tree analysis for large-scale projects.

**Now featuring animated hedgehogs!** 🦔🦔

## ✨ Features

- **Multi-language support**: Rust and Python via SCIP indexing
- **Call graph generation**: Visualize function dependencies
- **AST analysis**: Parse and analyze source code structure
- **Dependency tracing**: Forward and reverse path analysis
- **Qt GUI**: Modern dark-themed desktop application with animated hedgehogs!

## 📦 Installation

### Prerequisites

- Rust toolchain (1.70+)
- Qt 6 (for GUI: `brew install qt@6` on macOS)
- rust-analyzer (for Rust SCIP analysis)
- scip-python (optional, for Python support)

```bash
# Install rust-analyzer
brew install rust-analyzer

# Install scip-python (optional)
npm install -g @sourcegraph/scip-python
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/plokm5151/Mr-Hedgehog
cd Mr-Hedgehog

# Build Rust backend
cargo build --release

# Build Qt frontend
cd frontend/build
cmake .. -DCMAKE_PREFIX_PATH=/usr/local/opt/qt@6
make -j4
```

## 🚀 Usage

### Command Line

```bash
# Analyze Rust workspace
mr_hedgehog --workspace ./Cargo.toml --output graph.dot

# Use SCIP engine (more precise)
mr_hedgehog --workspace ./Cargo.toml --output graph.dot --engine scip

# Analyze Python project
mr_hedgehog --engine scip --lang python --workspace ./project --output graph.dot

# Reverse trace
mr_hedgehog --workspace ./Cargo.toml --reverse "MyType::my_func" --output trace.txt
```

### GUI Application

```bash
open dist/MrHedgehog.app
```

Watch the hedgehogs walk around while you analyze your code! 🦔

## 📋 CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--workspace` | Path to Cargo.toml or project folder | - |
| `--output` | Output file path | required |
| `--engine` | `syn` or `scip` | `syn` |
| `--lang` | `rust` or `python` | `rust` |
| `--reverse` | Reverse trace target | - |
| `--expand-paths` | Expand all paths from main | `false` |
| `--debug` | Debug output | `false` |

## 🏗️ Architecture

```
src/
├── domain/           # Core logic
│   ├── language.rs   # Language enum (Rust, Python)
│   ├── callgraph.rs  # Call graph structures
│   └── scip_ingest.rs # SCIP parser (parallel)
├── infrastructure/   # External integrations
│   ├── scip_runner.rs # Multi-language SCIP
│   └── scip_cache.rs  # Incremental caching
└── ports/            # Interface adapters

frontend/             # C++ Qt GUI
├── src/
│   ├── mainwindow.cpp
│   └── graphview.cpp  # With animated hedgehogs! 🦔
└── CMakeLists.txt
```

## ⚡ Performance

- **Parallel processing**: rayon-based concurrent SCIP ingestion
- **Incremental caching**: Skip re-indexing unchanged files
- **Memory-mapped I/O**: Efficient large file loading

## 📄 License

MIT OR Apache-2.0

## 👤 Author

Frank Chen <plokm85222131@gmail.com>
