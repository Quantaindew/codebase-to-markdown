#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

echo "# Codebase Contents" > "$OUTPUT_FILE"

echo "Starting script at $(date)"

# Generate tree structure
echo "Generating tree structure..."
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
tree -I ".git|$OUTPUT_FILE" --gitignore >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to check if a file is NOT a binary/image file
is_valid_text_file() {
    ! file -i "$1" | grep -qE 'binary|charset=binary|image/'
}

echo "Processing files..."
git ls-files | while read -r file; do
    if [[ -f "$file" ]]; then
        if is_valid_text_file "$file"; then
            echo "Adding $file"
            echo "## File: $file" >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
            cat "$file" >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
            echo "" >> "$OUTPUT_FILE"
        else
            echo "Skipping $file (likely binary or image file)"
        fi
    fi
done

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
echo "Script finished at $(date)"
