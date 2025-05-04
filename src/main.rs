// src/main.rs
mod tree; // Declare the tree module

use crate::tree::Tree; // Import the Tree struct
use chrono::Local;
use content_inspector::ContentType;
use ignore::{self, WalkBuilder}; // Add ignore to imports if missing
use std::collections::HashMap; // Needed for building the tree structure
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

const OUTPUT_FILE: &str = "codebase.md";

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let start_time = Local::now();
    println!(
        "Starting script at {}",
        start_time.format("%Y-%m-%d %H:%M:%S")
    );

    let output_path = PathBuf::from(OUTPUT_FILE);
    let absolute_output_path = fs::canonicalize(&output_path).ok(); // Get absolute path if exists

    // Define filters early
    let output_path_filter = output_path.clone(); // Clone for the closure
    let filter = move |entry: &ignore::DirEntry| -> bool {
        let path = entry.path();
        // Filter out .git
        if path.file_name().map_or(false, |name| name == ".git") {
            return false;
        }
        // Filter out the output file itself (compare absolute paths if possible)
        if let Some(abs_output) = &absolute_output_path {
             if fs::canonicalize(path).ok() == Some(abs_output.clone()) {
                 return false;
             }
        } else if path == output_path_filter.as_path() { // Fallback to relative comparison
            return false;
        }
        true
    };


    // --- Collect all relevant paths first ---
    let mut all_paths: Vec<PathBuf> = Vec::new();
    let walker = WalkBuilder::new(".")
        .hidden(false)      // Process hidden files/dirs (like .gitignore)
        .parents(false)     // Don't include parent paths
        .ignore(true)       // Use .ignore files
        .git_global(true)   // Use global gitignore
        .git_ignore(true)   // Use .gitignore
        .git_exclude(true)  // Use .git/info/exclude
        .require_git(false) // Don't fail if not in git repo
        .sort_by_file_path(|a,b| a.cmp(b)) // Important for tree building
        .filter_entry(filter) // Use the combined filter
        .build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                // Only add paths relative to the root (depth > 0)
                 if entry.depth() > 0 {
                     // Strip './' prefix for cleaner paths
                     all_paths.push(entry.path().strip_prefix("./").unwrap_or(entry.path()).to_path_buf());
                 }
            }
            Err(e) => eprintln!("Warning: Failed to process entry: {}", e),
        }
    }

    // --- Prepare output file ---
    if output_path.exists() {
        fs::remove_file(&output_path)?;
        println!("Removed existing {}", OUTPUT_FILE);
    }
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "<codebase>")?;

    // --- Generate Tree Structure ---
    println!("Generating tree structure...");
    writeln!(writer, "<project_structure>")?;

    // Build the termtree::Tree structure
    let root_dir = PathBuf::from(".");
    let (project_tree, dir_count, file_count) = build_file_tree(&root_dir, &all_paths);

    // Write the formatted tree using termtree's Display impl
    write!(writer, "{}", project_tree)?; // Use write! not writeln! as termtree adds its own newlines

    // Write counts
    writeln!(writer, "\n{} directories, {} files", dir_count, file_count)?;

    writeln!(writer, "</project_structure>")?;
    writeln!(writer)?; // Add a blank line

    // --- Process Files ---
    println!("Processing files...");

    // Iterate through the *collected* paths for content inclusion
    for path in &all_paths { // Use the collected list
        // Skip directories explicitly (though collected paths might only be files depending on walker)
        if !path.is_file() {
            continue;
        }

        // Check if the file is likely text
        match is_valid_text_file(path) {
            Ok(true) => {
                let relative_path_str = path.to_string_lossy(); // Already stripped ./
                println!("Adding {}", relative_path_str);
                writeln!(writer, "<file src=\"{}\">", escape_xml(&relative_path_str))?;

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let escaped_content = escape_xml(&content);
                        // Add newline *after* content, before closing tag
                        writeln!(writer, "{}", escaped_content)?;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not read file {} as UTF-8 text: {}. Skipping content.",
                            relative_path_str, e
                        );
                        writeln!(writer, "<!-- Error reading file: {} -->", e)?;
                    }
                }
                 // Closing tag always starts on a new line.
                writeln!(writer, "</file>")?;
                writeln!(writer)?; // Add a blank line after each file block
            }

            Ok(false) => {
                println!("Skipping {} (likely binary or image file)", path.display());
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not determine file type for {}: {}. Skipping.",
                    path.display(),
                    e
                );
            }
        }
    }

    writeln!(writer, "</codebase>")?;
    writer.flush()?; // Ensure buffer is written

    let end_time = Local::now();
    println!(
        "File processing completed at {}",
        end_time.format("%Y-%m-%d %H:%M:%S")
    );

    println!(
        "Codebase conversion complete. Output saved to {}",
        OUTPUT_FILE
    );

    match fs::metadata(&output_path) {
        Ok(metadata) => {
            println!("File size: {} bytes", metadata.len());
        }
        Err(e) => eprintln!("Warning: Could not get file metadata: {}", e),
    }

    println!(
        "Script finished at {}",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}

