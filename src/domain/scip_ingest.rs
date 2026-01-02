/// SCIP Index Ingestor.
/// Parses SCIP indices and builds a precise CallGraph using semantic information.
/// 
/// Phase 3.1: Parallel processing with rayon and DashMap for high performance.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::{Context, Result};
use dashmap::DashMap;
use rayon::prelude::*;

use crate::domain::callgraph::{CallGraph, CallGraphNode};

/// Represents a range in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRange {
    start_line: i32,
    start_col: i32,
    end_line: i32,
    end_col: i32,
}

impl SourceRange {
    fn contains(&self, other: &SourceRange) -> bool {
        // Check if `other` is fully contained within `self`
        if self.start_line > other.start_line || self.end_line < other.end_line {
            return false;
        }
        if self.start_line == other.start_line && self.start_col > other.start_col {
            return false;
        }
        if self.end_line == other.end_line && self.end_col < other.end_col {
            return false;
        }
        true
    }
}

/// A definition occurrence extracted from SCIP.
#[derive(Debug, Clone)]
struct DefinitionInfo {
    symbol: String,
    range: SourceRange,
}

/// SCIP Ingestor for building CallGraphs from SCIP indices.
pub struct ScipIngestor;

impl ScipIngestor {
    /// Ingest a SCIP index file and build a CallGraph.
    /// 
    /// Uses parallel processing for both definition collection (Pass 1)
    /// and reference resolution (Pass 2).
    /// 
    /// Phase 3.3: Uses memory-mapped file I/O to avoid large allocations.
    pub fn ingest_and_build_graph(scip_path: &Path) -> Result<CallGraph> {
        use std::fs::File;
        use memmap2::Mmap;
        use protobuf::Message;

        println!("[SCIP Ingest] Loading index from: {}", scip_path.display());
        
        // Memory-map the SCIP index file for efficient access
        let file = File::open(scip_path)
            .context("Failed to open SCIP index file")?;
        
        // SAFETY: We assume the file won't be modified while we're reading it.
        // The mmap provides a zero-copy view into the file.
        let mmap = unsafe { Mmap::map(&file) }
            .context("Failed to memory-map SCIP index file")?;
        
        let index = scip::types::Index::parse_from_bytes(&mmap)
            .context("Failed to parse SCIP index protobuf")?;

        // ═══════════════════════════════════════════════════════════════════
        // Pass 1: Parallel Definition Collection
        // ═══════════════════════════════════════════════════════════════════
        
        // Thread-safe maps for parallel access
        let definitions_by_file: DashMap<String, Vec<DefinitionInfo>> = DashMap::new();
        let symbol_to_node: DashMap<String, usize> = DashMap::new();
        let node_counter = AtomicUsize::new(0);
        
        // Collect nodes in parallel (we'll sort them later)
        let node_data: DashMap<usize, CallGraphNode> = DashMap::new();

        index.documents.par_iter().for_each(|document| {
            let file_path = document.relative_path.clone();
            let mut file_defs: Vec<DefinitionInfo> = Vec::new();

            for occurrence in &document.occurrences {
                // Check if this is a Definition (bit 0 of symbol_roles)
                let is_definition = occurrence.symbol_roles & 1 != 0;
                
                if is_definition && !occurrence.symbol.is_empty() {
                    let range = parse_scip_range(&occurrence.range);
                    
                    // Atomically get or create node ID for this symbol
                    let node_id = *symbol_to_node
                        .entry(occurrence.symbol.clone())
                        .or_insert_with(|| {
                            let id = node_counter.fetch_add(1, Ordering::SeqCst);
                            let label = extract_label_from_symbol(&occurrence.symbol);
                            node_data.insert(id, CallGraphNode {
                                id: occurrence.symbol.clone(),
                                callees: Vec::new(),
                                label: Some(label),
                            });
                            id
                        });

                    // We don't use node_id here directly, just ensure it's registered
                    let _ = node_id;

                    file_defs.push(DefinitionInfo {
                        symbol: occurrence.symbol.clone(),
                        range,
                    });
                }
            }

            // Sort definitions by range size (largest first) for containment lookup
            file_defs.sort_by(|a, b| {
                let a_size = (a.range.end_line - a.range.start_line) * 1000 
                           + (a.range.end_col - a.range.start_col);
                let b_size = (b.range.end_line - b.range.start_line) * 1000 
                           + (b.range.end_col - b.range.start_col);
                b_size.cmp(&a_size) // Largest first
            });

            definitions_by_file.insert(file_path, file_defs);
        });

        let def_count = node_counter.load(Ordering::SeqCst);
        println!("[SCIP Ingest] Found {} definitions (parallel)", def_count);

        // ═══════════════════════════════════════════════════════════════════
        // Pass 2: Parallel Reference Resolution
        // ═══════════════════════════════════════════════════════════════════
        
        let edge_counter = AtomicUsize::new(0);

        index.documents.par_iter().for_each(|document| {
            let file_path = &document.relative_path;
            
            // Get definitions for this file (if any)
            let file_defs = definitions_by_file
                .get(file_path)
                .map(|r| r.clone())
                .unwrap_or_default();

            for occurrence in &document.occurrences {
                // Check if this is a Reference (not a definition)
                let is_definition = occurrence.symbol_roles & 1 != 0;
                
                if !is_definition && !occurrence.symbol.is_empty() {
                    let ref_range = parse_scip_range(&occurrence.range);
                    let callee_symbol = &occurrence.symbol;

                    // Find the enclosing definition (the caller)
                    for def in &file_defs {
                        if def.range.contains(&ref_range) {
                            let caller_symbol = &def.symbol;
                            
                            // Add edge: caller -> callee
                            if let Some(caller_idx) = symbol_to_node.get(caller_symbol) {
                                // Avoid self-references
                                if caller_symbol != callee_symbol {
                                    // Thread-safe edge insertion
                                    if let Some(mut node) = node_data.get_mut(&*caller_idx) {
                                        if !node.callees.contains(callee_symbol) {
                                            node.callees.push(callee_symbol.clone());
                                            edge_counter.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                            break; // Found the innermost enclosing definition
                        }
                    }
                }
            }
        });

        let edge_count = edge_counter.load(Ordering::Relaxed);
        println!("[SCIP Ingest] Created {} edges (parallel)", edge_count);

        // ═══════════════════════════════════════════════════════════════════
        // Finalize: Convert DashMap to sorted Vec
        // ═══════════════════════════════════════════════════════════════════
        
        let mut nodes: Vec<CallGraphNode> = node_data
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(_, node)| node)
            .collect();
        
        // Sort by ID for deterministic output
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(CallGraph { nodes })
    }
}

/// Parse SCIP range format: [start_line, start_col, end_line, end_col] or [start_line, start_col, end_col]
fn parse_scip_range(range: &[i32]) -> SourceRange {
    match range.len() {
        3 => SourceRange {
            start_line: range[0],
            start_col: range[1],
            end_line: range[0], // Same line
            end_col: range[2],
        },
        4 => SourceRange {
            start_line: range[0],
            start_col: range[1],
            end_line: range[2],
            end_col: range[3],
        },
        _ => SourceRange {
            start_line: 0, start_col: 0, end_line: 0, end_col: 0,
        },
    }
}

/// Extract a human-readable label from a SCIP symbol string.
/// SCIP symbols look like: `rust-analyzer cargo crate_name 0.1.0 module/struct#method().`
fn extract_label_from_symbol(symbol: &str) -> String {
    // Take the last meaningful segment
    let parts: Vec<&str> = symbol.split(' ').collect();
    if let Some(last) = parts.last() {
        // Remove trailing punctuation like `().` or `#`
        let cleaned = last.trim_end_matches(|c| c == '(' || c == ')' || c == '.' || c == '#');
        // Replace path separators
        cleaned.replace('/', "::").to_string()
    } else {
        symbol.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use protobuf::Message;

    #[test]
    fn test_source_range_contains() {
        let outer = SourceRange {
            start_line: 10, start_col: 0,
            end_line: 20, end_col: 0,
        };
        let inner = SourceRange {
            start_line: 15, start_col: 5,
            end_line: 15, end_col: 10,
        };
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_parse_scip_range() {
        let r3 = parse_scip_range(&[10, 5, 15]);
        assert_eq!(r3.start_line, 10);
        assert_eq!(r3.end_line, 10);
        
        let r4 = parse_scip_range(&[10, 5, 20, 10]);
        assert_eq!(r4.start_line, 10);
        assert_eq!(r4.end_line, 20);
    }

    #[test]
    fn test_extract_label() {
        let symbol = "rust-analyzer cargo my_crate 0.1.0 src/lib.rs/MyStruct#my_method().";
        let label = extract_label_from_symbol(symbol);
        assert!(label.contains("my_method"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Mmap Loading Tests (Phase 3.3)
    // ═══════════════════════════════════════════════════════════════════

    /// Helper to create a SCIP index file for testing
    fn create_test_scip_index(
        dir: &std::path::Path,
        num_docs: usize,
        defs_per_doc: usize,
    ) -> std::path::PathBuf {
        let mut index = scip::types::Index::new();

        for doc_idx in 0..num_docs {
            let mut doc = scip::types::Document::new();
            doc.relative_path = format!("src/file_{}.rs", doc_idx);

            for def_idx in 0..defs_per_doc {
                let mut occ = scip::types::Occurrence::new();
                occ.symbol = format!("pkg::file_{}::func_{}", doc_idx, def_idx);
                let start_line = (def_idx * 20) as i32;
                occ.range = vec![start_line, 0, start_line + 15, 0];
                occ.symbol_roles = 1; // Definition bit
                doc.occurrences.push(occ);
            }

            index.documents.push(doc);
        }

        let path = dir.join("test.scip");
        let bytes = index.write_to_bytes().unwrap();
        let mut file = File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        path
    }

    #[test]
    fn test_mmap_loading_basic() {
        let dir = tempdir().unwrap();
        let scip_path = create_test_scip_index(dir.path(), 5, 10);
        
        let result = ScipIngestor::ingest_and_build_graph(&scip_path);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 50); // 5 docs * 10 defs
    }

    #[test]
    fn test_mmap_loading_empty_index() {
        let dir = tempdir().unwrap();
        
        // Create an empty but valid SCIP index
        let index = scip::types::Index::new();
        let path = dir.path().join("empty.scip");
        let bytes = index.write_to_bytes().unwrap();
        let mut file = File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        
        let result = ScipIngestor::ingest_and_build_graph(&path);
        assert!(result.is_ok());
        
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 0);
    }

    #[test]
    fn test_mmap_loading_large_index() {
        let dir = tempdir().unwrap();
        // 100 docs * 50 defs = 5000 definitions
        let scip_path = create_test_scip_index(dir.path(), 100, 50);
        
        let result = ScipIngestor::ingest_and_build_graph(&scip_path);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 5000);
    }

    #[test]
    fn test_mmap_loading_nonexistent_file() {
        let result = ScipIngestor::ingest_and_build_graph(Path::new("/nonexistent/path/index.scip"));
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to open"));
    }

    #[test]
    fn test_mmap_loading_invalid_protobuf() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.scip");
        
        // Write invalid data
        let mut file = File::create(&path).unwrap();
        file.write_all(b"this is not a valid protobuf").unwrap();
        
        let result = ScipIngestor::ingest_and_build_graph(&path);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_parallel_processing_correctness() {
        let dir = tempdir().unwrap();
        
        // Create index with references to test edge creation
        let mut index = scip::types::Index::new();
        
        // File 1: defines func_a
        let mut doc1 = scip::types::Document::new();
        doc1.relative_path = "src/a.rs".to_string();
        let mut def_a = scip::types::Occurrence::new();
        def_a.symbol = "pkg::func_a".to_string();
        def_a.range = vec![0, 0, 20, 0];
        def_a.symbol_roles = 1; // Definition
        doc1.occurrences.push(def_a);
        
        // Reference to func_b inside func_a
        let mut ref_b = scip::types::Occurrence::new();
        ref_b.symbol = "pkg::func_b".to_string();
        ref_b.range = vec![10, 5, 15]; // Inside func_a
        ref_b.symbol_roles = 0; // Reference
        doc1.occurrences.push(ref_b);
        
        index.documents.push(doc1);
        
        // File 2: defines func_b
        let mut doc2 = scip::types::Document::new();
        doc2.relative_path = "src/b.rs".to_string();
        let mut def_b = scip::types::Occurrence::new();
        def_b.symbol = "pkg::func_b".to_string();
        def_b.range = vec![0, 0, 10, 0];
        def_b.symbol_roles = 1; // Definition
        doc2.occurrences.push(def_b);
        
        index.documents.push(doc2);
        
        // Write index
        let path = dir.path().join("refs.scip");
        let bytes = index.write_to_bytes().unwrap();
        let mut file = File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        
        let result = ScipIngestor::ingest_and_build_graph(&path);
        assert!(result.is_ok());
        
        let graph = result.unwrap();
        
        // Should have 2 definitions
        assert_eq!(graph.nodes.len(), 2);
        
        // func_a should call func_b
        let func_a = graph.nodes.iter().find(|n| n.id == "pkg::func_a");
        assert!(func_a.is_some());
        assert!(func_a.unwrap().callees.contains(&"pkg::func_b".to_string()));
    }
}

