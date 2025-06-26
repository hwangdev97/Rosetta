#!/bin/bash

# Create assets directory if it doesn't exist
mkdir -p assets

# Use figlet to create ASCII art and convert it to PNG using ImageMagick
echo "ROSETTA" | figlet -f standard | convert -background black -fill white -font "Courier" -pointsize 24 label:@- assets/Cover.png

# Add some padding and effects
convert assets/Cover.png -bordercolor black -border 20x20 \
    \( +clone -background black -shadow 80x3+5+5 \) +swap \
    -background black -layers merge +repage \
    assets/Cover.png

echo "Cover image generated at assets/Cover.png" 