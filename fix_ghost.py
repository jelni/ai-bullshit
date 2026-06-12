import re

with open("src/game/game_struct.rs", "r") as f:
    content = f.read()

# ghost_snake issue. Wait, did my fix_test.py or patch_game_struct.py break ghost snake?
# I'll check my diffs to see if I touched anything else.
