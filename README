# Markdown File Concatenator (`md_concat`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A command-line utility written in Rust to recursively search a directory for files with specified extensions, sort them, and concatenate their contents into a single Markdown file. Each file's content is placed within a fenced code block, labeled with the file's relative path.

This tool is useful for:

*   Creating a single context file of a project's source code for analysis or documentation.
*   Providing large codebases as context to Large Language Models (LLMs).
*   Generating simple code snapshots.

## ✨ Features

*   **Recursive Directory Search:** Scans the specified root directory and all its subdirectories.
*   **Extension Filtering:** Includes only files matching the provided list of extensions (e.g., `.c`, `.h`, `.rs`, `.py`).
*   **Directory Exclusion:** Allows specifying directory names (like `target`, `.git`, `node_modules`) to exclude from the search.
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

*   `--root-dir=<PATH>`: The root directory to start searching from.
    *   Defaults to the current directory (`.`).
    *   Example: `--root-dir=./src`

*   `--exclude-dirs=<DIR1,DIR2,...>`: A comma-separated list of directory *names* to exclude from the search. Any directory matching one of these names will be skipped.
    *   Defaults to `""` (none excluded).
    *   Common usage: `--exclude-dirs=.git,target,node_modules,vendor`
    *   Example: `--exclude-dirs=build,dist`

*   `-h, --help`: Print help information.
*   `-V, --version`: Print version information.

## 📝 Examples

1.  **Concatenate all `.rs` and `.toml` files in the current directory and subdirectories into `project_summary.md`:**
    ```bash
    ./target/release/md_concat project_summary.md --extensions=rs,toml
    ```

2.  **Concatenate `.c` and `.h` files from the `src/` directory, excluding `build` and `tests` subdirectories, into `code.md`:**
    ```bash
    ./target/release/md_concat code.md --root-dir=src --extensions=c,h --exclude-dirs=build,tests
    ```

3.  **Concatenate Python (`.py`) and JavaScript (`.js`) files from the current directory, excluding `.git` and `node_modules`, into `web_app.md`:**
    ```bash
    ./target/release/md_concat web_app.md --extensions=py,js --exclude-dirs=.git,node_modules
    ```

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests on GitHub.

*   Please ensure code is formatted using `cargo fmt`.
*   Run `cargo clippy` to catch common issues.
*   Add tests if introducing new functionality.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```