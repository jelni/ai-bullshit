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

    // Draw poison food
    if let Some((poison_p, _)) = game.poison_food {
        ctx.set_fill_style_str("#800080"); // DarkMagenta
        if poison_p.x < game.width && poison_p.y < game.height {
            ctx.fill_rect(
                f64::from(poison_p.x) * cell_size,
                f64::from(poison_p.y) * cell_size,
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

    // Draw boss
    if let Some(boss) = &game.boss {
        ctx.set_fill_style_str("#FF00FF");
        if boss.position.x < game.width && boss.position.y < game.height {
            ctx.fill_rect(
                f64::from(boss.position.x) * cell_size,
                f64::from(boss.position.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw bonus food
    if let Some((bonus_p, _)) = game.bonus_food {
        ctx.set_fill_style_str("#FFFF00");
        if bonus_p.x < game.width && bonus_p.y < game.height {
            ctx.fill_rect(
                f64::from(bonus_p.x) * cell_size,
                f64::from(bonus_p.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw power_up
    if let Some(power_up) = &game.power_up {
        if power_up.activation_time.is_none() && power_up.location.x < game.width && power_up.location.y < game.height {
            ctx.set_fill_style_str("#00FFFF");
            ctx.fill_rect(
                f64::from(power_up.location.x) * cell_size,
                f64::from(power_up.location.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw lasers
    for laser in &game.lasers {
        if laser.position.x < game.width && laser.position.y < game.height {
            if laser.player == 1 {
                ctx.set_fill_style_str("#FFFFFF");
            } else {
                ctx.set_fill_style_str("#0000FF");
            }
            ctx.fill_rect(
                f64::from(laser.position.x) * cell_size,
                f64::from(laser.position.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw particles
    for p in &game.particles {
        let px = p.x.round() as u16;
        let py = p.y.round() as u16;
        if px < game.width && py < game.height {
            ctx.set_fill_style_str("#888888");
            ctx.fill_rect(
                f64::from(px) * cell_size,
                f64::from(py) * cell_size,
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

    // Draw player2
    if let Some(p2) = &game.player2 {
        ctx.set_fill_style_str("#0000FF");
        for part in &p2.body {
            if part.x < game.width && part.y < game.height {
                ctx.fill_rect(
                    f64::from(part.x) * cell_size,
                    f64::from(part.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
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
