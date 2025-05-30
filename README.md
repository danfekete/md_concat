# Markdown File Concatenator (`md_concat`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A command-line utility written in Rust to recursively search directories for files with specified extensions, sort them, and concatenate their contents into a single Markdown file. Each file's content is placed within a fenced code block, labeled with the file's relative path. The tool automatically respects `.gitignore` files to exclude unwanted files and provides comprehensive token count estimates for LLM usage.

This tool is useful for:

*   Creating a single context file of a project's source code for analysis or documentation.
*   Providing large codebases as context to Large Language Models (LLMs) with **automatic token count estimates**.
*   Generating simple code snapshots with **memory-efficient processing** and **automatic gitignore filtering**.
*   **Analyzing token usage** before submitting to different LLM providers.

## ✨ Features

*   **Multiple Input Directories:** Process multiple directories with automatic deduplication and hierarchy optimization.
*   **Recursive Directory Search:** Scans the specified directories and all their subdirectories.
*   **Extension Filtering:** Includes only files matching the provided list of extensions (e.g., `.c`, `.h`, `.rs`, `.py`).
*   **Directory Exclusion:** Allows specifying directory names (like `target`, `.git`, `node_modules`) to exclude from the search.
*   **Gitignore Support:** Automatically respects `.gitignore` files found in input directories:
    *   **Automatic Discovery:** Finds and applies `.gitignore` files in any input directory
    *   **Hierarchical Rules:** Properly handles gitignore inheritance from parent directories
    *   **Additional Gitignore Files:** Supports specifying extra gitignore files with `--additional-gitignore`
    *   **Toggleable:** Can be disabled with `--no-gitignore` flag
*   **Token Count Estimation:** Automatically estimates token counts for different LLM providers:
    *   **GPT-style** (~4 chars/token) - OpenAI models
    *   **Claude-style** (~3.5 chars/token) - Anthropic models  
    *   **Conservative** (~3 chars/token) - Safe estimate for most LLMs
    *   **Word-based** (~5 chars/token) - Alternative approach
*   **Memory Efficient:** Processes files incrementally without storing entire content in memory.
*   **Markdown Output:** Generates a clean Markdown file with:
    *   Level 2 headings (`##`) containing the relative path of each file.
    *   Fenced code blocks (``` ```) with language hints based on the file extension.
*   **Sorted Output:** Files are included in alphabetical order based on their relative paths.
*   **Robust Argument Parsing:** Uses `clap` for clear and user-friendly command-line arguments and help messages.
*   **Cross-Platform:** Built with Rust, works on Linux, macOS, and Windows.

##  Prerequisites

*   **Rust Toolchain:** You need `rustc` and `cargo` installed. If you don't have them, install Rust via [rustup.rs](https://rustup.rs/).

