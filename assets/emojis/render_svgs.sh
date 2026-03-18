svgs_path="svgs/*.svg"
svg_size=1024

for file in $svgs_path; do
    if [ -f "$file" ]; then
        base_filename=$(basename ${file%.*})
        inkscape -w $svg_size -h $svg_size $file -o $base_filename.png
    fi
done
