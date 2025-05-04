# codebase-to-markdown
Convert your codebase into a formatted markdown file for inference.

This tool converts all text files in your project into a neatly formatted markdown file (`codebase.md`), respecting files and directories defined in `.gitignore`. It offers two methods to achieve this: a Bash script (`codebase.sh`) and a Rust CLI program (`codebase-to-markdown`).

---

## Prerequisites

### For the Bash Script
1. A POSIX-compatible terminal (e.g., Git Bash, Bash, Zsh).
2. The `tree` command-line tool installed.

### For the Rust CLI
1. Rust and Cargo installed. Install via [rustup](https://www.rust-lang.org/tools/install).

---

## How to Use

### Option 1: Using the Rust CLI
1. Install the CLI tool using Cargo:
   ```bash
   cargo install --git https://github.com/Quantaindew/codebase-to-markdown
   ```
2. Run the tool:
   ```bash
   codebase-to-markdown
   ```
3. Your codebase will be formatted into `codebase.md`.

### Option 2: Using the Bash Script
1. Ensure the script has executable permissions after downloading:
   ```bash
   chmod +x ./codebase.sh
   ```
2. Run the script:
   ```bash
   ./codebase.sh
   ```
3. Your codebase will be formatted into `codebase.md`.

---

## Output
In both methods, the output is a `codebase.md` file containing:
- A project structure tree.
- The contents of all text files (excluding binary/image files and those in `.gitignore`), wrapped in `<file src="...">` tags.

---

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.