## ⚙️ Installation / Building

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/danfekete/md_concat.git
    cd md_concat
    ```

2.  **Build the project:**
    *   For a development build:
        ```bash
        cargo build
        ```
        The executable will be located at `target/debug/md_concat`.

    *   For an optimized release build:
        ```bash
        cargo build --release
        ```
        The executable will be located at `target/release/md_concat`.

3.  **(Optional) Add to PATH:** You can copy the executable from the `target/...` directory to a location in your system's PATH (e.g., `/usr/local/bin` or `~/.local/bin`) for easier access.

## 🚀 Usage

```bash
md_concat <OUTPUT_FILE> --extensions=<EXT1,EXT2,...> [OPTIONS]
```

### Arguments

*   `<OUTPUT_FILE>`: (Required) The path to the output Markdown file that will be created or overwritten.

### Options

*   `--extensions=<EXT1,EXT2,...>`: (Required) A comma-separated list of file extensions to include (without the leading dot).
    *   Example: `--extensions=rs,toml`

*   `--input-dirs=<DIR1,DIR2,...>`: A comma-separated list of input directories to search.
    *   Defaults to the current directory (`.`).
    *   Supports multiple directories with automatic deduplication and hierarchy optimization.
    *   Example: `--input-dirs=./src,./tests,./examples`

*   `--exclude-dirs=<DIR1,DIR2,...>`: A comma-separated list of directory *names* to exclude from the search. Any directory matching one of these names will be skipped.
    *   Defaults to `""` (none excluded).
    *   Common usage: `--exclude-dirs=.git,target,node_modules,vendor`
    *   Example: `--exclude-dirs=build,dist`

*   `--no-gitignore`: Disables automatic `.gitignore` file detection and filtering.
    *   By default, the tool will find and respect `.gitignore` files in input directories.
    *   Use this flag to ignore `.gitignore` rules and include all files matching other criteria.

*   `--additional-gitignore=<FILE1,FILE2,...>`: A comma-separated list of additional `.gitignore` files to consider.
    *   These files will be applied in addition to any automatically discovered `.gitignore` files.
    *   Useful for applying custom ignore rules or using `.gitignore` files from different locations.
    *   Example: `--additional-gitignore=.myignore,../shared.gitignore`

*   `-h, --help`: Print help information.
*   `-V, --version`: Print version information.

## 📝 Examples

1.  **Concatenate all `.rs` and `.toml` files in the current directory and subdirectories into `project_summary.md`:**
    ```bash
    ./target/release/md_concat project_summary.md --extensions=rs,toml
    ```

2.  **Concatenate `.c` and `.h` files from multiple directories, excluding `build` and `tests` subdirectories, into `code.md`:**
    ```bash
    ./target/release/md_concat code.md --input-dirs=src,include --extensions=c,h --exclude-dirs=build,tests
    ```

3.  **Concatenate Python (`.py`) and JavaScript (`.js`) files from the current directory, excluding `.git` and `node_modules`, into `web_app.md`:**
    ```bash
    ./target/release/md_concat web_app.md --extensions=py,js --exclude-dirs=.git,node_modules
    ```

4.  **Process multiple source directories with token counting for LLM context preparation:**
    ```bash
    ./target/release/md_concat context.md --input-dirs=src,lib,examples --extensions=rs --exclude-dirs=target
    ```
    
    This will output something like:
    ```
    Successfully concatenated 25 files into context.md
    
    === Token Count Estimates ===
    Characters: 45,823
    Words: 3,847
    
    Conservative: ~15,274 tokens
    Claude-style: ~13,092 tokens
    GPT-style: ~11,456 tokens
    Word-based: ~9,165 tokens
    ```

5.  **Using gitignore functionality to automatically exclude files:**
    ```bash
    ./target/release/md_concat project.md --extensions=py,js,html --input-dirs=src,frontend
    ```
    This will automatically respect any `.gitignore` files found in `src/` or `frontend/` directories.

6.  **Disable gitignore and include all files (useful for comprehensive analysis):**
    ```bash
    ./target/release/md_concat complete_project.md --extensions=rs --no-gitignore
    ```

7.  **Use custom gitignore files for specialized filtering:**
    ```bash
    ./target/release/md_concat filtered.md --extensions=cpp,h --additional-gitignore=.buildignore,tests/.testignore
    ```

## 📊 Token Count Estimates

The tool automatically provides token count estimates for different LLM providers after processing:

- **Character count**: Total characters in the output
- **Word count**: Total words in the output  
- **Conservative estimate**: Safest estimate for most LLMs (~3 chars/token)
- **Claude-style**: Optimized for Anthropic's Claude models (~3.5 chars/token)
- **GPT-style**: Tailored for OpenAI's GPT models (~4 chars/token)
- **Word-based**: Alternative estimation based on average word length (~5 chars/token)

This helps you understand the scope of your codebase before submitting to LLM providers and plan accordingly for token limits.

## 🚫 Gitignore Support

The tool includes comprehensive `.gitignore` support to automatically exclude files that shouldn't be included in your concatenated output:

### Automatic Discovery
- Scans all input directories for `.gitignore` files
- Applies gitignore rules hierarchically (parent directory rules apply to subdirectories)
- Respects the standard gitignore pattern syntax

### How It Works
1. **Discovery Phase**: Finds all `.gitignore` files in input directories and subdirectories
2. **Rule Compilation**: Parses and compiles gitignore patterns for efficient matching
3. **File Filtering**: During traversal, checks each file against applicable gitignore rules
4. **Hierarchical Application**: Files are checked against the most specific gitignore rules that apply

### Integration with Other Filters
Gitignore filtering works alongside other filtering mechanisms:
1. **Extension filtering** (`--extensions`) - Only files with specified extensions are considered
2. **Directory exclusion** (`--exclude-dirs`) - Explicitly excluded directories are skipped
3. **Gitignore filtering** - Files matching gitignore patterns are excluded
4. **Deduplication** - Ensures the same file isn't included multiple times

### Control Options
- **Enabled by default**: Gitignore support is active unless explicitly disabled
- **`--no-gitignore`**: Completely disables gitignore processing for maximum inclusion
- **`--additional-gitignore`**: Add custom gitignore files beyond those automatically discovered
- **Graceful degradation**: If gitignore parsing fails, continues with warnings


## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests on GitHub.

*   Please ensure code is formatted using `cargo fmt`.
*   Run `cargo clippy` to catch common issues.
*   Add tests if introducing new functionality.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.