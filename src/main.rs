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

    let output_file = &args.output_file;
    let root_dir = fs::canonicalize(&args.root_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to canonicalize root directory {:?}: {}",
                args.root_dir, e
            ),
        )
    })?;
    let extensions: HashSet<String> = args.extensions.into_iter().collect();
    let exclude_dirs: HashSet<String> = args
        .exclude_dirs
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

    println!("Searching in: {}", root_dir.display());
    println!("Including extensions: {:?}", extensions);
    println!("Excluding directories: {:?}", exclude_dirs);
    println!("Outputting to: {}", output_file.display());

    let mut found_files = Vec::new();
    // Use filter_entry to properly skip excluded directories during traversal
    let walker = WalkDir::new(&root_dir)
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
                    if let Ok(rel_path) = path.strip_prefix(&root_dir) {
                        found_files.push(rel_path.to_path_buf());
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

    found_files.sort();

    let output_file_handle = File::create(output_file)?;
    let mut writer = BufWriter::new(output_file_handle);

    println!("\nConcatenating {} files...", found_files.len());

    for rel_path in &found_files {
        let full_path = root_dir.join(rel_path);
        let display_path = rel_path.display();
        let ext = rel_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        writeln!(writer, "## {}\n", display_path)?;
        writeln!(writer, "```{}", ext)?;

        match File::open(&full_path) {
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
                        full_path.display()
                    );
                    write!(
                        writer,
                        "\nError: Could not read file content (e.g., binary or non-UTF-8)"
                    )?;
                    writeln!(writer)?;
                }
            }
            Err(e) => {
                eprintln!("Error opening file {}: {}", full_path.display(), e);
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
