// src/main.rs
mod tree; // Declare the tree module

use crate::tree::Tree; // Import the Tree struct
use chrono::Local;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
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
    let absolute_output_path = fs::canonicalize(&output_path).ok();

    let output_path_filter = output_path.clone();
    let filter = move |entry: &ignore::DirEntry| -> bool {
        let path = entry.path();
        if path.file_name().map_or(false, |name| name == ".git") {
            return false;
        }
        if let Some(abs_output) = &absolute_output_path {
            if fs::canonicalize(path).ok() == Some(abs_output.clone()) {
                return false;
            }
        } else if path == output_path_filter.as_path() {
            return false;
        }
        true
    };

    let mut all_paths: Vec<PathBuf> = Vec::new();
    let walker = WalkBuilder::new(".")
        .hidden(false)
        .parents(false)
        .ignore(true)
        .git_global(true)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(filter)
        .build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                if entry.depth() > 0 {
                    all_paths.push(
                        entry
                            .path()
                            .strip_prefix("./")
                            .unwrap_or(entry.path())
                            .to_path_buf(),
                    );
                }
            }
            Err(e) => eprintln!("Warning: Failed to process entry: {}", e),
        }
    }

    if output_path.exists() {
        fs::remove_file(&output_path)?;
        println!("Removed existing {}", OUTPUT_FILE);
    }
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "<codebase>")?;

    println!("Generating tree structure...");
    writeln!(writer, "<project_structure>")?;
    let (project_tree, dir_count, file_count) = build_file_tree(&all_paths);
    write!(writer, "{}", project_tree)?;
    writeln!(writer, "\n{} directories, {} files", dir_count, file_count)?;
    writeln!(writer, "</project_structure>")?;
    writeln!(writer)?;

    println!("Processing files...");
    for path in &all_paths {
        if !path.is_file() {
            continue;
        }
        match fs::read_to_string(path) {
            Ok(content) => {
                let relative_path_str = path.to_string_lossy();
                println!("Adding {}", relative_path_str);
                writeln!(writer, "<file src=\"{}\">", escape_xml(&relative_path_str))?;
                let escaped_content = escape_xml(&content);
                writeln!(writer, "{}", escaped_content)?;
                writeln!(writer, "</file>")?;
                writeln!(writer)?;
            }
            Err(e) => {
                eprintln!("Skipping {}: {}", path.display(), e);
            }
        }
    }

    writeln!(writer, "</codebase>")?;
    writer.flush()?;

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
            let size = metadata.len();
            println!("File size: {}", format_file_size(size));
        }
        Err(e) => eprintln!("Warning: Could not get file metadata: {}", e),
    }
    println!(
        "Script finished at {}",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}

fn build_file_tree(paths: &[PathBuf]) -> (Tree<String>, usize, usize) {
    let mut children_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut dir_count = 0;
    let mut file_count = 0;

    for path in paths {
        if let Some(parent) = path.parent() {
            children_map
                .entry(parent.to_path_buf())
                .or_default()
                .push(path.clone());
        }
        if path.is_dir() {
            dir_count += 1;
        } else if path.is_file() {
            file_count += 1;
        }
    }

    for children in children_map.values_mut() {
        children.sort();
    }

    fn build_recursive(
        current_path: &Path,
        children_map: &HashMap<PathBuf, Vec<PathBuf>>,
    ) -> Tree<String> {
        let filename = current_path
            .file_name()
            .map_or_else(|| ".".into(), |os| os.to_string_lossy().into_owned());
        let mut tree = Tree::new(filename);
        if let Some(children) = children_map.get(current_path) {
            for child_path in children {
                tree.push(build_recursive(child_path, children_map));
            }
        }
        tree
    }

    let root_node_path = PathBuf::from("");
    let tree = build_recursive(&root_node_path, &children_map);
    let display_tree = Tree::new(".".to_string()).with_leaves(tree.leaves);
    (display_tree, dir_count, file_count)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&").replace('<', "<").replace('>', ">")
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} bytes", size)
    } else if size < MB {
        format!("{:.3} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.3} MB", size as f64 / MB as f64)
    } else {
        format!("{:.3} GB", size as f64 / GB as f64)
    }
}
