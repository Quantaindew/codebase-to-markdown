#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

echo "# Codebase Contents" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo "Starting script at $(date)"

# Function to check if a file is a text file
is_text_file() {
    file -b --mime-type "$1" | grep -q '^text/'
}

# Function to check if a file or directory should be ignored
should_ignore() {
    local path="$1"
    
    # Always ignore .git directory and codebase.md
    [[ "$path" == ".git"* ]] && return 0
    [[ "$path" == "$OUTPUT_FILE" ]] && return 0
    
    # Check against .gitignore patterns
    if [[ -f ".gitignore" ]]; then
        while IFS= read -r pattern || [[ -n "$pattern" ]]; do
            [[ $pattern =~ ^# ]] && continue  # Skip comments
            [[ -z $pattern ]] && continue     # Skip empty lines
            if [[ "$path" == $pattern ]] || [[ "$path" == *"/$pattern"* ]]; then
                return 0
            fi
        done < ".gitignore"
    fi
    return 1
}

# Generate tree structure
echo "Generating tree structure..."
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
tree -I ".git|$OUTPUT_FILE" --gitignore >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo "Processing files..."

process_directory() {
    local dir="$1"
    for item in "$dir"/*; do
        if [[ -d "$item" ]]; then
            if ! should_ignore "$item"; then
                process_directory "$item"
            fi
        elif [[ -f "$item" ]]; then
            local relative_path="${item#./}"
            if ! should_ignore "$relative_path" && is_text_file "$item"; then
                echo "Adding $relative_path"
                echo "## File: $relative_path" >> "$OUTPUT_FILE"
                echo '```' >> "$OUTPUT_FILE"
                cat "$item" >> "$OUTPUT_FILE"
                echo '```' >> "$OUTPUT_FILE"
                echo "" >> "$OUTPUT_FILE"
            fi
        fi
    done
}

process_directory "."

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
echo "Script finished at $(date)"
