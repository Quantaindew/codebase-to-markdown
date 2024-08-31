#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

# Start the codebase.md file
echo "# Codebase Contents" > "$OUTPUT_FILE"

# Function to escape special regex characters
escape_regex() {
    echo "$1" | tr -d '\n' | sed 's/[]\/$*.^|[]/\\&/g'
}

# Function to generate ignore patterns for tree command
generate_tree_ignore_patterns() {
    local patterns="-I '.git' -I '$OUTPUT_FILE'"
    if [ -f .gitignore ]; then
        while IFS= read -r line || [[ -n "$line" ]]; do
            # Ignore comments and empty lines
            if [[ ! "$line" =~ ^\s*# && -n "$line" ]]; then
                patterns+=" -I '$line'"
            fi
        done < .gitignore
    fi
    echo "$patterns"
}

# Add the tree structure to the file
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
eval tree $(generate_tree_ignore_patterns) -L 3 -F >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to add file contents
add_file_contents() {
    local file="$1"
    local target_file="$2"
    echo "## File: $file" >> "$target_file"
    echo '```' >> "$target_file"
    
    # Check if the file is binary, image, or text
    local file_type=$(get_file_type "$file")
    case "$file_type" in
        "image")
            echo "[Image file, contents not displayed]" >> "$target_file"
            ;;
        "binary")
            echo "[Binary file, contents not displayed]" >> "$target_file"
            ;;
        "text")
            cat "$file" >> "$target_file"
            ;;
    esac
    
    echo '```' >> "$target_file"
    echo "" >> "$target_file"
}

# Function to determine file type (image, binary, or text)
get_file_type() {
    local file="$1"
    
    # Use 'file' command to determine the file type
    local mime_type=$(file -b --mime-type "$file")
    
    # Check if it's an image file
    if [[ $mime_type == image/* ]]; then
        echo "image"
        return
    fi
    
    # Check if it's explicitly a text type
    if [[ $mime_type == text/* || 
          $mime_type == application/json || 
          $mime_type == application/xml || 
          $mime_type == application/javascript ]]; then
        echo "text"
        return
    fi
    
    # For other types, use a heuristic approach
    if LC_ALL=C grep -qP "[\x00]" "$file"; then
        echo "binary"  # File contains null bytes, likely binary
    elif LC_ALL=C grep -qP "[\x00-\x08\x0E-\x1F\x7F]" <(head -c 1024 "$file"); then
        echo "binary"  # File contains control characters, likely binary
    else
        echo "text"  # Likely a text file
    fi
}

# Function to check if a file should be ignored
should_ignore() {
    local path="$1"
    if [[ "$path" == ./.git* || "$path" == "./$OUTPUT_FILE" ]]; then
        return 0
    fi
    if [ -f .gitignore ]; then
        while IFS= read -r pattern; do
            if [[ ! "$pattern" =~ ^\s*# && -n "$pattern" ]]; then
                if [[ "$path" == *"$pattern"* ]]; then
                    return 0
                fi
            fi
        done < .gitignore
    fi
    return 1
}

# Loop through all files in the directory and subdirectories
find . -type f | while read -r file; do
    if ! should_ignore "$file"; then
        add_file_contents "$file" "$OUTPUT_FILE"
    fi
done
