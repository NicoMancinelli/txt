use serde::{Deserialize, Serialize};

/// High-performance memory metrics for buffer management and UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub buffer_bytes: usize,
    pub rope_nodes_estimate: usize,
    pub lines_count: usize,
    pub characters_count: usize,
    pub history_depth: usize,
}

impl MemoryMetrics {
    pub fn format_human_readable(&self) -> String {
        let kb = (self.buffer_bytes as f64) / 1024.0;
        if kb < 1024.0 {
            format!("{:.1} KB ({} lines)", kb, self.lines_count)
        } else {
            let mb = kb / 1024.0;
            format!("{:.2} MB ({} lines)", mb, self.lines_count)
        }
    }
}
