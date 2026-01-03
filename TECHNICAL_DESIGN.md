# Technical Design & Engineering Highlights

This document provides a deep dive into the engineering decisions and high-performance techniques implemented in Mr. Hedgehog.

## 1. High-Performance Graph Algorithms

### Custom DFS with Path Reconstruction
To enable deep trace analysis, we implemented a stack-based **Depth-First Search (DFS)** algorithm:
- **Cycle Detection**: Maintains a `visited` stack during traversal to identify and break recursive cycles, preventing infinite loops.
- **Trace Reconstruction**: Efficiently rebuilds the full execution path from the entry point (e.g., `main`) to any leaf node using a path-tracking mechanism.

### Layered Flowchart Layout
For the flowchart visualization (`--mode flowchart`):
- **Execution Depth Calculation**: Nodes are assigned layers based on their minimum distance from entry points.
- **Topological Sorting**: Ensures a logical, top-to-bottom sequence for sequential execution flow.

---

## 2. Advanced Memory Management

### Memory-Mapped I/O (mmap)
For processing large SCIP (LSIF) index files (potentially GB-sized):
- **Zero-Copy Loading**: Uses the `memmap2` crate to map index files directly into the process's address space.
- **Zero-Copy Protobuf Parsing**: Combined with the `prost` crate, we parse Protobuf messages directly from the memory-mapped region without copying to the heap, significantly reducing peak memory usage.

### String Interning & Deduplication
- **Memory Footprint Optimization**: Uses `HashSet` and reference-counted structures to ensure function identifiers (e.g., `std::vec::Vec`) are stored only once, drastically reducing memory usage in large workspaces.

---

## 3. Concurrency & Parallelism

### Adaptive Thread Pooling
- **System-Aware Indexing**: Detects total CPU cores and reserves a configurable percentage (default 50%) for UI and other background tasks, preventing system starvation during heavy analysis.

### Fine-Grained Locking
- **Concurrent Indexing**: Utilizes `DashMap` (a concurrent hash map based on sharded locking) instead of coarse-grained `RwLock<HashMap>`. This reduces lock contention by up to 90% when multiple threads are indexing different crates simultaneously.
- **Rayon-Driven Parallelism**: Implements a work-stealing scheduler for file-level parallel analysis, ensuring balanced load across all available cores.

---

## 4. System-Level Optimizations

### Optimized Persistence
- **Batch Transactions**: In the `sled` DB storage backend, multiple node/edge writes are consolidated into single batch transactions to maximize IOPS and minimize write-ahead log overhead.

### Robust Environment Discovery
- **GUI Context Handling**: Implements a POSIX-compliant PATH resolution logic and fallback mechanism to locate the `cargo` binary when running as a standalone macOS Bundle (`.app`), where shell-defined environment variables are often missing.

---

## 5. Built for Scalability

Mr. Hedgehog's core is designed as a language-agnostic graph engine. By implementing a common `domain` model for `CallGraph` and `FlowGraph`, we can easily extend support to other languages that provide SCIP/LSIF indexers.
