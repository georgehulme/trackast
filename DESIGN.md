# Trackast Design Document

**Date:** November 18, 2025  
**Project:** Call Dependency Graph Generator  

## Overview

Trackast is a Rust-based tool for analyzing abstract syntax tree (AST) data to generate call dependency graphs. It takes a set of top-level functions and constructs a directed graph representing function call relationships, enabling developers to understand call hierarchies and dependencies across a codebase.

## Project Goals

1. Parse AST-like data structures from source code
2. Identify top-level functions as entry points for analysis
3. Build a complete call dependency graph from these entry points
4. Track all function calls transitively through the call chain
5. Treat external and builtin functions as leaf nodes (no further traversal)
6. Generate a comprehensive, queryable representation of function dependencies

## Architecture

### Project Structure

```
trackast/
├── trackast-lib/          # Library crate (graph logic)
├── trackast/              # Application crate (tree-sitter translation & CLI)
├── src/
│   ├── lib.rs            # Core graph algorithms
│   └── bin/
│       └── trackast.rs   # CLI entry point
├── src/
│   ├── translators/      # Language-specific translators
│   │   ├── mod.rs
│   │   ├── rust.rs
│   │   ├── python.rs
│   │   ├── javascript.rs
│   │   ├── lua.rs
│   │   └── cpp.rs
│   ├── main.rs           # CLI and orchestration
│   └── lib.rs            # Re-exports translators
└── DESIGN.md             # This document
```

### Separation of Concerns

**trackast-lib (Library)** - Language-agnostic graph logic
- Abstract AST data models (independent of tree-sitter)
- Function ID generation
- Graph construction and traversal
- Query and analysis API
- Cycle detection and analysis
- No language-specific code
- No tree-sitter dependencies

**trackast (Application)** - Tree-sitter integration & translation
- Tree-sitter language bindings (rust, python, javascript, lua, cpp)
- Language-specific translators: tree-sitter CST → abstract AST
- Language-specific query patterns (function definitions, calls)
- Input/output handling and CLI
- No graph algorithm code (delegated to library)

### Supported Languages

**Current targets:**
- **Rust**: Via tree-sitter-rust grammar, modules, clear signatures
- **Python**: Via tree-sitter-python grammar, dynamic typing handling
- **Node.js**: Via tree-sitter-javascript grammar, ES6+ modules

**Stretch goals:**
- **Lua**: Via tree-sitter-lua grammar
- **C++**: Via tree-sitter-cpp grammar (with overload disambiguation challenges)

Each language implementation consists of:
- Tree-sitter grammar queries (find function definitions and calls)
- A translator module (tree-sitter → abstract AST)
- Language-specific configuration for module resolution

## Core Concepts

### Function ID

A unique identifier for each function composed of three parts:

```
[module/namespace]::[function_name]::[signature]
```

**Components:**
- **Module/Namespace**: Hierarchical path representing the function's location (e.g., `my_crate::utils::string_helpers`)
- **Function Name**: The simple name of the function (e.g., `process_input`)
- **Signature**: The function's type signature including parameter and return types (e.g., `(String, i32) -> Result<String, Error>`)

**Monomorphization Grouping:**
Monomorphized functions (e.g., `Vec::push<i32>` and `Vec::push<String>`) are grouped under a single generic Function ID. The graph treats them as a single entity to avoid explosion of nodes while still capturing the logical call structure.

**Example:**
```
std::collections::Vec::push::<T>(T) -> ()     # Groups all monomorphized variants
my_app::parser::parse_expr::(String) -> Result<Expr, ParseError>
<external>::serde::deserialize::<T>(String) -> Result<T, Error>
```

### Call Dependency Graph

A directed graph where:

- **Vertices**: Represent functions (identified by Function ID)
- **Edges**: Represent function calls (A → B means A calls B)
- **Leaf Nodes**: External or builtin functions with no outgoing edges
- **Entry Points**: Top-level functions specified by the user

**Properties:**
- May contain cycles due to recursive or mutually recursive functions
- Unidirectional: edges represent "calls" relationships
- Cycles can exist independent of external/leaf nodes

### External/Builtin Functions

