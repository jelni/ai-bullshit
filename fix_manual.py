import sys

def fix_file():
    with open("src/main.rs", "r") as f:
        lines = f.readlines()

    # We will look for sequences where the number is duplicated and increment the latter ones
    # Specifically, look for lines with "=> {" right after each other with the same number
    # Or more generally just run a regex substitution to fix the lines mentioned in clippy output.

    # 367 and 371 both have 1 => {  (371 should be 2)
    # 375 and 379 both have 3 => {  (379 should be 4)
    # 407 and 411 both have 11 => { (411 should be 12)
    # 415 and 419 both have 13 => { (419 should be 14)
    # 447 and 451 both have 21 => { (451 should be 22)
    # 455 and 459 both have 23 => { (459 should be 24)
    # 487 and 491 both have 31 => { (491 should be 32)
    # 495 and 499 both have 33 => { (499 should be 34)
    # 527 and 531 both have 41 => { (531 should be 42)
    # 535 and 540 both have 43 => { (540 should be 44)
    # 594 and 598 both have 61 => { (598 should be 62)
    # 602 and 606 both have 63 => { (606 should be 64)
    # 635 and 639 both have 71 => { (639 should be 72)
    # 643 and 647 both have 73 => { (647 should be 74)

    # 928 and 943 both have 3 => { (943 should be 4)
    # 1504 and 1513 both have 3 => { (1513 should be 4)
    # 1690 and 1703 both have 3 => { (1703 should be 4)
    # 1816 and 1825 both have 3 => { (1825 should be 4)

    # 2378 and 2402 both have 1 => { (2402 should be 2)
    # 2458 and 2469 both have 1 => { (2469 should be 2)
    # 2482 and 2493 both have 3 => { (2493 should be 4)

    # 2544 and 2553 both have 1 => { (2553 should be 2)
    # 2595 and 2606 both have 1 => { (2606 should be 2)
    # 2662 and 2673 both have 1 => { (2673 should be 2)

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
        # 1-based index to 0-based index
        idx = line_num - 1
        lines[idx] = replacement

    with open("src/main.rs", "w") as f:
        f.writelines(lines)

fix_file()
