#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

echo "# Codebase Contents" > "$OUTPUT_FILE"

echo "Starting script at $(date)"

# Function to check if a file is a text file
is_text_file() {
    file -b --mime-type "$1" | grep -q '^text/'
}

echo "Processing files..."
git ls-files | while read -r file; do
    if [[ -f "$file" ]] && is_text_file "$file"; then
        echo "Adding $file"
        echo "## File: $file" >> "$OUTPUT_FILE"
        echo '```' >> "$OUTPUT_FILE"
        cat "$file" >> "$OUTPUT_FILE"
        echo '```' >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
echo "Script finished at $(date)"
