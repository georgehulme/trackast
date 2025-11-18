# Trackast Implementation Plan

**Created:** November 18, 2025  
**Last Updated:** November 18, 2025  
**Status:** Phase 3.5 Complete - Ready for Phase 4

**Progress: 102/124 tasks complete (82%)**

This document breaks down the implementation into small, atomic tasks that can be executed incrementally.

---

## Summary

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1: Foundation (trackast-lib) | ✅ COMPLETE | 41/41 |
| Phase 2: Rust & Tree-sitter | ✅ COMPLETE | 26/26 |
| Phase 3: Python & Node.js | ✅ COMPLETE | 21/21 |
| Phase 3.5: Module Discovery | ✅ COMPLETE | 14/14 |
| Phase 4: Export/Analysis | ⏸️ PENDING | 0/11 |
| Phase 5: Stretch Goals | ⏸️ PENDING | 0/13 |

---

## Phase 1: Foundation (trackast-lib)

### 1.1 Project Setup

#### Task 1.1.1: Create trackast-lib crate structure
- [x] Create `trackast-lib/src/lib.rs` as entry point
- [x] Export empty modules (to be filled later)
- [x] Verify `cargo build` works for trackast-lib

#### Task 1.1.2: Create trackast binary crate structure
- [x] Create `trackast/src/main.rs` with minimal hello world
- [x] Create `trackast/src/lib.rs` for translator exports
- [x] Verify `cargo build` works for trackast binary

#### Task 1.1.3: Add dependencies to Cargo.toml files
- [x] Add `trackast-lib` as dependency in `trackast/Cargo.toml`
- [x] Test that both crates compile together

---

### 1.2 Abstract AST Data Models (trackast-lib)

#### Task 1.2.1: Create module file structure
- [x] Create `trackast-lib/src/ast/mod.rs`
- [x] Create `trackast-lib/src/ast/types.rs`
- [x] Create `trackast-lib/src/lib.rs` export statements

#### Task 1.2.2: Define Signature struct
- [x] Create `Signature` struct with fields: `params: Vec<(String, String)>`, `return_type: String`
- [x] Implement `Display` trait for `Signature`
- [x] Implement `PartialEq` and `Eq` traits
- [x] Write unit tests for signature display and equality

#### Task 1.2.3: Define FunctionCall struct
- [x] Create `FunctionCall` struct with fields: `target_name`, `target_module: Option<String>`, `line: usize`
- [x] Implement `Debug` and `Clone` traits
- [x] Write unit tests for instantiation

#### Task 1.2.4: Define FunctionDef struct
- [x] Create `FunctionDef` struct with fields: `name`, `signature`, `calls: Vec<FunctionCall>`, `module`
- [x] Implement `Debug`, `Clone`, `PartialEq` traits
- [x] Write unit tests for instantiation

#### Task 1.2.5: Define AbstractAST struct
- [x] Create `AbstractAST` struct with fields: `functions: Vec<FunctionDef>`, `module_path: String`
- [x] Implement `Debug` trait
- [x] Write unit tests for instantiation

#### Task 1.2.6: Add methods to AbstractAST
- [x] Add method `get_function(&self, name: &str) -> Option<&FunctionDef>`
- [x] Add method `add_function(&mut self, func: FunctionDef)`
- [x] Add method `module_path(&self) -> &str`
- [x] Write unit tests for each method

---

### 1.3 Function ID Generation (trackast-lib)

#### Task 1.3.1: Create function_id module
- [x] Create `trackast-lib/src/function_id/mod.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.3.2: Define FunctionId type
- [x] Create newtype `FunctionId(String)`
- [x] Implement `Display`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` traits
- [x] Write unit tests for display and hashing

#### Task 1.3.3: Implement Function ID generation algorithm
- [x] Create function `generate_id(module: &str, name: &str, signature: &Signature) -> FunctionId`
- [x] Format as `module::name::signature`
- [x] Write comprehensive unit tests with various module depths and signatures

#### Task 1.3.4: Create ID generation helper on FunctionDef
- [x] Add method `fn_id(&self) -> FunctionId` to `FunctionDef`
- [x] Uses `generate_id()` with module, name, and signature
- [x] Write unit tests

---

### 1.4 Graph Data Structure (trackast-lib)

