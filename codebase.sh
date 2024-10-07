#!/bin/bash

OUTPUT_FILE="codebase.md"
rm -f "$OUTPUT_FILE"

echo "# Codebase Contents" > "$OUTPUT_FILE"

echo "Starting script at $(date)"

# Function to check if a file should be ignored based on .gitignore patterns
should_ignore() {
    local file="$1"
    if [ "$file" == ".gitignore" ]; then
        return 0  # Ignore .gitignore file itself
    fi
    if [ -f ".gitignore" ]; then
        while IFS= read -r pattern; do
            [[ $pattern =~ ^# ]] && continue  # Skip comments
            [[ -z $pattern ]] && continue     # Skip empty lines
            if [[ $file == $pattern || $file == */$pattern || $file == $pattern/* || $file == */$pattern/* ]]; then
                return 0  # Should be ignored
            fi
        done < .gitignore
    fi
    return 1  # Should not be ignored
}

# Generate tree structure
echo "Generating tree structure..."
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
tree -I "$OUTPUT_FILE" -P "*" | while read -r line; do
    file=$(echo "$line" | sed 's/^[│├└─]* //')
    if ! should_ignore "$file"; then
        echo "$line"
    fi
done >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to check if a file is NOT a binary/image file
is_valid_text_file() {
    ! file -i "$1" | grep -qE 'binary|charset=binary|image/'
}

echo "Processing files..."
find . -type f | while read -r file; do
    file="${file#./}"  # Remove leading './' if present
    if [[ "$file" != "$OUTPUT_FILE" && ! $(should_ignore "$file") ]]; then
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
    else
        echo "Skipping $file (ignored by .gitignore, is .gitignore, or is output file)"
    fi
done

echo "File processing completed at $(date)"

echo "Codebase conversion complete. Output saved to $OUTPUT_FILE"
la $OUTPUT_FILE
echo "Script finished at $(date)"
