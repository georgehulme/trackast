use clap::Parser;
use trackast_lib::builder::CallGraphBuilder;
use trackast_lib::export::to_dot;
use trackast_lib::function_id::FunctionId;
use trackast_lib::ast::AbstractAST;
use trackast_lib::graph::CallGraph;
use trackast::module_loader::ModuleLoader;
use trackast::language::Language;
use std::path::{PathBuf, Path};

#[derive(Parser, Debug)]
#[command(name = "trackast")]
#[command(about = "Call dependency graph generator", long_about = None)]
struct Args {
    /// Input file path (entry point)
    #[arg(short, long)]
    input: PathBuf,

    /// Root directory for module resolution (defaults to input file directory)
    #[arg(short, long)]
    root: Option<PathBuf>,

    /// Module path for the input file (auto-detected if not specified)
    #[arg(short, long)]
    module: Option<String>,

    /// Output file path (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: json or dot
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Language (auto-detected from file extension if not specified)
    #[arg(short, long)]
    language: Option<String>,

    /// Automatically discover and load dependencies
    #[arg(long)]
    no_discover: bool,

    /// Entry point function ID(s) for graph traversal
    /// Format: `module::function` (auto-matches signature) or `module::function::signature` (exact match)
    /// Example: --entry-points `myapp::main` --entry-points `api::handler`
    #[arg(long)]
    entry_points: Vec<String>,
}