#### Task 1.4.1: Create graph module
- [x] Create `trackast-lib/src/graph/mod.rs`
- [x] Create `trackast-lib/src/graph/node.rs`
- [x] Create `trackast-lib/src/graph/edge.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.4.2: Define GraphNode struct
- [x] Create `GraphNode` struct with fields: `id: FunctionId`, `is_external: bool`, `metadata: FunctionDef`
- [x] Implement `Debug`, `Clone`, `PartialEq`, `Eq` traits
- [x] Write unit tests

#### Task 1.4.3: Define GraphEdge struct
- [x] Create `GraphEdge` struct with fields: `from: FunctionId`, `to: FunctionId`, `line: usize`
- [x] Implement `Debug`, `Clone`, `PartialEq`, `Eq` traits
- [x] Write unit tests

#### Task 1.4.4: Define CallGraph struct
- [x] Create `CallGraph` struct with fields: `nodes: HashMap<FunctionId, GraphNode>`, `edges: Vec<GraphEdge>`
- [x] Implement `Debug`, `Clone` traits
- [x] Write unit tests for instantiation

#### Task 1.4.5: Implement CallGraph insert_node method
- [x] Add method `insert_node(&mut self, node: GraphNode) -> Result<()>`
- [x] Prevent duplicate nodes (return error if exists)
- [x] Write unit tests

#### Task 1.4.6: Implement CallGraph insert_edge method
- [x] Add method `insert_edge(&mut self, edge: GraphEdge) -> Result<()>`
- [x] Verify both nodes exist before adding edge
- [x] Allow multiple edges between same nodes (different lines)
- [x] Write unit tests

#### Task 1.4.7: Implement CallGraph get_node method
- [x] Add method `get_node(&self, id: &FunctionId) -> Option<&GraphNode>`
- [x] Write unit tests

#### Task 1.4.8: Implement CallGraph get_edges method
- [x] Add method `get_edges_from(&self, id: &FunctionId) -> Vec<&GraphEdge>`
- [x] Write unit tests

---

### 1.5 Graph Traversal (trackast-lib)

#### Task 1.5.1: Create traversal module
- [x] Create `trackast-lib/src/traversal/mod.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.5.2: Define TraversalResult struct
- [x] Create struct with fields: `reachable: HashSet<FunctionId>`, `visited_order: Vec<FunctionId>`
- [x] Implement `Debug`, `Clone` traits
- [x] Write unit tests

#### Task 1.5.3: Implement DFS traversal function
- [x] Create function `dfs_traversal(graph: &CallGraph, start: &FunctionId) -> TraversalResult`
- [x] Handle cycles (track visited nodes)
- [x] Return all reachable nodes
- [x] Write comprehensive unit tests with cycles

#### Task 1.5.4: Implement multi-entry traversal
- [x] Create function `traversal_from_entries(graph: &CallGraph, entries: &[FunctionId]) -> TraversalResult`
- [x] Call dfs_traversal for each entry
- [x] Combine results, removing duplicates
- [x] Write unit tests

#### Task 1.5.5: Create visitor trait for traversal
- [x] Define trait `Visitor` with method `visit(&mut self, node_id: &FunctionId)`
- [x] Modify DFS to accept optional visitor
- [x] Write unit tests with simple counter visitor

---

### 1.6 Query API (trackast-lib)

#### Task 1.6.1: Create query module
- [x] Create `trackast-lib/src/query/mod.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.6.2: Implement reachable_from query
- [x] Add method `reachable_from(&self, id: &FunctionId) -> Result<HashSet<FunctionId>>`
- [x] Use traversal internally
- [x] Write unit tests

#### Task 1.6.3: Implement direct_callers query
- [x] Add method `direct_callers(&self, id: &FunctionId) -> Vec<FunctionId>`
- [x] Return functions that directly call the given function
- [x] Write unit tests

#### Task 1.6.4: Implement direct_callees query
- [x] Add method `direct_callees(&self, id: &FunctionId) -> Vec<FunctionId>`
- [x] Return functions directly called by the given function
- [x] Write unit tests

#### Task 1.6.5: Implement get_function query
- [x] Add method `get_function(&self, id: &FunctionId) -> Option<&GraphNode>`
- [x] Write unit tests

#### Task 1.6.6: Implement external_calls query
- [x] Add method `external_calls(&self) -> Vec<&GraphEdge>`
- [x] Return only edges to external nodes
- [x] Write unit tests

---

### 1.7 Cycle Detection (trackast-lib)

#### Task 1.7.1: Create cycle detection module
- [x] Create `trackast-lib/src/cycles/mod.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.7.2: Define Cycle struct
- [x] Create struct `Cycle` with field `nodes: Vec<FunctionId>`
- [x] Implement `Debug`, `Clone` traits
- [x] Write unit tests

