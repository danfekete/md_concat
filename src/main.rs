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

    /// The root directory to search for files (defaults to current directory).
    #[arg(long = "root-dir", default_value = ".")]
    root_dir: PathBuf,

    /// Comma-separated list of file extensions to include (e.g., "c,h,rs").
    #[arg(long, value_delimiter = ',', required = true)]
    extensions: Vec<String>,

    /// Comma-separated list of directory names to exclude from search (e.g., "target,.git,build").
    #[arg(long = "exclude-dirs", value_delimiter = ',', default_value = "")]
    exclude_dirs: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    // --- Argument Processing ---
    let output_file = &args.output_file;
    // Ensure root_dir is canonicalized to handle relative paths like "." correctly
    let root_dir = fs::canonicalize(&args.root_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to canonicalize root directory {:?}: {}", args.root_dir, e),
        )
    })?;
    // Store extensions and excluded directories in HashSets for efficient lookup
    let extensions: HashSet<String> = args.extensions.into_iter().collect();
    let exclude_dirs: HashSet<String> = args.exclude_dirs.into_iter().filter(|s| !s.is_empty()).collect(); // Filter out empty strings resulting from default_value=""

    println!("Searching in: {}", root_dir.display());
    println!("Including extensions: {:?}", extensions);
    println!("Excluding directories: {:?}", exclude_dirs);
    println!("Outputting to: {}", output_file.display());

    // --- Find Files ---
    let mut found_files = Vec::new();
    let walker = WalkDir::new(&root_dir).follow_links(false); // Don't follow symlinks by default

    for entry_result in walker {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Warning: Failed to access entry: {}", e);
                continue; // Skip problematic entries
            }
        };

        let path = entry.path();

        // --- Directory Exclusion ---
        if entry.file_type().is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if exclude_dirs.contains(dir_name) {
                    // Tell walkdir not to recurse into this directory
                    // Note: This requires walkdir >= 2.4.0 for set_skip_current_dir
                     if entry.depth() > 0 { // Only skip if not the root itself
                        // walkdir doesn't have a direct "skip this dir" after finding it easily,
                        // but filtering it out here prevents processing its contents *later*.
                        // A more robust way would be filtering *during* iteration if walkdir offered it directly.
                        // For now, we just skip processing files *within* it by checking the path prefix later.
                        // Let's refine the filtering:
                         if exclude_dirs.iter().any(|ex_dir| path.components().any(|comp| comp.as_os_str() == ex_dir.as_str())) {
                            // If any component of the path is an excluded dir name, skip.
                            // This isn't perfect as it might exclude /home/user/src/my_target if 'target' is excluded.
                            // A better walkdir filter approach is needed for precise directory skipping.
                            // walkdir's into_iter().filter_entry() is the proper way:
                            // Let's rewrite the loop using filter_entry for correct pruning.
                            // (Rewriting below)
                         }
                    }
                }
            }
            continue; // Only process files
        }

        // --- File Filtering ---
        if entry.file_type().is_file() {
            // Check extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(ext) {
                    // Check if the file is within an excluded directory path
                    let is_in_excluded_dir = path.components().any(|comp| {
                        comp.as_os_str()
                            .to_str()
                            .map_or(false, |name| exclude_dirs.contains(name))
                    });

                    if !is_in_excluded_dir {
                        // Get relative path
                        if let Ok(rel_path) = path.strip_prefix(&root_dir) {
                             found_files.push(rel_path.to_path_buf());
                        } else {
                             eprintln!("Warning: Could not get relative path for {}", path.display());
                        }
                    }
                }
            }
        }
    }

    // --- Let's rewrite the file finding loop using walkdir's filter_entry for proper pruning ---
    found_files.clear(); // Clear previous attempt
    let walker = WalkDir::new(&root_dir)
        .follow_links(false) // Don't follow symlinks
        .into_iter()
        .filter_entry(|e| {
            // Filter out excluded directories *before* recursing into them
            if e.file_type().is_dir() {
                if let Some(dir_name) = e.file_name().to_str() {
                     // Check if the directory name itself is excluded (and it's not the root)
                    if exclude_dirs.contains(dir_name) && e.depth() > 0 {
                         return false; // Don't enter this directory
                    }
                }
            }
            true // Keep other entries (files, allowed directories)
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

        // Process only files
        if entry.file_type().is_file() {
            // Check extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(ext) {
                     // Get relative path
                    if let Ok(rel_path) = path.strip_prefix(&root_dir) {
                        found_files.push(rel_path.to_path_buf());
                    } else {
                        eprintln!("Warning: Could not get relative path for {}", path.display());
                        // Fallback: use the absolute path? Or skip? Skipping seems safer.
                    }
                }
            }
        }
    }


    // --- Sort Files ---
    found_files.sort();

    // --- Concatenate to Markdown ---
    let output_file_handle = File::create(output_file)?;
    let mut writer = BufWriter::new(output_file_handle);

    println!("\nConcatenating {} files...", found_files.len());

    for rel_path in &found_files {
        let full_path = root_dir.join(rel_path);
        let display_path = rel_path.display(); // Use relative path for display

        // Get extension for code block fence
        let ext = rel_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Write header and open code block
        writeln!(writer, "## {}\n", display_path)?; // Add newline after header
        writeln!(writer, "```{}", ext)?; // No newline immediately after fence start

        // Copy file contents
        match File::open(&full_path) {
            Ok(mut input_file) => {
                let mut buffer = String::new(); // Read as string for simplicity
                if input_file.read_to_string(&mut buffer).is_ok() {
                     // Ensure newline before content if file isn't empty
                    if !buffer.is_empty() && !buffer.starts_with('\n') {
                        // writeln!(writer)?; // Start content on new line relative to ```
                    }
                     // Write content, ensure newline before closing fence
                    write!(writer, "{}", buffer)?;
                    if !buffer.ends_with('\n') {
                        writeln!(writer)?; // Ensure the ``` starts on its own line
                    }
                } else {
                    eprintln!("Warning: Failed to read file content (possibly not UTF-8): {}", full_path.display());
                    write!(writer, "\nError: Could not read file content (e.g., binary or non-UTF-8)")?;
                    writeln!(writer)?; // Ensure newline before closing fence
                }
            }
            Err(e) => {
                eprintln!("Error opening file {}: {}", full_path.display(), e);
                 // Write error message inside code block
                write!(writer, "\nError: Could not open file: {}", e)?;
                 writeln!(writer)?; // Ensure newline before closing fence
            }
        }

        // Close code block
        writeln!(writer, "```\n")?; // Add newline after closing block
    }

    writer.flush()?; // Ensure all buffered data is written

    println!(
        "Successfully concatenated {} files into {}",
        found_files.len(),
        output_file.display()
    );

    Ok(())
}