fn resolve_entry_points(
    entry_point_specs: &[String],
    graph: &trackast_lib::graph::CallGraph,
) -> Result<Vec<FunctionId>, String> {
    let mut resolved = Vec::new();

    for spec in entry_point_specs {
        let parts: Vec<&str> = spec.splitn(3, "::").collect();
        
        let (module, function, signature_opt) = match parts.len() {
            2 => (parts[0], parts[1], None),
            3 => (parts[0], parts[1], Some(parts[2])),
            _ => {
                return Err(format!(
                    "Invalid entry point format '{spec}'. Use 'module::function' or 'module::function::signature'"
                ))
            }
        };

        if let Some(sig) = signature_opt {
            // Exact match with signature
            let exact_id = FunctionId::new(format!("{module}::{function}::{sig}"));
            if graph.nodes.contains_key(&exact_id) {
                resolved.push(exact_id);
            } else {
                return Err(format!("Entry point not found: {spec}"));
            }
        } else {
            // Fuzzy match: find functions matching module::function with any signature
            let matching: Vec<FunctionId> = graph
                .nodes
                .keys()
                .filter(|id| {
                    let id_str = id.as_str();
                    let id_parts: Vec<&str> = id_str.splitn(3, "::").collect();
                    if id_parts.len() >= 2 {
                        id_parts[0] == module && id_parts[1] == function
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            match matching.len() {
                0 => {
                    return Err(format!(
                        "No matching entry point found for '{}::{}'. Available functions: {:?}",
                        module,
                        function,
                        graph
                            .nodes
                            .keys()
                            .take(5)
                            .map(trackast_lib::function_id::FunctionId::as_str)
                            .collect::<Vec<_>>()
                    ))
                }
                1 => {
                    eprintln!(
                        "✓ Resolved entry point '{}::{}' to '{}'",
                        module, function, matching[0]
                    );
                    resolved.push(matching[0].clone());
                }
                _ => {
                    eprintln!(
                        "⚠ Entry point '{module}::{function}' matches multiple signatures, using all:"
                    );
                    for id in &matching {
                        eprintln!("  - {id}");
                    }
                    resolved.extend(matching);
                }
            }
        }
    }

    Ok(resolved)
}

fn detect_language(language: Option<String>, input_path: &Path) -> Result<Language, Box<dyn std::error::Error>> {
    let language = if let Some(lang_str) = language {
        match lang_str.to_lowercase().as_str() {
            "rust" | "rs" => Some(Language::Rust),
            "python" | "py" => Some(Language::Python),
            "javascript" | "js" | "typescript" | "ts" => Some(Language::JavaScript),
            _ => {
                eprintln!("Error: Unknown language '{lang_str}'");
                std::process::exit(1);
            }
        }
    } else {
        Language::from_file_path(input_path.to_str().unwrap())
    };

    language.ok_or_else(|| {
        "Error: Could not detect language from file extension. Use --language to specify.".into()
    })
}

fn load_ast(
    language: Language,
    input_path: &Path,
    root_dir: &Path,
    module: Option<String>,
    no_discover: bool,
) -> Result<AbstractAST, Box<dyn std::error::Error>> {
    if no_discover {
        eprintln!("📄 Loading single file (dependencies disabled)");
        let translator = trackast::translator_factory::get_translator(language);
        let module = module.unwrap_or_else(|| {
            input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("root")
                .to_string()
        });
        translator.translate_file(input_path.to_str().unwrap(), Some(&module)).map_err(Into::into)
    } else {
        eprintln!("🔍 Auto-discovering module dependencies...");
        let mut loader = ModuleLoader::new(root_dir, language);
        loader.load_all(input_path.to_str().unwrap()).map_err(Into::into)
    }
}

fn build_output(
    format: &str,
    graph: &CallGraph,
    language: Language,
    entry_points: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    if entry_points.is_empty() {
        match format {
            "json" => {
                Ok(serde_json::json!({
                    "language": language.as_str(),
                    "nodes": graph.node_count(),
                    "edges": graph.edge_count(),
                    "message": "Call graph built successfully"
                })
                .to_string())
            }
            "dot" => Ok(to_dot(graph)),
            _ => unreachable!(),
        }
    } else {
        eprintln!("🔍 Resolving entry points...");
        let entry_ids = resolve_entry_points(entry_points, graph)?;
        eprintln!("📍 Using {} entry point(s)", entry_ids.len());

        let traversal_result = trackast_lib::traversal::traversal_from_entries(graph, &entry_ids);
        let reachable_count = traversal_result.reachable.len();
        eprintln!("🌳 Reachable functions from entry points: {reachable_count}");

        match format {
            "json" => {
                Ok(serde_json::json!({
                    "language": language.as_str(),
                    "total_nodes": graph.node_count(),
                    "total_edges": graph.edge_count(),
                    "entry_points": entry_points,
                    "reachable_functions": reachable_count,
                    "reachable_ids": traversal_result.reachable.iter().map(std::string::ToString::to_string).collect::<Vec<_>>(),
                })
                .to_string())
            }
            "dot" => {
                let mut reachable_graph = trackast_lib::graph::CallGraph::new();
                for node_id in &traversal_result.reachable {
                    if let Some(node) = graph.get_node(node_id) {
                        reachable_graph.insert_node(node.clone()).ok();
                    }
                }
                for edge in &graph.edges {
                    if traversal_result.reachable.contains(&edge.from)
                        && traversal_result.reachable.contains(&edge.to)
                    {
                        reachable_graph.insert_edge(edge.clone()).ok();
                    }
                }
                Ok(to_dot(&reachable_graph))
            }
            _ => unreachable!(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check if file exists
    if !args.input.exists() {
        eprintln!("Error: Input file does not exist: {}", args.input.display());
        std::process::exit(1);
    }

    // Validate format
    if args.format != "json" && args.format != "dot" {
        eprintln!("Error: Unknown format '{}'. Use 'json' or 'dot'", args.format);
        std::process::exit(1);
    }

    // Detect language
    let language = detect_language(args.language, &args.input)?;
    eprintln!("📝 Detected language: {}", language.as_str());

    // Determine root directory for module resolution
    let root_dir = args.root.unwrap_or_else(|| {
        args.input
            .parent().map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
    });
    eprintln!("📂 Using root directory: {}", root_dir.display());

    // Load AST
    let ast = load_ast(language, &args.input, &root_dir, args.module, args.no_discover)?;
    eprintln!("📦 Found {} functions", ast.functions.len());

    // Build call graph
    let mut builder = CallGraphBuilder::new();
    builder.add_ast(ast)?;
    let graph = builder.build()?;
    eprintln!("🔗 Built graph with {} nodes and {} edges", graph.node_count(), graph.edge_count());

    // Generate output
    let output = build_output(&args.format, &graph, language, &args.entry_points)?;

    // Write output
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &output)?;
        eprintln!("✅ Output written to {}", output_path.display());
    } else {
        println!("{output}");
    }

    Ok(())
}
