#!/bin/bash

rm -f codebase.md

# Start the codebase.md file
echo "# Codebase Contents" > ./codebase.md

# Function to generate gitignore patterns
generate_gitignore_patterns() {
    echo -n ".git|"  # Always ignore .git folder
    if [ -f .gitignore ]; then
        grep -v '^#' .gitignore | sed 's/^/\//; s/$/|/' | tr -d '\n' | sed 's/|$//'
    fi
}

# Add the tree structure to the file
echo "## Project Structure" >> ./codebase.md
echo '```' >> ./codebase.md
tree -I "$(generate_gitignore_patterns)" >> ./codebase.md
echo '```' >> ./codebase.md
echo "" >> ./codebase.md

# Function to add file contents
add_file_contents() {
    local target_file="$2"
    echo "## File: $1" >> "$target_file"
    echo '```' >> "$target_file"
    cat "$1" >> "$target_file"
    echo '```' >> "$target_file"
    echo "" >> "$target_file"
}

# Function to check if a file should be ignored
should_ignore() {
    local path="$1"
    local gitignore_patterns

    # Generate gitignore patterns (including .git)
    gitignore_patterns=$(generate_gitignore_patterns)

    # Check if the path matches any gitignore pattern or starts with ./.git
    echo "$path" | grep -qE "^./\.git|$gitignore_patterns" && return 0

    # Check if the file is an image
    if [ -f "$path" ]; then
        file --mime-type "$path" | grep -q "image/" && return 0
    fi

    return 1
}

# Loop through all files in the directory and subdirectories
find . -type f | while read -r file; do
    if ! should_ignore "$file"; then
        add_file_contents "$file" "./codebase.md"
    fi
done
