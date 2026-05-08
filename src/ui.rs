use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};

use crate::game::{Game, GameState};
use crate::snake::Direction;

pub fn draw<W: Write>(game: &Game, writer: &mut W) -> io::Result<()> {
    // Clear screen
    writer.queue(Clear(ClearType::All))?;

    match game.state {
        GameState::Menu => draw_menu(game, writer)?,
        GameState::Help => draw_help(game, writer)?,
        GameState::Playing | GameState::GameOver | GameState::Paused => draw_game(game, writer)?,
    }

    writer.flush()?;
    Ok(())
}

pub fn draw_countdown<W: Write>(game: &Game, writer: &mut W, count: u32) -> io::Result<()> {
    draw_game(game, writer)?;
    let msg = format!("{count}");
    let x_pos = (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap() / 2);
    let y_pos = game.height / 2;

    writer.queue(SetForegroundColor(Color::White))?;
    writer.queue(cursor::MoveTo(x_pos, y_pos))?;
    write!(writer, "{msg}")?;
    writer.flush()?;
    Ok(())
}

fn draw_menu<W: Write>(game: &Game, writer: &mut W) -> io::Result<()> {
    let title = "SNAKE GAME";

    writer.queue(SetForegroundColor(Color::Green))?;
    writer.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2),
        game.height / 2 - 5,
    ))?;
    write!(writer, "{title}")?;

    let menu_items = ["Start Game", "Load Game", "Help", "Quit"];
    for (i, item) in menu_items.iter().enumerate() {
        if i == game.menu_selection {
            writer.queue(SetForegroundColor(Color::Yellow))?;
            writer.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2) - 2,
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(writer, "> {item} <")?;
        } else {
            writer.queue(SetForegroundColor(Color::White))?;
            writer.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(writer, "{item}")?;
        }
    }

    // Draw Leaderboard
    let scores = crate::game::Game::load_high_scores_static();
    if !scores.is_empty() {
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(10),
            game.height / 2 + 6,
        ))?;
        write!(writer, "Top Scores:")?;
        for (i, s) in scores.iter().enumerate().take(5) {
            writer.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(10),
                game.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(writer, "{}. {}", i + 1, s)?;
        }
    }
    Ok(())
}

fn draw_help<W: Write>(game: &Game, writer: &mut W) -> io::Result<()> {
    let title = "HELP & CONTROLS";
    let controls = [
        "Arrow Keys: Move Snake",
        "P: Pause / Resume",
        "S: Save Game (in Pause)",
        "L: Load Game (in Menu)",
        "Q: Quit / Back to Menu",
        "Space/Enter: Select/Start",
    ];
    let skin_line = format!("{} : Snake Body", game.skin);
    let legend = [
        "Symbols:",
        skin_line.as_str(),
        "● : Food (+1 Score)",
        "★ : Bonus Food (+5 Score)",
        "X : Obstacle (Avoid!)",
    ];

    writer.queue(SetForegroundColor(Color::Cyan))?;
    writer.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2),
        3,
    ))?;
    write!(writer, "{title}")?;

    writer.queue(SetForegroundColor(Color::White))?;
    for (i, line) in controls.iter().enumerate() {
        writer.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2),
            6 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(writer, "{line}")?;
    }

    writer.queue(SetForegroundColor(Color::Yellow))?;
    for (i, line) in legend.iter().enumerate() {
        // String ownership issue with format!, so we reconstruct or handle differently if needed.
        // legend array constructed above creates temporaries.
        // Let's print directly.
        writer.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2),
            14 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(writer, "{line}")?;
    }

    let back = "Press 'q' to go back";
    writer.queue(SetForegroundColor(Color::Red))?;
    writer.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(back.len()).unwrap() / 2),
        game.height - 2,
    ))?;
    write!(writer, "{back}")?;

    Ok(())
}