#### Task 1.7.3: Implement find_cycles function
- [x] Create function `find_cycles(graph: &CallGraph) -> Vec<Cycle>`
- [x] Use DFS-based cycle detection algorithm
- [x] Return minimal cycles (no duplicates)
- [x] Write comprehensive unit tests with various cycle patterns

#### Task 1.7.4: Implement has_cycle method on CallGraph
- [x] Add method `has_cycles(&self) -> bool`
- [x] Uses find_cycles internally
- [x] Write unit tests

---

### 1.8 Call Graph Builder (trackast-lib)

#### Task 1.8.1: Create builder module
- [x] Create `trackast-lib/src/builder/mod.rs`
- [x] Export from `trackast-lib/src/lib.rs`

#### Task 1.8.2: Define CallGraphBuilder struct
- [x] Create struct with fields: `asts: Vec<AbstractAST>`, `functions_map: HashMap<FunctionId, FunctionDef>`
- [x] Implement `new()` constructor
- [x] Write unit tests

#### Task 1.8.3: Implement add_ast method
- [x] Add method `add_ast(&mut self, ast: AbstractAST) -> Result<()>`
- [x] Extract all functions and add to map
- [x] Detect duplicate function IDs and return error
- [x] Write unit tests

#### Task 1.8.4: Implement build method
- [x] Add method `build(&self) -> Result<CallGraph>`
- [x] Create nodes for all functions (mark externals)
- [x] Create edges based on function calls
- [x] Write unit tests with various scenarios

#### Task 1.8.5: Implement build_from_entries method
- [x] Add method `build_from_entries(&self, entries: &[FunctionId]) -> Result<(CallGraph, TraversalResult)>`
- [x] Build complete graph
- [x] Traverse from entry points
- [x] Return both graph and traversal result
- [x] Write unit tests

---

## Phase 2: Tree-Sitter Integration & Rust Support

### 2.1 Project Setup (trackast)

#### Task 2.1.1: Add tree-sitter dependencies
- [x] Add `tree-sitter`, `tree-sitter-rust` to `trackast/Cargo.toml`
- [x] Verify dependencies resolve correctly
- [x] Run `cargo build`

#### Task 2.1.2: Create translators module structure
- [x] Create `trackast/src/translators/mod.rs`
- [x] Create `trackast/src/translators/rust.rs` (empty module)
- [x] Export from `trackast/src/lib.rs`

#### Task 2.1.3: Update main.rs with imports
- [x] Import `trackast_lib::*`
- [x] Import translators module
- [x] Create minimal CLI structure
- [x] Verify compilation

---

### 2.2 Rust Translator (trackast)

#### Task 2.2.1: Create RustTranslator struct
- [x] Define struct `RustTranslator;` (unit struct)
- [x] Write unit tests for instantiation

#### Task 2.2.2: Implement basic tree-sitter setup
- [x] Add method `setup_parser() -> Result<Parser>`
- [x] Create parser, set Rust language
- [x] Write unit tests

#### Task 2.2.3: Implement source parsing
- [x] Add method `parse_source(source: &str) -> Result<Tree>`
- [x] Parse source code into tree-sitter Tree
- [x] Write unit tests with simple Rust code

#### Task 2.2.4: Implement function definition query
- [x] Add method `query_function_defs(tree: &Tree, source: &str) -> Result<Vec<Node>>`
- [x] Use tree-sitter query for function_item
- [x] Return matching nodes
- [x] Write unit tests

#### Task 2.2.5: Implement function name extraction
- [x] Add method `extract_function_name(node: Node, source: &str) -> Result<String>`
- [x] Extract identifier from function node
- [x] Write unit tests

#### Task 2.2.6: Implement module path extraction
- [x] Add method `extract_module_path(tree: &Tree, source: &str) -> Result<String>`
- [x] Walk tree to find mod declarations
- [x] Build hierarchical module path
- [x] Write unit tests with nested modules

