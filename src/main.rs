use clap::Parser;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;

mod gitignore;
use gitignore::{GitignoreManager, collect_files_with_gitignore};

/// Token counting strategies for different LLMs
#[derive(Debug, Clone)]
pub enum TokenCountStrategy {
    /// GPT-style tokenization (roughly 4 chars per token)
    Gpt,
    /// Claude-style tokenization (roughly 3.5 chars per token)
    Claude,
    /// Conservative estimate (roughly 3 chars per token)
    Conservative,
    /// Character count divided by average word length
    WordBased,
}

impl TokenCountStrategy {
    fn chars_per_token(&self) -> f64 {
        match self {
            TokenCountStrategy::Gpt => 4.0,
            TokenCountStrategy::Claude => 3.5,
            TokenCountStrategy::Conservative => 3.0,
            TokenCountStrategy::WordBased => 5.0, // Average word length + space
        }
    }

    fn name(&self) -> &str {
        match self {
            TokenCountStrategy::Gpt => "GPT-style",
            TokenCountStrategy::Claude => "Claude-style",
            TokenCountStrategy::Conservative => "Conservative",
            TokenCountStrategy::WordBased => "Word-based",
        }
    }
}

/// Estimates tokens for multiple LLM strategies and returns a formatted report
pub fn estimate_tokens_report(char_count: usize, word_count: usize) -> String {
    let strategies = vec![
        TokenCountStrategy::Conservative,
        TokenCountStrategy::Claude,
        TokenCountStrategy::Gpt,
        TokenCountStrategy::WordBased,
    ];

    let mut report = String::new();
    report.push_str(&format!("=== Token Count Estimates ===\n"));
    report.push_str(&format!("Characters: {}\n", char_count));
    report.push_str(&format!("Words: {}\n\n", word_count));

    for strategy in strategies {
        let estimated_tokens = char_count as f64 / strategy.chars_per_token();
        let token_count = estimated_tokens.ceil() as usize;
        report.push_str(&format!("{}: ~{} tokens\n", strategy.name(), token_count));
    }

    report
}

/// Incremental token counter to avoid storing all content in memory
#[derive(Default)]
pub struct TokenCounter {
    pub char_count: usize,
    pub word_count: usize,
}

impl TokenCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_text(&mut self, text: &str) {
        self.char_count += text.chars().count();
        self.word_count += text.split_whitespace().count();
    }

    pub fn get_token_estimates(&self) -> String {
        estimate_tokens_report(self.char_count, self.word_count)
    }
}

