import re
with open("src/main.rs", "r") as f:
    lines = f.readlines()

fixes = {
    371: "            2 => {\n",
    379: "            4 => {\n",
    411: "            12 => {\n",
    419: "            14 => {\n",
    451: "            22 => {\n",
    459: "            24 => {\n",
    491: "            32 => {\n",
    499: "            34 => {\n",
    531: "            42 => {\n",
    540: "            44 => {\n",
    598: "            62 => {\n",
    606: "            64 => {\n",
    639: "            72 => {\n",
    647: "            74 => {\n",

    943: "            4 => {\n",
    1513: "            4 => {\n",
    1703: "            4 => {\n",
    1825: "            4 => {\n",

    2402: "                2 => {\n",
    2469: "                2 => {\n",
    2493: "                4 => {\n",

    2553: "            2 => {\n",
    2606: "            2 => {\n",
    2673: "            2 => {\n",
}

for line_num, replacement in fixes.items():
    idx = line_num - 1
    # Check if the line to replace looks like it should be replaced, and it hasn't been mangled
    if "=> {" in lines[idx]:
        lines[idx] = replacement
    else:
        # Search backwards or forwards a few lines in case line numbers shifted slightly
        for offset in range(-5, 6):
            if idx + offset < len(lines) and "=> {" in lines[idx + offset] and (str(line_num) in lines[idx + offset] or re.search(r'\s*\d+\s*=>\s*\{', lines[idx + offset])):
                pass # This is complicated.

# Let's just fix the unreachable patterns by reading the clippy errors directly from cargo check.