#### Task 2.2.7: Implement function call query
- [x] Add method `query_function_calls(node: Node, source: &str) -> Result<Vec<String>>`
- [x] Query for call expressions within function
- [x] Extract called function names
- [x] Write unit tests

#### Task 2.2.8: Implement signature extraction
- [x] Add method `extract_signature(node: Node, source: &str) -> Result<Signature>`
- [x] Parse parameters and return type
- [x] Handle generics by preserving them as-is
- [x] Write unit tests with various signatures

#### Task 2.2.9: Implement translate method
- [x] Add method `translate(source: &str, module_path: &str) -> Result<AbstractAST>`
- [x] Parse source with tree-sitter
- [x] Extract functions using above methods
- [x] Build AbstractAST
- [x] Write comprehensive integration tests

#### Task 2.2.10: Implement translate_file method
- [x] Add method `translate_file(path: &str, module_path: &str) -> Result<AbstractAST>`
- [x] Read file contents
- [x] Call `translate()`
- [x] Write integration tests

---

### 2.3 Call Resolution for Rust

#### Task 2.3.1: Create resolver module in trackast
- [x] Create `trackast/src/resolver/mod.rs`
- [x] Create `trackast/src/resolver/rust.rs`
- [x] Export from `trackast/src/lib.rs`

#### Task 2.3.2: Implement simple call resolution
- [x] Add function `resolve_call(call_name: &str, current_module: &str, all_functions: &[FunctionDef]) -> Option<(String, String)>`
- [x] Search for function in current module first
- [x] Then search parent modules
- [x] Return (module, name) or None
- [x] Write unit tests

#### Task 2.3.3: Implement import tracking (basic)
- [x] Add function `extract_imports(tree: &Tree, source: &str) -> Result<Vec<(String, String)>>`
- [x] Find use statements in Rust
- [x] Store import aliases
- [x] Write unit tests
- [x] Note: Return in `(alias, target)` format

#### Task 2.3.4: Mark unresolved as external
- [x] Modify translator to mark unresolved calls as external
- [x] Set `target_module = None` for unresolved calls
- [x] Write unit tests

---

### 2.4 CLI Interface - Phase 2 (trackast)

#### Task 2.4.1: Add clap dependency
- [x] Add `clap` to `trackast/Cargo.toml` with derive feature
- [x] Verify compilation

#### Task 2.4.2: Define CLI args structure
- [x] Create struct `Args` with fields: `input_path: String`, `entry_points: Vec<String>`
- [x] Use clap derive macros
- [x] Write tests for arg parsing

#### Task 2.4.3: Implement basic main function
- [x] Parse CLI arguments
- [x] Read input file
- [x] Translate with RustTranslator
- [x] Output AbstractAST as JSON to stdout
- [x] Write integration tests

#### Task 2.4.4: Add --language flag
- [x] Add optional `language` field to Args (defaults to auto-detect)
- [x] Update help text
- [x] Write tests

#### Task 2.4.5: Add error handling to main
- [x] Wrap main logic in error handling
- [x] Print errors to stderr
- [x] Exit with code 1 on error
- [x] Write tests

---

### 2.5 End-to-End Testing (trackast)

#### Task 2.5.1: Create test fixtures
- [x] Create `tests/fixtures/rust/simple.rs` with 2 simple functions
- [x] Create `tests/fixtures/rust/recursive.rs` with recursive function
- [x] Create `tests/fixtures/rust/imports.rs` with module imports

#### Task 2.5.2: Write integration test for simple Rust file
- [x] Test translating simple.rs
- [x] Verify AbstractAST structure
- [x] Write assertions

#### Task 2.5.3: Write integration test for recursive Rust file
- [x] Test translating recursive.rs
- [x] Verify call graph includes recursion
- [x] Write assertions

#### Task 2.5.4: Write end-to-end test combining all
- [x] Translate test fixture
- [x] Build CallGraph
- [x] Perform traversal
- [x] Verify results
- [x] Write comprehensive assertions

---

## Phase 3: Python & Node.js Support

### 3.1 Python Translator (trackast)

#### Task 3.1.1: Add tree-sitter-python dependency
- [x] Add to `trackast/Cargo.toml`
- [x] Verify compilation

#### Task 3.1.2: Create PythonTranslator struct
- [x] Define struct `PythonTranslator;` (unit struct)
- [x] Write unit tests

