use crossterm::{
    QueueableCommand, cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

use crate::game::{Game, GameState};
use crate::snake::Direction;

pub fn draw<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    // Clear screen
    stdout.queue(Clear(ClearType::All))?;

    match game.state {
        GameState::Menu => draw_menu(game, stdout)?,
        GameState::Help => draw_help(game, stdout)?,
        GameState::Playing | GameState::GameOver | GameState::Paused | GameState::EnterName => draw_game(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

pub fn draw_countdown<W: Write>(game: &Game, stdout: &mut W, count: u32) -> io::Result<()> {
    draw_game(game, stdout)?;
    let msg = format!("{count}");
    let x_pos = (game.config.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap() / 2);
    let y_pos = game.config.height / 2;

    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
    write!(stdout, "{msg}")?;
    stdout.flush()?;
    Ok(())
}

fn draw_menu<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SNAKE GAME";

    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo(
        (game.config.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2),
        game.config.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    let menu_items = ["Start Game", "Load Game", "Help", "Quit"];
    for (i, item) in menu_items.iter().enumerate() {
        if i == game.menu_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2) - 2,
                game.config.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "> {item} <")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                (game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2),
                game.config.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{item}")?;
        }
    }

    // Draw Leaderboard
    let scores = &game.high_scores;
    if !scores.is_empty() {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(
            (game.config.width / 2).saturating_sub(10),
            game.config.height / 2 + 6,
        ))?;
        write!(stdout, "Top Scores:")?;
        for (i, s) in scores.iter().enumerate().take(5) {
            stdout.queue(cursor::MoveTo(
                (game.config.width / 2).saturating_sub(10),
                game.config.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{}. {}: {}", i + 1, s.name, s.score)?;
        }
    }
    Ok(())
}

fn draw_help<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "HELP & CONTROLS";
    let controls = [
        "Arrow Keys: Move Snake",
        "P: Pause / Resume",
        "S: Save Game (in Pause)",
        "L: Load Game (in Menu)",
        "Q: Quit / Back to Menu",
        "Space/Enter: Select/Start",
    ];
    let skin_line = format!("{} : Snake Body", game.config.skin);
    let legend = [
        "Symbols:",
        skin_line.as_str(),
        "● : Food (+1 Score)",
        "★ : Bonus Food (+5 Score)",
        "X : Obstacle (Avoid!)",
    ];

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.config.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2),
        3,
    ))?;
    write!(stdout, "{title}")?;

    stdout.queue(SetForegroundColor(Color::White))?;
    for (i, line) in controls.iter().enumerate() {
        stdout.queue(cursor::MoveTo(
            (game.config.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2),
            6 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{line}")?;
    }

    stdout.queue(SetForegroundColor(Color::Yellow))?;
    for (i, line) in legend.iter().enumerate() {
        // String ownership issue with format!, so we reconstruct or handle differently if needed.
        // legend array constructed above creates temporaries.
        // Let's print directly.
        stdout.queue(cursor::MoveTo(
            (game.config.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2),
            14 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{line}")?;
    }

    let back = "Press 'q' to go back";
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(cursor::MoveTo(
        (game.config.width / 2).saturating_sub(u16::try_from(back.len()).unwrap() / 2),
        game.config.height - 2,
    ))?;
    write!(stdout, "{back}")?;

    Ok(())
}

#[expect(clippy::too_many_lines)]
fn draw_game<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let (border_color, food_color, snake_color, obs_color) = match game.config.theme.as_str() {
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
        stdout.queue(SetForegroundColor(Color::Red))?;
    } else {
        stdout.queue(SetForegroundColor(border_color))?;
    }

    for y in 0..game.config.height {
        for x in 0..game.config.width {
            if x == 0 || x == game.config.width - 1 || y == 0 || y == game.config.height - 1 {
                stdout.queue(cursor::MoveTo(x, y))?;
                write!(stdout, "#")?;
            }
        }
    }

    // Draw Status Bar
    let status = format!(" Score: {} | Lives: {} ", game.score, game.lives);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(2, 0))?;
    write!(stdout, "{status}")?;

    // Draw food
    stdout.queue(cursor::MoveTo(game.food.x, game.food.y))?;
    stdout.queue(SetForegroundColor(food_color))?;
    write!(stdout, "●")?;

    // Draw obstacles
    stdout.queue(SetForegroundColor(obs_color))?;
    for obs in &game.obstacles {
        stdout.queue(cursor::MoveTo(obs.x, obs.y))?;
        write!(stdout, "X")?;
    }

    // Draw bonus food
    if let Some((bonus_p, _)) = game.bonus_food {
        stdout.queue(cursor::MoveTo(bonus_p.x, bonus_p.y))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "★")?;
    }

    if let Some(power_up) = &game.power_up
        && power_up.activation_time.is_none()
    {
        stdout.queue(cursor::MoveTo(power_up.location.x, power_up.location.y))?;
        match power_up.p_type {
            crate::game::PowerUpType::SlowDown => {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "S")?;
            }
            crate::game::PowerUpType::Invincibility => {
                stdout.queue(SetForegroundColor(Color::Magenta))?;
                write!(stdout, "I")?;
            }
        }
    }

    // Check if invincible
    let is_invincible = game.power_up.as_ref().is_some_and(|p| {
        p.p_type == crate::game::PowerUpType::Invincibility && p.activation_time.is_some()
    });
    let actual_snake_color = if is_invincible {
        Color::Magenta
    } else {
        snake_color
    };

    // Draw snake
    stdout.queue(SetForegroundColor(actual_snake_color))?;
    for (i, part) in game.snake.body.iter().enumerate() {
        stdout.queue(cursor::MoveTo(part.x, part.y))?;
        if i == 0 {
            // Head
            let head_char = match game.snake.direction {
                Direction::Up => '^',
                Direction::Down => 'v',
                Direction::Left => '<',
                Direction::Right => '>',
            };
            write!(stdout, "{head_char}")?;
        } else {
            // Body
            let body_char = if is_invincible { '*' } else { game.config.skin };
            write!(stdout, "{body_char}")?;
        }
    }

    // Draw score
    let level = game.score / 20 + 1;
    stdout.queue(SetForegroundColor(Color::Reset))?;
    stdout.queue(cursor::MoveTo(0, game.config.height))?;
    write!(
        stdout,
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
            write!(stdout, "{power_up_msg}")?;
        }
    }

    // Draw Game Over
    if game.state == GameState::GameOver {
        let msg = "GAME OVER";
        let msg_len = u16::try_from(msg.len()).unwrap();
        let x_pos = (game.config.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.config.height / 2;

        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let cause_msg = &game.death_message;
        let cause_len = u16::try_from(cause_msg.len()).unwrap();
        let x_cause = (game.config.width / 2).saturating_sub(cause_len / 2);
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(x_cause, y_pos + 1))?;
        write!(stdout, "{cause_msg}")?;

        let sub_msg = "Press 'q' to quit, 'r' to restart";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
        let x_sub = (game.config.width / 2).saturating_sub(sub_msg_len / 2);
        stdout.queue(cursor::MoveTo(x_sub, y_pos + 2))?;
        write!(stdout, "{sub_msg}")?;

        if !game.high_scores.is_empty() {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(5), y_pos + 4))?;
            write!(stdout, "Top Scores:")?;

            stdout.queue(SetForegroundColor(Color::White))?;
            for (i, s) in game.high_scores.iter().enumerate().take(5) {
                let s_msg = format!("{}. {}: {}", i + 1, s.name, s.score);
                let s_len = u16::try_from(s_msg.len()).unwrap_or(0);
                stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(s_len / 2), y_pos + 5 + u16::try_from(i).unwrap_or(0)))?;
                write!(stdout, "{s_msg}")?;
            }
        }
        stdout.queue(SetForegroundColor(Color::Reset))?;
    }

    if game.state == GameState::EnterName {
        let msg = "NEW HIGH SCORE!";
        let msg_len = u16::try_from(msg.len()).unwrap();
        let x_pos = (game.config.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.config.height / 2 - 2;

        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let prompt = "Enter Name:";
        let prompt_len = u16::try_from(prompt.len()).unwrap();
        stdout.queue(cursor::MoveTo(
            (game.config.width / 2).saturating_sub(prompt_len / 2),
            y_pos + 1,
        ))?;
        write!(stdout, "{prompt}")?;

        let name = format!("{}_", game.input_buffer);
        let name_len = u16::try_from(name.len()).unwrap();
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(
            (game.config.width / 2).saturating_sub(name_len / 2),
            y_pos + 2,
        ))?;
        write!(stdout, "{name}")?;
    }

    if game.state == GameState::Paused {
        let msg = "PAUSED";
        let msg_len = u16::try_from(msg.len()).unwrap();
        let x_pos = (game.config.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.config.height / 2 - 2;

        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let stats_msg = format!(
            "Score: {} | Lives: {} | Time: {}s",
            game.score,
            game.lives,
            game.start_time.elapsed().as_secs()
        );
        let stats_len = u16::try_from(stats_msg.len()).unwrap_or(0);
        let x_stats = (game.config.width / 2).saturating_sub(stats_len / 2);
        stdout.queue(SetForegroundColor(Color::Cyan))?;
        stdout.queue(cursor::MoveTo(x_stats, y_pos + 2))?;
        write!(stdout, "{stats_msg}")?;

        let sub_msg = "Press 's' to Save & Quit, 'p' to Resume";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
        let x_sub = (game.config.width / 2).saturating_sub(sub_msg_len / 2);
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(x_sub, y_pos + 4))?;
        write!(stdout, "{sub_msg}")?;

        stdout.queue(SetForegroundColor(Color::Reset))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    fn get_expected_ansi_tail(x: u16, y: u16, msg: &str) -> String {
        let mut expected_buf = Vec::new();
        expected_buf.queue(SetForegroundColor(Color::White)).unwrap();
        expected_buf.queue(cursor::MoveTo(x, y)).unwrap();
        write!(expected_buf, "{msg}").unwrap();
        String::from_utf8(expected_buf).unwrap()
    }

    #[test]
    fn test_draw_countdown() {
        let game = Game::new(crate::config::GameConfig { width: 20, height: 20, wrap_mode: false, skin: 'O', theme: "dark".to_string(), difficulty: crate::config::Difficulty::Normal });

        // Test single digit (count = 3)
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 3).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // center is width/2 (10), msg.len() is 1, so 1/2 is 0. 10 - 0 = 10.
        let expected = get_expected_ansi_tail(10, 10, "3");
        assert!(output.ends_with(&expected), "Expected output to end with drawing '3' at (10, 10)");

        // Test double digit (count = 10) to test centering subtraction
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 10).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // msg.len() is 2, so 2/2 is 1. 10 - 1 = 9.
        let expected = get_expected_ansi_tail(9, 10, "10");
        assert!(output.ends_with(&expected), "Expected output to end with drawing '10' at (9, 10)");

        // Test count = 0
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 0).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let expected = get_expected_ansi_tail(10, 10, "0");
        assert!(output.ends_with(&expected), "Expected output to end with drawing '0' at (10, 10)");
    }
}