/// Builds a termtree::Tree from a list of paths relative to a root.
/// Returns the tree and counts of directories and files included.
fn build_file_tree(_root_dir: &Path, paths: &[PathBuf]) -> (Tree<String>, usize, usize) {
    let mut children_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut dir_count = 0;
    let mut file_count = 0;

    // Populate the map: parent -> list of direct children
    for path in paths {
        if let Some(parent) = path.parent() {
            children_map
                .entry(parent.to_path_buf())
                .or_default()
                .push(path.clone());
        }
        // Count dirs/files based on metadata
        if let Ok(metadata) = fs::metadata(path) {
             if metadata.is_dir() {
                 dir_count += 1;
             } else if metadata.is_file() {
                 file_count += 1;
             }
        } else {
            // Fallback: guess based on path if metadata fails (less reliable)
            if path.is_dir() { // Path::is_dir might not require metadata access
                 dir_count += 1;
            } else {
                 file_count += 1; // Assume file if not dir-like
            }
        }
    }

    // Sort children within each directory for consistent order
    for children in children_map.values_mut() {
        children.sort();
    }

    // Recursive helper function to build the Tree structure
    fn build_recursive(
        current_path: &Path,
        children_map: &HashMap<PathBuf, Vec<PathBuf>>,
    ) -> Tree<String> {
        let filename = current_path
            .file_name()
            .map_or_else(|| ".".into(), |os| os.to_string_lossy().into_owned()); // Handle root "." case

        let mut tree = Tree::new(filename);

        if let Some(children) = children_map.get(current_path) {
            for child_path in children {
                tree.push(build_recursive(child_path, children_map));
            }
        }
        tree
    }

    // Start building from the logical root (which is empty path for collected paths)
    let root_node_path = PathBuf::from(""); // Represents the parent of top-level items
    let tree = build_recursive(&root_node_path, &children_map);

    // We want the display root to be "."
    let display_tree = Tree::new(".".to_string()).with_leaves(tree.leaves);

    (display_tree, dir_count, file_count)
}


/// Checks if a file is likely a text file based on content inspection.
fn is_valid_text_file(path: &Path) -> io::Result<bool> {
    const PEEK_BYTES: usize = 1024;
    let file = File::open(path)?; // Re-open is ok, done infrequently
    let mut buffer = Vec::with_capacity(PEEK_BYTES);
    // Read up to PEEK_BYTES
    let bytes_read = file.take(PEEK_BYTES as u64).read_to_end(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(true); // Empty file is considered text
    }

    let content_type = content_inspector::inspect(&buffer[..bytes_read]);

    // Consider TEXT or BINARY that looks like UTF-8 as valid.
    // Adjust this logic if you need stricter text-only (e.g., !content_type.is_binary())
    Ok(content_type.is_text() || content_type == ContentType::BINARY) // Example: keep original logic
}

/// Escapes XML special characters: &, <, >
fn escape_xml(s: &str) -> String {
    // Basic escaping for the markdown structure.
    // Consider a more robust XML escaping library if handling arbitrary XML/HTML content.
    s.replace('&', "&") // Use & for ampersand
     .replace('<', "<")  // Use < for less than
     .replace('>', ">")  // Use > for greater than
}

// Removed count_dirs_files function as counting is integrated into build_file_tree