#### Task 3.1.3: Implement tree-sitter setup for Python
- [x] Add method `setup_parser() -> Result<Parser>`
- [x] Create parser, set Python language
- [x] Write unit tests

#### Task 3.1.4: Implement function definition query
- [x] Add method `query_function_defs(tree: &Tree, source: &str) -> Result<Vec<Node>>`
- [x] Use tree-sitter query for function_definition
- [x] Write unit tests

#### Task 3.1.5: Implement module path extraction
- [x] Add method `extract_module_path(file_path: &str) -> Result<String>`
- [x] Use file path to determine Python module hierarchy
- [x] Write unit tests

#### Task 3.1.6: Implement function call query
- [x] Add method `query_function_calls(node: Node, source: &str) -> Result<Vec<String>>`
- [x] Query for call expressions within function
- [x] Write unit tests

#### Task 3.1.7: Implement signature extraction (Python)
- [x] Add method `extract_signature(node: Node, source: &str) -> Result<Signature>`
- [x] Parse parameters (no type hints by default)
- [x] Return simple signature
- [x] Write unit tests

#### Task 3.1.8: Implement translate method for Python
- [x] Add method `translate(source: &str, module_path: &str) -> Result<AbstractAST>`
- [x] Integrate all extraction methods
- [x] Write integration tests

#### Task 3.1.9: Implement translate_file method for Python
- [x] Add method `translate_file(path: &str) -> Result<AbstractAST>`
- [x] Auto-derive module path from file path
- [x] Call `translate()`
- [x] Write tests

---

### 3.2 Node.js Translator (trackast)

#### Task 3.2.1: Add tree-sitter-javascript dependency
- [x] Add to `trackast/Cargo.toml`
- [x] Verify compilation

#### Task 3.2.2: Create JavaScriptTranslator struct
- [x] Define struct `JavaScriptTranslator;` (unit struct)
- [x] Write unit tests

#### Task 3.2.3: Implement tree-sitter setup for JavaScript
- [x] Add method `setup_parser() -> Result<Parser>`
- [x] Create parser, set JavaScript language
- [x] Write unit tests

#### Task 3.2.4: Implement function definition query
- [x] Add method `query_function_defs(tree: &Tree, source: &str) -> Result<Vec<Node>>`
- [x] Query for: function_declaration, method_definition, arrow_function (exported)
- [x] Write unit tests

#### Task 3.2.5: Implement module path extraction
- [x] Add method `extract_module_path(file_path: &str) -> Result<String>`
- [x] Use file path to determine module hierarchy
- [x] Write unit tests

#### Task 3.2.6: Implement function call query
- [x] Add method `query_function_calls(node: Node, source: &str) -> Result<Vec<String>>`
- [x] Query for call_expression nodes
- [x] Write unit tests

#### Task 3.2.7: Implement signature extraction (JavaScript)
- [x] Add method `extract_signature(node: Node, source: &str) -> Result<Signature>`
- [x] Extract parameters, no types
- [x] Write unit tests

#### Task 3.2.8: Implement translate method for JavaScript
- [x] Add method `translate(source: &str, module_path: &str) -> Result<AbstractAST>`
- [x] Integrate all extraction methods
- [x] Write integration tests

#### Task 3.2.9: Implement translate_file method for JavaScript
- [x] Add method `translate_file(path: &str) -> Result<AbstractAST>`
- [x] Auto-derive module path from file path
- [x] Call `translate()`
- [x] Write tests

---

### 3.3 Language Auto-Detection (trackast)

#### Task 3.3.1: Create language detection module
- [x] Create `trackast/src/language_detector/mod.rs`
- [x] Export from `trackast/src/lib.rs`

#### Task 3.3.2: Implement detect_language function
- [x] Add function `detect_language(file_path: &str) -> Result<Language>`
- [x] Use file extension (.rs, .py, .js, .ts)
- [x] Return error if unknown
- [x] Write unit tests

#### Task 3.3.3: Create Language enum
- [x] Define enum `Language { Rust, Python, JavaScript }`
- [x] Implement Display trait
- [x] Write unit tests

---

### 3.4 Translator Trait (trackast)

#### Task 3.4.1: Create translator trait
- [x] Define trait `Translator` with method `translate_file(&self, path: &str) -> Result<AbstractAST>`
- [x] Implement for RustTranslator, PythonTranslator, JavaScriptTranslator
- [x] Write unit tests

