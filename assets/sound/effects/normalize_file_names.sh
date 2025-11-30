#!/bin/bash

# Normalize file and folder names:
# - Replace spaces with underscores
# - Convert to lowercase
# - Remove "SFX" string
# - Remove non-alphanumeric characters (except underscores, dots, and path separators)

cd "$(dirname "$0")"

# Process files and directories from deepest to shallowest
# This ensures we rename contents before renaming parent directories
find . -depth -name "*" | while read -r path; do
    # Skip the current directory and this script
    [ "$path" = "." ] && continue
    [ "$(basename "$path")" = "normalize_file_names.sh" ] && continue
    
    dir=$(dirname "$path")
    name=$(basename "$path")
    
    # Get the extension if it's a file
    if [ -f "$path" ]; then
        ext="${name##*.}"
        base="${name%.*}"
    else
        ext=""
        base="$name"
    fi
    
    # Transform the base name:
    # 1. Remove "SFX" (case insensitive)
    # 2. Replace spaces with underscores
    # 3. Convert to lowercase
    # 4. Remove non-alphanumeric characters (keep underscores)
    # 5. Collapse multiple underscores into one
    # 6. Remove leading/trailing underscores
    new_base=$(echo "$base" | \
        sed -E 's/[Ss][Ff][Xx]//g' | \
        tr ' ' '_' | \
        tr '[:upper:]' '[:lower:]' | \
        sed -E 's/[^a-z0-9_]//g' | \
        sed -E 's/_+/_/g' | \
        sed -E 's/^_+|_+$//g')
    
    # Reconstruct the new name
    if [ -n "$ext" ] && [ -f "$path" ]; then
        new_name="${new_base}.${ext}"
    else
        new_name="$new_base"
    fi
    
    # Rename if different
    if [ "$name" != "$new_name" ] && [ -n "$new_name" ]; then
        new_path="${dir}/${new_name}"
        echo "Renaming: $path -> $new_path"
        mv "$path" "$new_path"
    fi
done

echo "Done!"
