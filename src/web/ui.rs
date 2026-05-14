use crate::game::{Game, GameState};
use web_sys::CanvasRenderingContext2d;

pub fn draw(game: &Game, ctx: &CanvasRenderingContext2d) {
    let cell_size = 15.0;

    // Clear canvas
    ctx.set_fill_style_str("#000000");
    ctx.fill_rect(0.0, 0.0, f64::from(game.width) * cell_size, f64::from(game.height) * cell_size);

    // Draw borders
    ctx.set_fill_style_str("#555555");
    for y in 0..game.height {
        for x in 0..game.width {
            if x == 0 || x == game.width - 1 || y == 0 || y == game.height - 1 {
                ctx.fill_rect(
                    f64::from(x) * cell_size,
                    f64::from(y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw food
    ctx.set_fill_style_str("#00FF00");
    if game.food.x < game.width && game.food.y < game.height {
        ctx.fill_rect(
            f64::from(game.food.x) * cell_size,
            f64::from(game.food.y) * cell_size,
            cell_size,
            cell_size,
        );
    }

    // Draw obstacles
    ctx.set_fill_style_str("#FF0000");
    for obs in &game.obstacles {
        if obs.x < game.width && obs.y < game.height {
            ctx.fill_rect(
                f64::from(obs.x) * cell_size,
                f64::from(obs.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Portals
    if let Some((p1, p2)) = game.portals {
        if p1.x < game.width && p1.y < game.height {
            ctx.set_fill_style_str("#00FFFF"); // Cyan
            ctx.fill_rect(
                f64::from(p1.x) * cell_size,
                f64::from(p1.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
        if p2.x < game.width && p2.y < game.height {
            ctx.set_fill_style_str("#FF00FF"); // Magenta
            ctx.fill_rect(
                f64::from(p2.x) * cell_size,
                f64::from(p2.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw snake
    ctx.set_fill_style_str("#FFFFFF");
    for part in &game.snake.body {
        if part.x < game.width && part.y < game.height {
            ctx.fill_rect(
                f64::from(part.x) * cell_size,
                f64::from(part.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw overlays and texts
    ctx.set_fill_style_str("#FFFFFF");
    ctx.set_font("16px 'Courier New', Courier, monospace");
    ctx.set_text_align("left");

    let _ = ctx.fill_text(
        &format!("Score: {}  Lives: {}", game.score, game.lives),
        cell_size,
        cell_size * 1.5,
    );

    ctx.set_text_align("center");
    let center_x = (f64::from(game.width) * cell_size) / 2.0;
    let center_y = (f64::from(game.height) * cell_size) / 2.0;

    match game.state {
        GameState::Menu => {
            let _ = ctx.fill_text("--- MENU ---", center_x, center_y - 20.0);
            let _ = ctx.fill_text("Press Space to Play", center_x, center_y);
            let _ = ctx.fill_text("Press 't' for Bot Mode", center_x, center_y + 20.0);
        },
        GameState::Paused => {
            let _ = ctx.fill_text("--- PAUSED ---", center_x, center_y - 10.0);
            let _ = ctx.fill_text("Press 'p' to Resume", center_x, center_y + 10.0);
        },
        GameState::GameOver => {
            let _ = ctx.fill_text("--- GAME OVER ---", center_x, center_y - 10.0);
            let _ = ctx.fill_text("Press 'r' to Restart", center_x, center_y + 10.0);
        },
        _ => {},
    }
}