#### Task 3.4.2: Create translator factory
- [x] Add function `get_translator(language: Language) -> Box<dyn Translator>`
- [x] Return appropriate translator based on language
- [x] Write unit tests

---

### 3.5 CLI Update - Phase 3 (trackast)

#### Task 3.5.1: Update CLI to use language detection
- [x] Modify main to auto-detect language
- [x] Allow override with --language flag
- [x] Write tests

#### Task 3.5.2: Update CLI to accept directory
- [x] Allow --input-dir flag in addition to --input-file
- [x] Find all code files in directory
- [x] Translate all and combine AbstractASTs
- [x] Write tests

---

### 3.6 Multi-Language Integration Tests

#### Task 3.6.1: Create test fixtures for Python
- [x] Create `tests/fixtures/python/simple.py`
- [x] Create `tests/fixtures/python/imports.py`

#### Task 3.6.2: Create test fixtures for JavaScript
- [x] Create `tests/fixtures/javascript/simple.js`
- [x] Create `tests/fixtures/javascript/imports.js`

#### Task 3.6.3: Write integration tests for Python
- [x] Test translation and basic graph building
- [x] Write assertions

#### Task 3.6.4: Write integration tests for JavaScript
- [x] Test translation and basic graph building
- [x] Write assertions

#### Task 3.6.5: Write cross-language integration test
- [x] Translate mixed Rust + Python files
- [x] Build combined graph
- [x] Test traversal
- [x] Write comprehensive assertions

---

## Phase 4: Analysis & Export Features

### 4.1 JSON Export (trackast-lib)

#### Task 4.1.1: Add serde dependency
- [ ] Add `serde`, `serde_json` to `trackast-lib/Cargo.toml`
- [ ] Verify compilation

#### Task 4.1.2: Add serde derives to data types
- [ ] Add `#[derive(Serialize, Deserialize)]` to:
  - `FunctionId`
  - `Signature`
  - `FunctionCall`
  - `FunctionDef`
  - `GraphNode`
  - `GraphEdge`
  - `CallGraph`
- [ ] Write basic serialization tests

#### Task 4.1.3: Implement to_json method on CallGraph
- [ ] Add method `to_json(&self) -> Result<String>`
- [ ] Serialize complete graph to JSON
- [ ] Write tests

#### Task 4.1.4: Implement to_json_file method on CallGraph
- [ ] Add method `to_json_file(&self, path: &str) -> Result<()>`
- [ ] Write to file
- [ ] Write tests

---

### 4.2 Graphviz DOT Export (trackast-lib)

#### Task 4.2.1: Create dot module
- [ ] Create `trackast-lib/src/export/mod.rs`
- [ ] Create `trackast-lib/src/export/dot.rs`
- [ ] Export from `trackast-lib/src/lib.rs`

#### Task 4.2.2: Implement to_dot method on CallGraph
- [ ] Add method `to_dot(&self) -> String`
- [ ] Format nodes and edges as Graphviz DOT language
- [ ] Color external nodes differently
- [ ] Write unit tests

#### Task 4.2.3: Implement to_dot_file method on CallGraph
- [ ] Add method `to_dot_file(&self, path: &str) -> Result<()>`
- [ ] Write DOT to file
- [ ] Write tests

---

### 4.3 CLI Analysis Commands (trackast)

#### Task 4.3.1: Add sub-command structure
- [ ] Define enum `Command { Build, Query, Export }`
- [ ] Update Args to support sub-commands
- [ ] Write tests

#### Task 4.3.2: Implement build command
- [ ] Build call graph from input files
- [ ] Store in temporary format
- [ ] Output summary statistics
- [ ] Write tests

#### Task 4.3.3: Implement export command
- [ ] Read graph from input
- [ ] Export to specified format (json, dot)
- [ ] Write to output file
- [ ] Write tests

#### Task 4.3.4: Implement query command (basic)
- [ ] Query reachable functions from entry point
- [ ] Output results to stdout
- [ ] Write tests

---

### 4.4 Statistics API (trackast-lib)

#### Task 4.4.1: Create stats module
- [ ] Create `trackast-lib/src/stats/mod.rs`
- [ ] Export from `trackast-lib/src/lib.rs`

