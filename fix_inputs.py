import re

with open("src/main.rs", "r") as f:
    lines = f.readlines()

in_match_menu = False
match_counter = 0

for i, line in enumerate(lines):
    if "match game.menu_selection {" in line:
        in_match_menu = True
        match_counter = 0
        continue

    if in_match_menu:
        if "=> {" in line and line.strip().split(" ")[0].isdigit():
            # Extract current number
            old_num = int(line.strip().split(" ")[0])

            # Replace with new number
            lines[i] = line.replace(f"{old_num} => {{", f"{match_counter} => {{", 1)
            match_counter += 1

        elif "}" in line and line.strip() == "}":
            # Just a closing brace, not end of match yet maybe? Let's be careful.
            pass

        elif "_ => {}" in line or "_ => ()," in line:
            in_match_menu = False

# Wait, this might be too naive because there are multiple matches in the file.
# Let's fix them manually.
