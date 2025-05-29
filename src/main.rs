use clap::Parser;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

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
                } else {
                    println!("Note: Skipping duplicate directory: {}", input_dir.display());
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to canonicalize input directory {:?}: {}",
                    input_dir, e
                );
            }
        }
    }

    if valid_input_dirs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No valid input directories found",
        ));
    }

    let extensions: HashSet<String> = args.extensions.into_iter().collect();
    let exclude_dirs: HashSet<String> = args
        .exclude_dirs
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

    println!("Searching in {} directories:", valid_input_dirs.len());
    for dir in &valid_input_dirs {
        println!("  - {}", dir.display());
    }
    println!("Including extensions: {:?}", extensions);
    println!("Excluding directories: {:?}", exclude_dirs);
    println!("Outputting to: {}", output_file.display());

    let mut found_files = Vec::new();
    let mut processed_dirs = HashSet::new();

    // Process each input directory
    for input_dir in &valid_input_dirs {
        // Check if this directory is a subdirectory of any already processed directory
        let mut skip_dir = false;
        for processed_dir in &processed_dirs {
            if input_dir.starts_with(processed_dir) {
                println!("Note: Skipping {} (already covered by {})", 
                        input_dir.display(), processed_dir.display());
                skip_dir = true;
                break;
            }
        }
        
        if skip_dir {
            continue;
        }

        // Check if any previously processed directory is a subdirectory of current directory
        processed_dirs.retain(|processed_dir| {
            if processed_dir.starts_with(input_dir) {
                println!("Note: Directory {} superseded by {}", 
                        processed_dir.display(), input_dir.display());
                false // Remove from processed_dirs
            } else {
                true // Keep in processed_dirs
            }
        });

        processed_dirs.insert(input_dir.clone());

        // Use filter_entry to properly skip excluded directories during traversal
        let walker = WalkDir::new(input_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    if let Some(dir_name) = e.path().file_name().and_then(|n| n.to_str()) {
                        if exclude_dirs.contains(dir_name) && e.depth() > 0 {
                            return false;
                        }
                    }
                }
                true
            });

        for entry_result in walker {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Warning: Failed to access entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if entry.file_type().is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(ext) {
                        // Use absolute path for deduplication across different input directories
                        let canonical_file_path = match fs::canonicalize(path) {
                            Ok(p) => p,
                            Err(_) => path.to_path_buf(),
                        };
                        
                        // Check if we've already processed this file
                        if !found_files.iter().any(|(_, abs_path)| abs_path == &canonical_file_path) {
                            if let Ok(rel_path) = path.strip_prefix(input_dir) {
                                found_files.push((rel_path.to_path_buf(), canonical_file_path));
                            } else {
                                eprintln!(
                                    "Warning: Could not get relative path for {}",
                                    path.display()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by relative path for consistent output
    found_files.sort_by(|a, b| a.0.cmp(&b.0));

    let output_file_handle = File::create(output_file)?;
    let mut writer = BufWriter::new(output_file_handle);

    println!("\nConcatenating {} files...", found_files.len());

    for (rel_path, abs_path) in &found_files {
        let display_path = rel_path.display();
        let ext = rel_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        writeln!(writer, "## {}\n", display_path)?;
        writeln!(writer, "```{}", ext)?;

        match File::open(abs_path) {
            Ok(mut input_file) => {
                let mut buffer = String::new();
                if input_file.read_to_string(&mut buffer).is_ok() {
                    write!(writer, "{}", buffer)?;
                    if !buffer.ends_with('\n') {
                        writeln!(writer)?;
                    }
                } else {
                    eprintln!(
                        "Warning: Failed to read file content (possibly not UTF-8): {}",
                        abs_path.display()
                    );
                    write!(
                        writer,
                        "\nError: Could not read file content (e.g., binary or non-UTF-8)"
                    )?;
                    writeln!(writer)?;
                }
            }
            Err(e) => {
                eprintln!("Error opening file {}: {}", abs_path.display(), e);
                write!(writer, "\nError: Could not open file: {}", e)?;
                writeln!(writer)?;
            }
        }

        writeln!(writer, "```\n")?;
    }

    writer.flush()?;

    println!(
        "Successfully concatenated {} files into {}",
        found_files.len(),
        output_file.display()
    );

    Ok(())
}
