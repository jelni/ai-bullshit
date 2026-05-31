with open('src/game/statistics.rs', 'r') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if line.startswith('use super::{'):
        if 'Artifact' not in line:
            lines[i] = line.replace('use super::{', 'use super::{Artifact, ')
        break

with open('src/game/statistics.rs', 'w') as f:
    f.writelines(lines)
