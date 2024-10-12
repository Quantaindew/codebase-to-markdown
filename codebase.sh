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

echo "Processing files..."

# Use tree to list files, respecting .gitignore, and remove leading './'
tree -if --noreport --gitignore | sed 's|^./||' | while read -r file; do
    # Skip directories and excluded files
    if [ -f "$file" ] && [ "$file" != "$OUTPUT_FILE" ] && [ "$file" != "$SCRIPT_NAME" ]; then
        if is_valid_text_file "$file"; then
            echo "Adding $file"
            echo "## File: $file" >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
            cat "$file" >> "$OUTPUT_FILE"
            echo -e '\n```' >> "$OUTPUT_FILE"
            echo "" >> "$OUTPUT_FILE"
        else
            echo "Skipping $file (likely binary or image file)"
        fi
    fi
done

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
echo "File size:"
ls -lh "$OUTPUT_FILE"

echo "Script finished at $(date)"
