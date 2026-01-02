use crate::domain::callgraph::CallGraph;
use crate::infrastructure::source_manager::SourceManager;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub id: String,
    pub location: Option<String>,
    pub depth: usize,
    pub snippet: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TracePath {
    pub steps: Vec<TraceStep>,
}

pub struct TraceGenerator<'a> {
    graph: &'a CallGraph,
    source_manager: &'a SourceManager,
    max_depth: usize,
    max_paths: usize,
}

impl<'a> TraceGenerator<'a> {
    pub fn new(graph: &'a CallGraph, source_manager: &'a SourceManager) -> Self {
        Self {
            graph,
            source_manager,
            max_depth: 30, // Hardcap depth
            max_paths: 50, // Hardcap paths
        }
    }

    pub fn generate_paths(&self, start_node_id: &str) -> Vec<TracePath> {
        let mut results = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();

        self.dfs(
            start_node_id,
            0,
            &mut current_path,
            &mut visited,
            &mut results,
        );

        results
    }

    fn dfs(
        &self,
        current_id: &str,
        depth: usize,
        path_stack: &mut Vec<TraceStep>,
        visited: &mut HashSet<String>,
        results: &mut Vec<TracePath>,
    ) {
        if results.len() >= self.max_paths {
            return;
        }

        if depth >= self.max_depth {
            // Reached max depth, save current path and stop
            results.push(TracePath { steps: path_stack.clone() });
            return;
        }

        // Find node in graph
        let node_opt = self.graph.nodes.iter().find(|n| n.id == current_id);
        
        // Prepare trace step
        let location = node_opt.and_then(|n| n.label.clone());
        let snippet = location.as_ref().and_then(|loc| {
             // Location format "file:line"
             let parts: Vec<&str> = loc.split(':').collect();
             if parts.len() >= 2 {
                 let file = parts[0];
                 let line = parts[1].parse::<usize>().ok()?;
                 self.source_manager.get_snippet(file, line)
             } else {
                 None
             }
        });

        let step = TraceStep {
            id: current_id.to_string(),
            location,
            depth,
            snippet,
            note: if visited.contains(current_id) { Some("[Cycle Detected]".to_string()) } else { None },
        };

        path_stack.push(step);
        
        // Cycle check: If current node is already in the recursion stack (represented here by `path_stack` IDs? No, usually separate set)
        // Actually for DFS path enumeration, `visited` usually tracks nodes in the *current path* to detect cycles.
        // If we want to allow visiting same node via different branches, we strictly check if it's in ancestors.
        if visited.contains(current_id) {
             // Cycle detected. Commit path and back off.
             results.push(TracePath { steps: path_stack.clone() });
             path_stack.pop();
             return;
        }
        
        visited.insert(current_id.to_string());

        // Recurse
        if let Some(node) = node_opt {
            if node.callees.is_empty() {
                // Leaf node
                results.push(TracePath { steps: path_stack.clone() });
            } else {
                for callee in &node.callees {
                    self.dfs(callee, depth + 1, path_stack, visited, results);
                    if results.len() >= self.max_paths {
                        break;
                    }
                }
            }
        } else {
            // Node not found in graph (external or phantom?)
            results.push(TracePath { steps: path_stack.clone() });
        }

        visited.remove(current_id);
        path_stack.pop();
    }
}
