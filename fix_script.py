import re

with open("src/main.rs", "r") as f:
    lines = f.readlines()

new_lines = []
skip = False
for i, line in enumerate(lines):
    # This is getting tedious. Let's just fix the specific overlaps.