Functions that are not part of the analyzed codebase:

- Standard library functions (e.g., `std::vec::Vec::push`)
- External crate functions
- Language intrinsics or builtins

**Treatment:**
- Always treated as leaf nodes
- No attempt to traverse their implementation
- Included in the graph to show dependencies
- Useful for identifying external dependencies

## Data Flow

```
Entry Point Source File
    ↓
Module Dependency Discovery (recursive import scanning)
    ↓
Load All Referenced Local Files
    ↓
Tree-sitter Parser (language-specific grammar)
    ↓
Concrete Syntax Tree (CST)
    ↓
Language-Specific Translator (tree-sitter CST → abstract AST)
    ↓
Abstract AST Model (combined from all modules)
    ↓
trackast-lib: Graph Builder
    ↓
Build Complete Call Graph
    ↓
Traversal from Entry Points
    ↓
Output Dependency Graph (JSON, DOT, etc.)
```

## Implementation Strategy: Tree-Sitter Integration

### Why Tree-Sitter?

1. **Multi-language support**: Single tool for Rust, Python, JavaScript, Lua, C++ without implementing custom parsers
2. **Production-ready**: Used by GitHub, VS Code, and other major projects
3. **Incremental parsing**: Efficient for large files and batch processing
4. **Language grammars**: Community-maintained, regularly updated
5. **Query language**: Built-in S-expression queries to extract patterns (functions, calls)

### Abstract AST Model

The abstract AST model is **tree-sitter independent**:

```rust
// In trackast-lib

pub struct AbstractAST {
    pub functions: Vec<FunctionDef>,
    pub module_path: String,
}

pub struct FunctionDef {
    pub name: String,
    pub signature: Signature,
    pub calls: Vec<FunctionCall>,
    pub module: String,
}

pub struct FunctionCall {
    pub target_name: String,
    pub target_module: Option<String>,  // None = unresolved/external
    pub line: usize,
}

pub struct Signature {
    pub params: Vec<(String, String)>,  // (name, type)
    pub return_type: String,
}
```

### Language-Specific Translators

Each language translator (in `trackast` application):

1. **Uses tree-sitter to parse source code**
2. **Queries for function definitions and calls** using language-specific patterns
3. **Extracts module/namespace information** (language-specific logic)
4. **Translates to AbstractAST** (library-compatible format)
5. **Handles language-specific quirks** (generics, type inference, dynamic calls)

### Module Dependency Discovery

The `ModuleLoader` component handles automatic discovery of dependencies:

1. **Recursive Import Scanning**
   - Parses import/require statements from source files
   - Language-specific patterns:
     * Rust: `use module::path` statements
     * Python: `import module` and `from module import x`
     * JavaScript: `import x from 'path'` and `require('path')`

2. **Module Resolution**
   - Resolves imports to local file paths
   - Supports multiple file layouts:
     * Single files: `module.rs`, `module.py`, `module.js`
     * Package directories: `module/mod.rs`, `module/__init__.py`, `module/index.js`
   - Filters external/built-in libraries (std, built-ins, etc.)

3. **Recursive Loading**
   - Loads entry point file
   - Extracts and resolves imports
   - Recursively loads discovered modules
   - Merges all ASTs into single combined model
   - Prevents re-loading of already-loaded modules

This allows users to specify only the entry point; all dependencies are discovered automatically.

**Example: Rust Translator**
```rust
// In trackast/src/translators/rust.rs

pub struct RustTranslator;

impl RustTranslator {
    pub fn translate(source: &str, module_path: &str) -> Result<AbstractAST> {
        let parser = Parser::new();
        parser.set_language(Language::rust())?;
        
        let tree = parser.parse(source, None)?;
        let root = tree.root_node();
        
        // Query for function definitions
        let func_query = Query::new(
            Language::rust(),
            "(function_item name: (identifier) @func_name)"
        )?;
        
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&func_query, root, source.as_bytes());
        
        // Extract functions and build AbstractAST
        let functions = self.extract_functions(matches, source, module_path)?;
        
        Ok(AbstractAST {
            functions,
            module_path: module_path.to_string(),
        })
    }
    
    fn extract_functions(&self, matches: ..., source: &str, module: &str) -> Result<Vec<FunctionDef>> {
        // Parse function signatures, extract calls, etc.
        // Handle Rust-specific constructs (impl blocks, generics, etc.)
    }
}
```

