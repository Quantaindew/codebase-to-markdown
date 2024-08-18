#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

# Start the codebase.md file
echo "# Codebase Contents" > "$OUTPUT_FILE"

# Function to escape special regex characters
escape_regex() {
    echo "$1" | tr -d '\n' | sed 's/[]\/$*.^|[]/\\&/g'
}

# Function to generate gitignore patterns
generate_gitignore_patterns() {
    echo -n "^./\\.git($|/)"  # Always ignore .git folder
    echo -n "|^./$OUTPUT_FILE$"  # Ignore the output file itself
    if [ -f .gitignore ]; then
        while IFS= read -r line || [[ -n "$line" ]]; do
            # Ignore comments and empty lines
            if [[ ! "$line" =~ ^\s*# && -n "$line" ]]; then
                # Convert gitignore globs to regex
                pattern=$(escape_regex "$line")
                pattern=${pattern//\*/.*}
                echo -n "|^./$pattern($|/)"
            fi
        done < .gitignore
    fi
}

# Add the tree structure to the file
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
tree -I "$(generate_gitignore_patterns)" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to add file contents
add_file_contents() {
    local file="$1"
    local target_file="$2"
    echo "## File: $file" >> "$target_file"
    echo '```' >> "$target_file"
    
    # Check if the file is binary or an image
    if is_binary_or_image "$file"; then
        echo "[Binary or image file, contents not displayed]" >> "$target_file"
    else
        cat "$file" >> "$target_file"
    fi
    
    echo '```' >> "$target_file"
    echo "" >> "$target_file"
}

# Function to check if a file is binary or an image
is_binary_or_image() {
    local file="$1"
    local mime_type=$(file -b --mime-type "$file")
    
    # Check if the file is binary or an image
    if [[ $mime_type == application/* && $mime_type != application/x-empty && $mime_type != application/json && $mime_type != application/xml ]] || 
       [[ $mime_type == image/* ]] || 
       [[ "${file##*.}" =~ ^(png|jpg|jpeg|gif|bmp|svg)$ ]]; then
        return 0  # It's a binary or image file
    else
        return 1  # It's not a binary or image file
    fi
}

# Function to check if a file should be ignored
should_ignore() {
    local path="$1"
    local gitignore_patterns

    # Generate gitignore patterns (including .git and the output file)
    gitignore_patterns=$(generate_gitignore_patterns)

    # Check if the path matches any gitignore pattern
    [[ $path =~ $gitignore_patterns ]]
}

# Loop through all files in the directory and subdirectories
find . -type f | while read -r file; do
    if ! should_ignore "$file"; then
        add_file_contents "$file" "$OUTPUT_FILE"
    fi
done