#### Task 4.4.2: Define GraphStats struct
- [ ] Create struct with fields: `node_count`, `edge_count`, `external_count`, `cycle_count`
- [ ] Implement Display trait
- [ ] Write unit tests

#### Task 4.4.3: Implement stats method on CallGraph
- [ ] Add method `stats(&self) -> GraphStats`
- [ ] Compute all statistics
- [ ] Write tests

---

## Phase 5: Stretch Goals

### 5.1 Lua Support

#### Task 5.1.1: Add tree-sitter-lua dependency
- [ ] Add to `trackast/Cargo.toml`

#### Task 5.1.2: Implement LuaTranslator
- [ ] Create struct `LuaTranslator;`
- [ ] Implement all required methods (similar to Python)
- [ ] Write tests

#### Task 5.1.3: Register Lua in language detection
- [ ] Add Lua to Language enum
- [ ] Update detect_language for .lua files
- [ ] Update translator factory
- [ ] Write tests

---

### 5.2 C++ Support (Preliminary)

#### Task 5.2.1: Add tree-sitter-cpp dependency
- [ ] Add to `trackast/Cargo.toml`

#### Task 5.2.2: Implement CppTranslator (basic)
- [ ] Create struct `CppTranslator;`
- [ ] Implement basic function extraction
- [ ] Note limitations with overloading
- [ ] Write tests

#### Task 5.2.3: Register C++ in language detection
- [ ] Add Cpp to Language enum
- [ ] Update detect_language for .cpp, .h, .cc files
- [ ] Update translator factory
- [ ] Write tests

---

### 5.3 Type Inference (Stretch)

#### Task 5.3.1: Create type inference module
- [ ] Create `trackast-lib/src/types/mod.rs`
- [ ] Define `TypeContext` struct
- [ ] Write basic unit tests

#### Task 5.3.2: Implement basic type tracking for Rust
- [ ] Add method to track function return types
- [ ] Resolve generic parameters
- [ ] Write tests

---

### 5.4 Closure Handling (Stretch)

#### Task 5.4.1: Extend FunctionDef for closures
- [ ] Add optional `captured_by: Vec<FunctionId>` field
- [ ] Update serialization
- [ ] Write tests

#### Task 5.4.2: Implement closure detection in Rust translator
- [ ] Detect closure definitions
- [ ] Track which functions create them
- [ ] Write tests

---

### 5.5 HTML Visualization (Stretch)

#### Task 5.5.1: Add web dependencies
- [ ] Add `warp`, `tokio`, `serde_json` to new visualization crate
- [ ] Create separate `trackast-web` crate

#### Task 5.5.2: Implement basic web server
- [ ] Serve simple HTML file
- [ ] Accept call graph JSON as POST data
- [ ] Write basic tests

#### Task 5.5.3: Create interactive visualization
- [ ] Add JavaScript/visualization library (D3.js or Sigma.js)
- [ ] Render graph interactively
- [ ] Write documentation

---

## Implementation Order Recommendation

Execute phases in this order for best results:

1. **Phase 1: Foundation** (trackast-lib)
   - Do tasks 1.1-1.8 in sequence
   - All independent, no external dependencies
   - Provides test framework for later phases

2. **Phase 2: Rust Support** (trackast + 2.0)
   - Tasks 2.1-2.5
   - Validates Phase 1 implementation
   - First real translator

3. **Phase 3: Multi-language** (trackast + 3.0-3.6)
   - Tasks 3.1-3.6
   - Validates Phase 1 & 2 architecture
   - Proves generality

4. **Phase 4: Export/Analysis** (trackast-lib + trackast)
   - Tasks 4.1-4.4
   - Depends on Phases 1-3
   - Makes tool practical

5. **Phase 5: Stretch Goals**
   - Tasks 5.1-5.5
   - Optional, do as time permits
   - Minimal dependencies on earlier phases

---

## Task Execution Template

For each task, follow this pattern:

```
Task [ID]: [Name]
Status: [ ] Not started [ ] In progress [ ] Complete
Subtasks:
  - [ ] Subtask 1
  - [ ] Subtask 2
Tests: [ ] Written [ ] Passing
Documentation: [ ] Added
Notes: 
```

---

## Notes

- Each task should take 5-30 minutes for a non-powerful AI
- Tasks are ordered to minimize dependencies
- All tasks include unit/integration tests
- No task requires understanding of the entire codebase
- Changes are always additive (no refactoring)
