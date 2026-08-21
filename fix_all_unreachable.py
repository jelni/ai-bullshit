import re

with open("errors.txt", "r") as f:
    errors = f.read()

with open("src/main.rs", "r") as f:
    lines = f.readlines()

for match in re.finditer(r'src/main\.rs:(\d+):', errors):
    line_num = int(match.group(1)) - 1

    # We found an unreachable pattern at `line_num`.
    # Let's inspect it to see what number it currently is
    original_line = lines[line_num]
    m = re.search(r'(\s*)(\d+)(\s*=>\s*\{)', original_line)
    if m:
        indent = m.group(1)
        old_val = int(m.group(2))
        rest = m.group(3)

        # It should be incremented to avoid the overlap with the previous block
        # For duplicates (like 1, 1), the second one becomes 2
        new_val = old_val + 1

        lines[line_num] = f"{indent}{new_val}{rest}\n"

with open("src/main.rs", "w") as f:
    f.writelines(lines)
