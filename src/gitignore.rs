use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages gitignore files and provides filtering functionality
pub struct GitignoreManager {
    /// Maps directory paths to their compiled gitignore rules
    ignores: HashMap<PathBuf, Gitignore>,
    /// Global gitignore patterns that apply to all files
    global_ignore: Option<Gitignore>,
}

impl GitignoreManager {
    /// Creates a new GitignoreManager
    pub fn new() -> Self {
        Self {
            ignores: HashMap::new(),
            global_ignore: None,
        }
    }

    /// Discovers and loads all gitignore files in the given input directories
    pub fn discover_and_load(
        input_dirs: &[PathBuf],
        additional_gitignore_files: &[PathBuf],
    ) -> Result<Self, Box<dyn Error>> {
        let mut manager = Self::new();

        // First, discover all gitignore files in input directories
        let mut gitignore_files = HashMap::new();

        for input_dir in input_dirs {
            if let Ok(canonical_dir) = fs::canonicalize(input_dir) {
                manager.discover_gitignore_files_recursive(&canonical_dir, &mut gitignore_files)?;
            }
        }

        // Add any additional gitignore files specified by user
        for gitignore_file in additional_gitignore_files {
            if let Ok(canonical_file) = fs::canonicalize(gitignore_file) {
                if let Some(parent_dir) = canonical_file.parent() {
                    gitignore_files.insert(parent_dir.to_path_buf(), canonical_file);
                }
            }
        }

        // Build gitignore rules for each directory
        for (dir_path, gitignore_path) in gitignore_files {
            match manager.build_gitignore_for_directory(&dir_path, &gitignore_path) {
                Ok(gitignore) => {
                    manager.ignores.insert(dir_path, gitignore);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse gitignore file {}: {}",
                        gitignore_path.display(),
                        e
                    );
                }
            }
        }

        Ok(manager)
    }

    /// Recursively discovers gitignore files in a directory
    fn discover_gitignore_files_recursive(
        &self,
        dir_path: &Path,
        gitignore_files: &mut HashMap<PathBuf, PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        if !dir_path.is_dir() {
            return Ok(());
        }

        // Check for .gitignore in current directory
        let gitignore_path = dir_path.join(".gitignore");
        if gitignore_path.exists() && gitignore_path.is_file() {
            gitignore_files.insert(dir_path.to_path_buf(), gitignore_path);
        }

        // Recursively check subdirectories
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Don't recurse into hidden directories except .git
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') && name != ".git" {
                            continue;
                        }
                    }
                    self.discover_gitignore_files_recursive(&path, gitignore_files)?;
                }
            }
        }

        Ok(())
    }

    /// Builds gitignore rules for a specific directory
    fn build_gitignore_for_directory(
        &self,
        dir_path: &Path,
        gitignore_path: &Path,
    ) -> Result<Gitignore, Box<dyn Error>> {
        let mut builder = GitignoreBuilder::new(dir_path);

        // Add the gitignore file
        builder.add(gitignore_path);

        // Build and return the gitignore
        Ok(builder.build()?)
    }

    /// Checks if a file should be ignored based on all applicable gitignore rules
    pub fn should_ignore(&self, file_path: &Path, relative_path: &Path) -> bool {
        // Check global ignore first
        if let Some(ref global_ignore) = self.global_ignore {
            if global_ignore
                .matched(relative_path, file_path.is_dir())
                .is_ignore()
            {
                return true;
            }
        }

        // Check directory-specific ignores
        // We need to find the most specific gitignore that applies to this file
        let mut best_match_dir: Option<&Path> = None;
        let mut best_match_depth = 0;

        for dir_path in self.ignores.keys() {
            if file_path.starts_with(dir_path) {
                let depth = dir_path.components().count();
                if depth > best_match_depth {
                    best_match_depth = depth;
                    best_match_dir = Some(dir_path);
                }
            }
        }

        if let Some(matching_dir) = best_match_dir {
            if let Some(gitignore) = self.ignores.get(matching_dir) {
                // Calculate relative path from the gitignore directory
                if let Ok(rel_from_gitignore) = file_path.strip_prefix(matching_dir) {
                    return gitignore
                        .matched(rel_from_gitignore, file_path.is_dir())
                        .is_ignore();
                }
            }
        }

        false
    }

    /// Checks if a directory should be ignored (for early pruning during traversal)
    pub fn should_ignore_directory(&self, dir_path: &Path) -> bool {
        self.should_ignore(dir_path, dir_path)
    }
}

impl Default for GitignoreManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Collects files with gitignore filtering applied
pub fn collect_files_with_gitignore(
    input_dirs: &[PathBuf],
    extensions: &std::collections::HashSet<String>,
    exclude_dirs: &std::collections::HashSet<String>,
    gitignore_manager: &GitignoreManager,
    respect_gitignore: bool,
) -> Vec<(PathBuf, PathBuf)> {
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let mut found_files = Vec::new();
    let mut processed_files = HashSet::new();

    for input_dir in input_dirs {
        let walker = WalkDir::new(input_dir).follow_links(false);

        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Warning: Failed to access entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            // Skip if this is a directory and it's in exclude_dirs
            if entry.file_type().is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if exclude_dirs.contains(dir_name) {
                        continue;
                    }
                }

                // If respecting gitignore, check if directory should be ignored
                if respect_gitignore && gitignore_manager.should_ignore_directory(path) {
                    continue;
                }
            }

            if entry.file_type().is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(ext) {
                        // Get canonical path for deduplication
                        let canonical_file_path = match std::fs::canonicalize(path) {
                            Ok(p) => p,
                            Err(_) => path.to_path_buf(),
                        };

                        // Check if we've already processed this file
                        if processed_files.contains(&canonical_file_path) {
                            continue;
                        }

                        // Apply gitignore filtering if enabled
                        if respect_gitignore {
                            if let Ok(rel_path) = path.strip_prefix(input_dir) {
                                if gitignore_manager.should_ignore(path, rel_path) {
                                    continue;
                                }
                            }
                        }

                        // Add to results
                        if let Ok(rel_path) = path.strip_prefix(input_dir) {
                            found_files.push((rel_path.to_path_buf(), canonical_file_path.clone()));
                            processed_files.insert(canonical_file_path);
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

    // Sort by relative path for consistent output
    found_files.sort_by(|a, b| a.0.cmp(&b.0));
    found_files
}