**Example: Python Translator**
```rust
// In trackast/src/translators/python.rs

pub struct PythonTranslator;

impl PythonTranslator {
    pub fn translate(source: &str, module_path: &str) -> Result<AbstractAST> {
        let parser = Parser::new();
        parser.set_language(Language::python())?;
        
        let tree = parser.parse(source, None)?;
        let root = tree.root_node();
        
        // Query for function definitions
        let func_query = Query::new(
            Language::python(),
            "(function_definition name: (identifier) @func_name)"
        )?;
        
        // Extract functions (handle Python-specific: decorators, methods, etc.)
        let functions = self.extract_functions(...)?;
        
        Ok(AbstractAST {
            functions,
            module_path: module_path.to_string(),
        })
    }
    
    fn extract_functions(&self, ...) -> Result<Vec<FunctionDef>> {
        // Handle Python: no explicit types, dynamic calls, introspection
    }
}
```

## Function ID Generation

### Algorithm

```
function_id(function) = {
    module = function.module_path.join("::")
    name = function.name
    signature = function.signature.to_string()
    
    return format!("{}::{}::{}", module, name, signature)
}
```

### Uniqueness Guarantees

Function IDs are globally unique within an analyzed codebase because they combine:
- Full module path (distinguishes same name in different modules)
- Function name
- Type signature (distinguishes overloads if applicable)

## Graph Construction Algorithm (trackast-lib)

The graph construction is **language-independent**, operating on AbstractAST:

### Input
- AbstractAST representation (language-agnostic)
- Set of top-level function entry points
- Module resolution map (optional, for disambiguation)

### Process

1. **Indexing Phase** (in `trackast-lib`)
   - Load AbstractAST for all modules
   - Generate Function ID for each function (using module + name + signature)
   - Build index: Function ID → function metadata

2. **Call Resolution Phase** (in `trackast-lib`)
   - For each function, examine its calls
   - Resolve called function names to Function IDs using:
     - Local module definitions
     - Imported modules (if tracked)
     - Mark unresolvable calls as external

3. **Graph Construction Phase** (in `trackast-lib`)
   - Create graph nodes for all functions
   - Add edges for all resolved calls
   - Mark external functions as leaf nodes

4. **Traversal Phase** (in `trackast-lib`)
   - Start from top-level entry points
   - DFS/BFS traversal with cycle tracking
   - Accumulate all reachable functions

### Output
- Complete call dependency graph
- Reachable function set from entry points
- Call site information and statistics

**Key insight**: All graph logic is in `trackast-lib`. The application only translates source code to AbstractAST.

## External Dependency Handling

When a function call cannot be resolved within the analyzed codebase:

1. Check if it's defined locally (in analyzed code)
2. If not found: create a synthetic Function ID with `<external>` marker
3. Add as leaf node to the graph
4. Do not attempt traversal

All unresolved calls are treated uniformly as external functions. Built-in/standard library functions and third-party library functions are distinguished only by naming convention (e.g., `<external>::std::...` vs `<external>::custom_lib::...`).

**Examples:**
```
<external>::println::(String) -> ()
<external>::std::serialize::(T) -> String
<external>::serde::json::to_string::(T) -> Result<String, Error>
```

## Cycle Detection and Handling

### Recursive Functions
- Direct recursion: function calls itself
- Mutual recursion: chain of functions calling back to the origin

### Treatment
- Detect during traversal using visited node tracking
- Represent as graph cycles (may include in output for analysis)
- Prevent infinite traversal by tracking visited nodes per path

## Usage Example