#[expect(clippy::too_many_lines)]
pub fn draw_game<W: Write>(game: &Game, writer: &mut W) -> io::Result<()> {
    let (border_color, food_color, snake_color, obs_color) = match game.theme.as_str() {
        "dark" => (
            Color::DarkGrey,
            Color::DarkRed,
            Color::Green,
            Color::DarkMagenta,
        ),
        "retro" => (Color::Green, Color::Green, Color::Green, Color::Green),
        "neon" => (Color::Cyan, Color::Magenta, Color::Yellow, Color::Red),
        _ => (Color::Blue, Color::Red, Color::DarkGreen, Color::Magenta),
    };

    // Draw borders
    if game.just_died {
        writer.queue(SetForegroundColor(Color::Red))?;
    } else {
        writer.queue(SetForegroundColor(border_color))?;
    }

    // Top border
    writer.queue(cursor::MoveTo(0, 0))?;
    for _ in 0..game.width {
        write!(writer, "#")?;
    }

    // Bottom border
    writer.queue(cursor::MoveTo(0, game.height - 1))?;
    for _ in 0..game.width {
        write!(writer, "#")?;
    }

    // Left and Right borders
    for y in 1..game.height - 1 {
        writer.queue(cursor::MoveTo(0, y))?;
        write!(writer, "#")?;
        writer.queue(cursor::MoveTo(game.width - 1, y))?;
        write!(writer, "#")?;
    }

    // Draw food
    writer.queue(cursor::MoveTo(game.food.x, game.food.y))?;
    writer.queue(SetForegroundColor(food_color))?;
    write!(writer, "●")?;

    // Draw obstacles
    writer.queue(SetForegroundColor(obs_color))?;
    for obs in &game.obstacles {
        writer.queue(cursor::MoveTo(obs.x, obs.y))?;
        write!(writer, "X")?;
    }

    // Draw bonus food
    if let Some((bonus_p, _)) = game.bonus_food {
        writer.queue(cursor::MoveTo(bonus_p.x, bonus_p.y))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        write!(writer, "★")?;
    }

    if let Some(power_up) = &game.power_up
        && power_up.activation_time.is_none()
    {
        writer.queue(cursor::MoveTo(power_up.location.x, power_up.location.y))?;
        writer.queue(SetForegroundColor(Color::Cyan))?;
        write!(writer, "P")?;
    }

    // Draw snake
    writer.queue(SetForegroundColor(snake_color))?;
    for (i, part) in game.snake.body.iter().enumerate() {
        writer.queue(cursor::MoveTo(part.x, part.y))?;
        if i == 0 {
            // Head
            let head_char = match game.snake.direction {
                Direction::Up => '^',
                Direction::Down => 'v',
                Direction::Left => '<',
                Direction::Right => '>',
            };
            write!(writer, "{head_char}")?;
        } else {
            // Body
            write!(writer, "{}", game.skin)?;
        }
    }

    // Draw score
    let level = game.score / 20 + 1;
    writer.queue(SetForegroundColor(Color::Reset))?;
    writer.queue(cursor::MoveTo(0, game.height))?;
    write!(
        writer,
        "Score: {} | High: {} | Lives: {} | Level: {}",
        game.score, game.high_score, game.lives, level
    )?;

    if let Some(power_up) = &game.power_up
        && let Some(activation_time) = power_up.activation_time
    {
        let elapsed = activation_time.elapsed().unwrap_or_default().as_secs();
        if elapsed < 5 {
            let remaining = 5 - elapsed;
            let power_up_msg = format!(" | Slowdown: {remaining}s");
            write!(writer, "{power_up_msg}")?;
        }
    }

    // Draw Game Over
    if game.state == GameState::GameOver {
        let msg = "GAME OVER";
        let msg_len = u16::try_from(msg.len()).unwrap();
        let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.height / 2;

        writer.queue(SetForegroundColor(Color::Red))?;
        writer.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(writer, "{msg}")?;

        let cause_msg = &game.death_message;
        let cause_len = u16::try_from(cause_msg.len()).unwrap();
        let x_cause = (game.width / 2).saturating_sub(cause_len / 2);
        writer.queue(SetForegroundColor(Color::White))?;
        writer.queue(cursor::MoveTo(x_cause, y_pos + 1))?;
        write!(writer, "{cause_msg}")?;

        let sub_msg = "Press 'q' to quit, 'r' to restart";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
        let x_sub = (game.width / 2).saturating_sub(sub_msg_len / 2);
        writer.queue(cursor::MoveTo(x_sub, y_pos + 2))?;
        write!(writer, "{sub_msg}")?;
        writer.queue(SetForegroundColor(Color::Reset))?;
    }

    if game.state == GameState::Paused {
        let msg = "PAUSED";
        let msg_len = u16::try_from(msg.len()).unwrap();
        let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.height / 2;

        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(writer, "{msg}")?;

        let sub_msg = "Press 's' to Save & Quit, 'p' to Resume";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
        let x_sub = (game.width / 2).saturating_sub(sub_msg_len / 2);
        writer.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
        write!(writer, "{sub_msg}")?;

        writer.queue(SetForegroundColor(Color::Reset))?;
    }
    Ok(())
}