/// Estimates the number of tokens in a text string using the specified strategy
pub fn estimate_tokens(text: &str, strategy: &TokenCountStrategy) -> usize {
    let char_count = text.chars().count();
    let estimated_tokens = char_count as f64 / strategy.chars_per_token();
    estimated_tokens.ceil() as usize
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The output Markdown file path.
    output_file: PathBuf,

    /// Input directories to search for files (defaults to current directory if none provided).
    #[arg(long = "input-dirs", value_delimiter = ',', default_value = ".")]
    input_dirs: Vec<PathBuf>,

    /// Comma-separated list of file extensions to include (e.g., "c,h,rs").
    #[arg(long, value_delimiter = ',', required = true)]
    extensions: Vec<String>,

    /// Comma-separated list of directory names to exclude from search (e.g., "target,.git,build").
    #[arg(long = "exclude-dirs", value_delimiter = ',', default_value = "")]
    exclude_dirs: Vec<String>,

    /// Whether to respect .gitignore files (default: true)
    #[arg(long = "no-gitignore", action = clap::ArgAction::SetFalse)]
    respect_gitignore: bool,

    /// Additional gitignore files to consider
    #[arg(long = "additional-gitignore", value_delimiter = ',')]
    additional_gitignore_files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    let output_file = &args.output_file;

    // Canonicalize all input directories and deduplicate them
    let mut canonical_dirs = HashSet::new();
    let mut valid_input_dirs = Vec::new();

    for input_dir in &args.input_dirs {
        match fs::canonicalize(input_dir) {
            Ok(canonical_path) => {
                if canonical_dirs.insert(canonical_path.clone()) {
                    valid_input_dirs.push(canonical_path);
                    println!("Input directory: {}", input_dir.display());
                } else {
                    println!(
                        "Skipping duplicate directory: {} (same as already included directory)",
                        input_dir.display()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Error: Input directory '{}' is not accessible: {}",
                    input_dir.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    // Convert extensions to a HashSet for O(1) lookup
    let extensions: HashSet<String> = args.extensions.into_iter().collect();
    println!("Extensions: {:?}", extensions);

    // Convert exclude_dirs to a HashSet for O(1) lookup, filtering out empty strings
    let exclude_dirs: HashSet<String> = args
        .exclude_dirs
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

    if !exclude_dirs.is_empty() {
        println!("Excluding directories: {:?}", exclude_dirs);
    }

    // Initialize gitignore manager if needed
    let gitignore_manager = if args.respect_gitignore {
        match GitignoreManager::discover_and_load(
            &valid_input_dirs,
            &args.additional_gitignore_files,
        ) {
            Ok(manager) => {
                println!("Gitignore support enabled");
                Some(manager)
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize gitignore manager: {}", e);
                eprintln!("Continuing without gitignore support...");
                None
            }
        }
    } else {
        println!("Gitignore support disabled");
        None
    };

    // Collect files using the new system
    let found_files = if let Some(ref manager) = gitignore_manager {
        collect_files_with_gitignore(&valid_input_dirs, &extensions, &exclude_dirs, manager, true)
    } else {
        collect_files_with_gitignore(
            &valid_input_dirs,
            &extensions,
            &exclude_dirs,
            &GitignoreManager::new(),
            false,
        )
    };

    let output_file_handle = File::create(output_file)?;
    let mut writer = BufWriter::new(output_file_handle);
    let mut token_counter = TokenCounter::new();

    println!("\nConcatenating {} files...", found_files.len());

    for (rel_path, abs_path) in &found_files {
        let display_path = rel_path.display();
        let ext = rel_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let header = format!("## {}\n\n", display_path);
        let code_start = format!("```{}\n", ext);

        // Count tokens for markdown formatting
        token_counter.add_text(&header);
        token_counter.add_text(&code_start);

        writeln!(writer, "## {}\n", display_path)?;
        writeln!(writer, "```{}", ext)?;

        match File::open(abs_path) {
            Ok(mut input_file) => {
                let mut buffer = String::new();
                if input_file.read_to_string(&mut buffer).is_ok() {
                    token_counter.add_text(&buffer);
                    write!(writer, "{}", buffer)?;
                    if !buffer.ends_with('\n') {
                        token_counter.add_text("\n");
                        writeln!(writer)?;
                    }
                } else {
                    let error_msg =
                        "\nError: Could not read file content (e.g., binary or non-UTF-8)";
                    eprintln!(
                        "Warning: Failed to read file content (possibly not UTF-8): {}",
                        abs_path.display()
                    );
                    token_counter.add_text(error_msg);
                    token_counter.add_text("\n");
                    write!(writer, "{}", error_msg)?;
                    writeln!(writer)?;
                }
            }
            Err(e) => {
                let error_msg = format!("\nError: Could not open file: {}", e);
                eprintln!("Error opening file {}: {}", abs_path.display(), e);
                token_counter.add_text(&error_msg);
                token_counter.add_text("\n");
                write!(writer, "{}", error_msg)?;
                writeln!(writer)?;
            }
        }

        let code_end = "```\n\n";
        token_counter.add_text(code_end);
        writeln!(writer, "```\n")?;
    }

    writer.flush()?;

    println!(
        "Successfully concatenated {} files into {}",
        found_files.len(),
        output_file.display()
    );

    // Generate and display token count report
    println!("\n{}", token_counter.get_token_estimates());

    Ok(())
}
