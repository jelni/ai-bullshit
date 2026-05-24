import random

def generate_maze(width, height):
    # Dimensions must be odd for perfect maze algorithms
    # width 39 (0 to 38), height 21 (0 to 20)
    # We will use a grid of 19x10 for cells, plus walls.
    # Actually w=39, h=21. (39-1)/2 = 19 cells wide. (21-1)/2 = 10 cells high.

    maze = [["#" for _ in range(width)] for _ in range(height)]

    # Directions: (dx, dy)
    dirs = [(0, -2), (0, 2), (-2, 0), (2, 0)]

    def in_bounds(x, y):
        return 0 < x < width-1 and 0 < y < height-1

    def carve(x, y):
        maze[y][x] = " "
        random.shuffle(dirs)
        for dx, dy in dirs:
            nx, ny = x + dx, y + dy
            if in_bounds(nx, ny) and maze[ny][nx] == "#":
                maze[y + dy//2][x + dx//2] = " "
                carve(nx, ny)

    # start carving from (1, 1)
    carve(1, 1)

    # Randomly remove some walls to create loops
    for _ in range(20):
        rx = random.randint(1, width-2)
        ry = random.randint(1, height-2)
        if maze[ry][rx] == "#":
            maze[ry][rx] = " "

    # Clear out a 3x3 starting area in the center so the snake can spawn safely
    cx, cy = width//2, height//2
    for dy in range(-2, 3):
        for dx in range(-2, 3):
            maze[cy+dy][cx+dx] = " "

    return maze

def format_maze(maze):
    lines = []
    lines.append("    CampaignMap {")
    lines.append(f"        width: {len(maze[0])},")
    lines.append(f"        height: {len(maze)},")
    lines.append("        layout: &[")
    for row in maze:
        lines.append(f'            ' + '"' + "".join(row) + '",')
    lines.append("        ],")
    lines.append("    },")
    return "\n".join(lines)

with open('src/game/campaign_maps.rs', 'r') as f:
    lines = f.readlines()

end_idx = 0
for i, line in enumerate(reversed(lines)):
    if line.strip() == "];":
        end_idx = len(lines) - 1 - i
        break

extra_maps = "\n".join(format_maze(generate_maze(39, 21)) for _ in range(185)) + "\n"
lines.insert(end_idx, extra_maps)

with open('src/game/campaign_maps.rs', 'w') as f:
    f.writelines(lines)
