use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Stdout, Write};

use crate::game::{FoodType, Game, GameState, PowerUpType};
use crate::snake::Direction;

pub fn draw(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    // Clear screen
    stdout.queue(Clear(ClearType::All))?;

    match game.state {
        GameState::Menu => draw_menu(game, stdout)?,
        GameState::Help => draw_help(game, stdout)?,
        GameState::Playing | GameState::GameOver | GameState::Paused => draw_game(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

fn draw_power_ups(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    if let Some((pos, spawn_time, power_up_type)) = game.power_up {
        if spawn_time.elapsed().as_millis() % 1000 < 500 {
            stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
            match power_up_type {
                PowerUpType::SpeedBoost => {
                    stdout.queue(SetForegroundColor(Color::Blue))?;
                    write!(stdout, "SPD")?;
                }
                PowerUpType::Invincibility => {
                    stdout.queue(SetForegroundColor(Color::Rgb { r: 255, g: 165, b: 0 }))?;
                    write!(stdout, "INV")?;
                }
            }
        }
    }
    Ok(())
}

pub fn draw_countdown(game: &Game, stdout: &mut Stdout, count: u32) -> io::Result<()> {
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

fn draw_menu(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    let title = "SNAKE GAME";

    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2), game.config.height / 2 - 5))?;
    write!(stdout, "{title}")?;

    let menu_items = ["Start Game", "Load Game", "Help", "Quit"];
    for (i, item) in menu_items.iter().enumerate() {
        if i == game.menu_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2) - 2, game.config.height / 2 - 2 + u16::try_from(i).unwrap_or(0)))?;
            write!(stdout, "> {item} <")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2), game.config.height / 2 - 2 + u16::try_from(i).unwrap_or(0)))?;
            write!(stdout, "{item}")?;
        }
    }

    // Draw Leaderboard
    let scores = crate::game::Game::load_high_scores();
    if !scores.is_empty() {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(10), game.config.height / 2 + 6))?;
        write!(stdout, "Top Scores:")?;
        for (i, entry) in scores.iter().enumerate().take(5) {
            let text = format!("{}. {} - {}", i + 1, entry.name, entry.score);
            stdout.queue(cursor::MoveTo(
                (game.config.width / 2).saturating_sub(10),
                game.config.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{text}")?;
        }
    }
    Ok(())
}

fn draw_help(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
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
    stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2), 3))?;
    write!(stdout, "{title}")?;

    stdout.queue(SetForegroundColor(Color::White))?;
    for (i, line) in controls.iter().enumerate() {
        stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2), 6 + u16::try_from(i).unwrap_or(0)))?;
        write!(stdout, "{line}")?;
    }

    stdout.queue(SetForegroundColor(Color::Yellow))?;
    for (i, line) in legend.iter().enumerate() {
        // String ownership issue with format!, so we reconstruct or handle differently if needed.
        // legend array constructed above creates temporaries.
        // Let's print directly.
        stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(line.len()).unwrap() / 2), 14 + u16::try_from(i).unwrap_or(0)))?;
        write!(stdout, "{line}")?;
    }

    let back = "Press 'q' to go back";
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(back.len()).unwrap() / 2), game.config.height - 2))?;
    write!(stdout, "{back}")?;

    Ok(())
}

fn draw_game(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    let (border_color, food_color, snake_color, obs_color) = match game.config.theme.as_str() {
        "dark" => (Color::DarkGrey, Color::DarkRed, Color::Green, Color::DarkMagenta),
        "retro" => (Color::Green, Color::Green, Color::Green, Color::Green),
        "neon" => (Color::Cyan, Color::Magenta, Color::Yellow, Color::Red),
        _ => (Color::Blue, Color::Red, Color::DarkGreen, Color::Magenta),
    };

    draw_borders(game, stdout, border_color)?;
    draw_food(game, stdout, food_color)?;
    draw_obstacles(game, stdout, obs_color)?;
    draw_snake(game, stdout, snake_color)?;
    draw_score(game, stdout)?;
    draw_power_ups(game, stdout)?;

    if game.state == GameState::GameOver {
        draw_game_over(game, stdout)?;
    } else if game.state == GameState::Paused {
        draw_paused(game, stdout)?;
    }

    Ok(())
}

fn draw_borders(game: &Game, stdout: &mut Stdout, color: Color) -> io::Result<()> {
    if game.just_died {
        stdout.queue(SetForegroundColor(Color::Red))?;
    } else {
        stdout.queue(SetForegroundColor(color))?;
    }

    for y in 0..game.config.height {
        for x in 0..game.config.width {
            if x == 0 || x == game.config.width - 1 || y == 0 || y == game.config.height - 1 {
                stdout.queue(cursor::MoveTo(x, y))?;
                write!(stdout, "#")?;
            }
        }
    }
    Ok(())
}

fn draw_food(game: &Game, stdout: &mut Stdout, color: Color) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(game.food.x, game.food.y))?;
    stdout.queue(SetForegroundColor(color))?;
    write!(stdout, "●")?;

    if let Some((pos, spawn_time, food_type)) = game.special_food {
        if spawn_time.elapsed().as_millis() % 1000 < 500 {
            stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
            match food_type {
                FoodType::Golden => {
                    stdout.queue(SetForegroundColor(Color::Yellow))?;
                    write!(stdout, "★")?;
                }
                FoodType::ScoreBoost => {
                    stdout.queue(SetForegroundColor(Color::Cyan))?;
                    write!(stdout, "S")?;
                }
            }
        }
    }
    Ok(())
}

fn draw_obstacles(game: &Game, stdout: &mut Stdout, color: Color) -> io::Result<()> {
    stdout.queue(SetForegroundColor(color))?;
    for obs in &game.obstacles {
        stdout.queue(cursor::MoveTo(obs.x, obs.y))?;
        write!(stdout, "X")?;
    }
    Ok(())
}

fn draw_snake(game: &Game, stdout: &mut Stdout, color: Color) -> io::Result<()> {
    stdout.queue(SetForegroundColor(color))?;
    for (i, part) in game.snake.body.iter().enumerate() {
        stdout.queue(cursor::MoveTo(part.x, part.y))?;
        if i == 0 {
            let head_char = match game.snake.direction {
                Direction::Up => '^',
                Direction::Down => 'v',
                Direction::Left => '<',
                Direction::Right => '>',
            };
            write!(stdout, "{head_char}")?;
        } else {
            write!(stdout, "{}", game.config.skin)?;
        }
    }
    Ok(())
}

fn draw_score(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    let level = game.score / 20 + 1;
    stdout.queue(SetForegroundColor(Color::Reset))?;
    stdout.queue(cursor::MoveTo(0, game.config.height))?;
    write!(
        stdout,
        "Score: {} | High: {} | Lives: {} | Level: {}",
        game.score, game.high_score, game.lives, level
    )?;
    Ok(())
}

fn draw_game_over(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
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
    stdout.queue(SetForegroundColor(Color::Reset))?;
    Ok(())
}

fn draw_paused(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    let msg = "PAUSED";
    let msg_len = u16::try_from(msg.len()).unwrap();
    let x_pos = (game.config.width / 2).saturating_sub(msg_len / 2);
    let y_pos = game.config.height / 2;

    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
    write!(stdout, "{msg}")?;

    let sub_msg = "Press 's' to Save & Quit, 'p' to Resume";
    let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
    let x_sub = (game.config.width / 2).saturating_sub(sub_msg_len / 2);
    stdout.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
    write!(stdout, "{sub_msg}")?;

    stdout.queue(SetForegroundColor(Color::Reset))?;
    Ok(())
}
