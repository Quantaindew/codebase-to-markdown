use chrono::Local;
use content_inspector::ContentType;
use ignore::WalkBuilder;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

const OUTPUT_FILE: &str = "codebase.md";
// We don't need SCRIPT_NAME explicitly, as we build from source.
// We just need to ensure we don't include the OUTPUT_FILE itself.

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

    let output_path = PathBuf::from(OUTPUT_FILE); // Original PathBuf

    // Remove existing output file if it exists
    if output_path.exists() {
        fs::remove_file(&output_path)?; // Borrow original here is fine
        println!("Removed existing {}", OUTPUT_FILE);
    }

    let file = File::create(&output_path)?; // Borrow original here is fine
    let mut writer = BufWriter::new(file);

    writeln!(writer, "<codebase>")?;

    // --- Generate Tree Structure ---
    println!("Generating tree structure...");
    writeln!(writer, "<project_structure>")?;

    // Clone output_path for the first walker's closure
    let output_path_clone_tree = output_path.clone();
    let walker = WalkBuilder::new(".")
        .hidden(false)
        .parents(false)
        .ignore(true)
        .git_global(true)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(move |entry| {
            // Add move here
            // This closure now owns output_path_clone_tree
            let path = entry.path();
            path.file_name().map_or(true, |name| name != ".git")
                && path != output_path_clone_tree.as_path() // Compare against the clone
        })
        .build();

    // Store paths to format them like tree
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in walker {
        match entry {
            Ok(entry) => {
                if entry.depth() > 0 {
                    paths.push(
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

    // Write the formatted tree structure
    writeln!(writer, ".")?;
    for path in &paths {
        let depth = path.components().count();
        // Ensure correct depth calculation if root was skipped or included differently
        let display_depth = if path.starts_with("./") {
            depth - 1
        } else {
            depth
        };
        let indent = "    ".repeat(display_depth.saturating_sub(1));
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        // More robust prefix logic needed for exact tree look
        let prefix = if display_depth > 0 { "└── " } else { "" };
        writeln!(writer, "{}{}{}", indent, prefix, file_name)?;
    }
    let (dir_count, file_count) = count_dirs_files(&paths);
    writeln!(writer, "\n{} directories, {} files", dir_count, file_count)?;

    writeln!(writer, "</project_structure>")?;
    writeln!(writer)?;

    // --- Process Files ---
    println!("Processing files...");

    // Clone output_path for the second walker's closure
    let output_path_clone_files = output_path.clone();
    let file_walker = WalkBuilder::new(".")
        .hidden(false)
        .parents(false)
        .ignore(true)
        .git_global(true)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .filter_entry(move |entry| {
            // Add move here
            // This closure now owns output_path_clone_files
            let path = entry.path();
            path.file_name().map_or(true, |name| name != ".git")
                && path != output_path_clone_files.as_path() // Compare against the clone
        })
        .build();

    for entry in file_walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Skipping entry due to error: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Skip the output file itself (redundant check, but safe)
        // Compare against the original output_path here just for variety,
        // or could use another clone if preferred. Borrowing original is fine.
        if path == output_path.as_path() {
            continue;
        }

        // Check if the file is likely text
        match is_valid_text_file(path) {
            Ok(true) => {
                let relative_path_str = path.strip_prefix("./").unwrap_or(path).to_string_lossy();
                println!("Adding {}", relative_path_str);
                writeln!(writer, "<file src=\"{}\">", escape_xml(&relative_path_str))?;

                match fs::read_to_string(path) {
                    Ok(content) => {
                        // Write content, escaping XML special characters
                        writer.write_all(escape_xml(&content).as_bytes())?;
                        // ---> START FIX <---
                        // Ensure there's a newline *before* the closing tag,
                        // regardless of whether the original file content ended with one.
                        writer.write_all(b"\n")?;
                        // ---> END FIX <---
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not read file {} as UTF-8 text: {}. Skipping content.",
                            relative_path_str, e
                        );
                        // Error comment already includes newline via writeln!
                        writeln!(writer, "<!-- Error reading file: {} -->", e)?;
                    }
                }
                // Now write the closing tag, it will always start on a new line.
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

    // Get file size - Use the original output_path, which is still owned by run()
    match fs::metadata(&output_path) {
        // Borrow original - THIS IS NOW VALID
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

/// Checks if a file is likely a text file based on content inspection.
fn is_valid_text_file(path: &Path) -> io::Result<bool> {
    const PEEK_BYTES: usize = 1024;
    let file = File::open(path)?;
    let mut buffer = Vec::with_capacity(PEEK_BYTES);
    // Safety: We ensure buffer is filled by read before inspecting
    let bytes_read = file.take(PEEK_BYTES as u64).read_to_end(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(true); // Empty file is considered text
    }

    let content_type = content_inspector::inspect(&buffer[..bytes_read]);

    // Treat binary that smells like UTF-8 as text too, or just check is_text()
    Ok(content_type.is_text() || content_type == ContentType::BINARY)
}

/// Escapes XML special characters: &, <, >
fn escape_xml(s: &str) -> String {
    s.replace('&', "&").replace('<', "<").replace('>', ">")
}

/// Counts directories and files from a list of paths.
fn count_dirs_files(paths: &[PathBuf]) -> (usize, usize) {
    let mut dir_count = 0;
    let mut file_count = 0;
    for path in paths {
        // Re-check metadata as the walker might list things that were deleted
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_dir() {
                dir_count += 1;
            } else if metadata.is_file() {
                file_count += 1;
            }
        }
    }
    (dir_count, file_count)
}
