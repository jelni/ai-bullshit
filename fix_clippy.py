import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Add #[expect(clippy::too_many_lines)] above handle_menu_input
content = re.sub(r'(fn handle_menu_input\(code: KeyCode, game: &mut Game\) -> bool \{)',
                 r'#[expect(\n    clippy::too_many_lines,\n    reason = "Menu input handling matches many variants"\n)]\n\1', content)

with open('src/main.rs', 'w') as f:
    f.write(content)

with open('src/game.rs', 'r') as f:
    content = f.read()

content = content.replace("let nx = x as i32 + dx;", "let nx = i32::from(x) + dx;")
content = content.replace("let ny = y as i32 + dy;", "let ny = i32::from(y) + dy;")
content = content.replace("if grid[ny as usize][nx as usize] {", "if grid[usize::try_from(ny).unwrap()][usize::try_from(nx).unwrap()] {")
content = content.replace("let cx = start_x as i32 + dx;", "let cx = i32::from(start_x) + dx;")
content = content.replace("let cy = start_y as i32 + dy;", "let cy = i32::from(start_y) + dy;")
content = content.replace("if cx > 0 && cx < (width - 1) as i32 && cy > 0 && cy < (height - 1) as i32 {", "if cx > 0 && cx < i32::from(width - 1) && cy > 0 && cy < i32::from(height - 1) {")
content = content.replace("grid[cy as usize][cx as usize] = false;", "grid[usize::try_from(cy).unwrap()][usize::try_from(cx).unwrap()] = false;")

with open('src/game.rs', 'w') as f:
    f.write(content)
