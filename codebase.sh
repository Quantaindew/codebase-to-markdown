#!/bin/bash

SCRIPT_NAME=$(basename "$0")
OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

echo "# Codebase Contents" > "$OUTPUT_FILE"

echo "Starting script at $(date)"

# Generate tree structure
echo "Generating tree structure..."
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
tree -I ".git|$OUTPUT_FILE|$SCRIPT_NAME" --gitignore >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to check if a file is NOT a binary/image file
is_valid_text_file() {
    ! file -i "$1" | grep -qE 'binary|charset=binary|image/'
}

# Function to check if a path matches any pattern in .gitignore
is_ignored() {
    local path="$1"
    local ignore_file=".gitignore"
    
    # First, check if the path is the script itself or the output file
    if [ "$path" = "$SCRIPT_NAME" ] || [ "$path" = "$OUTPUT_FILE" ]; then
        return 0  # Path should be ignored
    fi
    
    if [ -f "$ignore_file" ]; then
        while IFS= read -r pattern || [ -n "$pattern" ]; do
            case "$pattern" in
                "#"*) continue ;;
                "") continue ;;
                *)
                    # Convert gitignore pattern to grep pattern
                    grep_pattern=$(echo "$pattern" | sed -e 's:^/::' -e 's:/$::' -e 's:^:^:' -e 's:$:(/|$):' -e 's/\./\\./g' -e 's/\*/.*/g')
                    if echo "$path" | grep -qE "$grep_pattern"; then
                        return 0  # Path is ignored
                    fi
                    ;;
            esac
        done < "$ignore_file"
    fi
    return 1  # Path is not ignored
}

# Recursive function to process directories and files
process_directory() {
    local dir="$1"
    local rel_path="$2"

    for item in "$dir"/*; do
        local item_rel_path="${rel_path:+$rel_path/}${item##*/}"
        
        if [ -d "$item" ]; then
            if ! is_ignored "$item_rel_path"; then
                process_directory "$item" "$item_rel_path"
            else
                echo "Skipping directory $item_rel_path (ignored)"
            fi
        elif [ -f "$item" ]; then
            if ! is_ignored "$item_rel_path"; then
                if is_valid_text_file "$item"; then
                    echo "Adding $item_rel_path"
                    echo "## File: $item_rel_path" >> "$OUTPUT_FILE"
                    echo '```' >> "$OUTPUT_FILE"
                    cat "$item" >> "$OUTPUT_FILE"
                    echo -e '\n```' >> "$OUTPUT_FILE"
                    echo "" >> "$OUTPUT_FILE"
                else
                    echo "Skipping $item_rel_path (likely binary or image file)"
                fi
            else
                echo "Skipping $item_rel_path (ignored)"
            fi
        fi
    done
}

echo "Processing files..."
process_directory "." ""

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
echo "File size:"
ls -lh $OUTPUT_FILE 

echo "Script finished at $(date)"