```rust
// Pseudocode combining module discovery + tree-sitter translation + graph analysis

use trackast_lib::{CallGraphBuilder, AbstractAST};
use trackast::module_loader::ModuleLoader;
use trackast::language::Language;

// Step 1: Auto-discover all dependencies from entry point
let mut loader = ModuleLoader::new("./src", Language::Rust);
let combined_ast = loader.load_all("main.rs")?;  // Recursively loads all imports

// Step 2: Build call graph from combined AST (all in trackast-lib)
let mut graph_builder = CallGraphBuilder::new();
graph_builder.add_ast(combined_ast);

// Step 3: Build and query the graph
let graph = graph_builder.build()?;

// Query the graph
let reachable = graph.reachable_from("my_app::main::()")?;
let dependencies = graph.get_dependencies_of("my_app::helpers::parse::(String)")?;

// Export
graph.export_to_json("call_graph.json")?;
graph.export_to_dot("call_graph.dot")?;
```

## CLI Usage Example

```bash
# Auto-discovery (default) - loads entry point + all imports recursively
$ trackast --input src/main.rs
📝 Detected language: Rust
📂 Using root directory: "src"
🔍 Auto-discovering module dependencies...
📦 Found 42 functions (across 7 modules)
🔗 Built graph with 50 nodes and 180 edges
✅ Output written to stdout

# Single file only - no dependency discovery
$ trackast --input src/main.rs --no-discover
📄 Loading single file (dependencies disabled)
📦 Found 15 functions
🔗 Built graph with 18 nodes and 30 edges

# Custom root directory for module resolution
$ trackast --input app/main.py --root /path/to/project
📂 Using root directory: "/path/to/project"
🔍 Auto-discovering module dependencies...

# Export to specific format and file
$ trackast --input main.js --format dot --output graph.dot
✅ Output written to "graph.dot"
```

## Implementation Phases

### Phase 1: Foundation (trackast-lib)
- [ ] Abstract AST data models
- [ ] Function ID generation
- [ ] Graph data structure
- [ ] Basic traversal algorithm (DFS/BFS)
- [ ] Core cycle detection
- [ ] Basic query API

### Phase 2: Tree-Sitter Integration & Rust Support (trackast)
- [ ] Tree-sitter dependency and bindings
- [ ] Rust translator (tree-sitter → abstract AST)
- [ ] Rust-specific queries (function defs, calls, modules)
- [ ] Call resolution for Rust (module paths, imports)
- [ ] Basic CLI interface
- [ ] End-to-end testing with Rust codebases

### Phase 3: Python & Node.js Support (trackast)
- [ ] Python translator (tree-sitter-python → abstract AST)
- [ ] Python-specific queries and module handling
- [ ] Node.js (tree-sitter-javascript → abstract AST)
- [ ] JavaScript/TypeScript-specific queries
- [ ] Language auto-detection and routing
- [ ] Multi-language integration testing

### Phase 4: Analysis & Export Features (trackast-lib + trackast)
- [ ] Advanced query API (filtering, statistics)
- [ ] JSON export with metadata
- [ ] Graphviz DOT export with styling
- [ ] Enhanced CLI with sub-commands
- [ ] Configuration file support (.trackast.toml)

### Phase 5: Stretch Goals
- [ ] **Lua Support**: Tree-sitter-lua translator
- [ ] **C++ Support**: Tree-sitter-cpp translator (with caveats on overloading)
- [ ] **Full Type Inference**: Cross-module type tracking (Rust focus)
- [ ] **Closure/Lambda Handling**: Anonymous function tracking
- [ ] **HTML Visualization**: Interactive web UI (JavaScript frontend)
- [ ] **Performance Optimization**: Parallel parsing, caching, incremental updates

## Testing Strategy

- **Unit Tests**: Function ID generation, graph construction, traversal
- **Integration Tests**: End-to-end analysis on sample codebases
- **Edge Cases**: Cycles, external functions, deeply nested calls

## Future Enhancements

1. **Additional Languages**: Extend to Go, Java, C#, and other languages
2. **Incremental Analysis**: Reuse previous graph results when code changes
3. **Performance Metrics**: Track call frequencies, execution paths, hot paths
4. **Data Flow Analysis**: Integrate with dataflow analysis for more context
5. **Dead Code Detection**: Identify functions unreachable from entry points
6. **API Documentation Generation**: Produce documentation from call graphs
7. **Interactive Analysis**: Web UI for exploring and filtering call graphs
