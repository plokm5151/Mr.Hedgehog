use serde::{Serialize, Deserialize};
use crate::domain::callgraph::CallGraph;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphDto {
    pub nodes: Vec<NodeDto>,
    pub edges: Vec<EdgeDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeDto {
    pub id: String,
    pub label: String,
    pub package: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeDto {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

impl From<CallGraph> for GraphDto {
    fn from(cg: CallGraph) -> Self {
        let nodes = cg.nodes.iter().map(|n| {
            NodeDto {
                id: n.id.clone(),
                label: n.label.clone().unwrap_or_else(|| n.id.clone()),
                package: None, // Mr. Hedgehog domain doesn't reliably store package yet
                location: None, // Location info is deep in SourceManager, optional for now.
            }
        }).collect();

        let mut edges = Vec::new();
        for node in &cg.nodes {
            for callee in &node.callees {
                edges.push(EdgeDto {
                    from: node.id.clone(),
                    to: callee.clone(),
                    label: Some("call".to_string()),
                });
            }
        }

        GraphDto { nodes, edges }
    }
}
