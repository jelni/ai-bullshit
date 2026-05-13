use crate::game::{Game, GameState};
use web_sys::{Document, HtmlElement};

pub fn draw(game: &Game, container: &HtmlElement, _document: &Document) {
    let mut output = String::new();

    // Simple text-based rendering to match the exact output of the terminal

    // We can simulate the text-based grid
    let mut grid = vec![vec![' '; game.width as usize]; game.height as usize];

    // Draw borders (simplified)
    for y in 0..game.height as usize {
        for x in 0..game.width as usize {
            if x == 0 || x == game.width as usize - 1 || y == 0 || y == game.height as usize - 1 {
                grid[y][x] = '#';
            }
        }
    }

    // Draw food
    if game.food.x < game.width && game.food.y < game.height {
        grid[game.food.y as usize][game.food.x as usize] = '@';
    }

    // Draw obstacles
    for obs in &game.obstacles {
        if obs.x < game.width && obs.y < game.height {
            grid[obs.y as usize][obs.x as usize] = 'X';
        }
    }

    // Draw snake
    for part in &game.snake.body {
        if part.x < game.width && part.y < game.height {
            grid[part.y as usize][part.x as usize] = 'O'; // Replace with game.skin later
        }
    }

    // Render
    for row in grid {
        output.push_str(&row.into_iter().collect::<String>());
        output.push('\n');
    }

    // Add Score
    output.push_str(&format!("\nScore: {}  Lives: {}\n", game.score, game.lives));

    // State dependent UI
    match game.state {
        GameState::Menu => {
            output.push_str("\n--- MENU ---\nPress Space to Play\nPress 't' for Bot Mode")
        },
        GameState::Paused => output.push_str("\n--- PAUSED ---\nPress 'p' to Resume"),
        GameState::GameOver => output.push_str("\n--- GAME OVER ---\nPress 'r' to Restart"),
        _ => {},
    }

    container.set_inner_text(&output);
}
