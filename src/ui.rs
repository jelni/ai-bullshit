use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Stdout, Write};

use crate::game::{Game, GameState};
use crate::snake::Direction;

pub fn draw(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    // Clear screen
    stdout.queue(Clear(ClearType::All))?;

    match game.state {
        GameState::Menu => draw_menu(game, stdout)?,
        GameState::Help => draw_help(game, stdout)?,
        GameState::Playing | GameState::GameOver | GameState::Paused => draw_game(game, stdout)?,
        GameState::EnterName => draw_enter_name(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

fn draw_enter_name(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
    draw_game(game, stdout)?;

    let title = "NEW HIGH SCORE!";
    let title_x = (game.config.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2);
    let title_y = game.config.height / 2 - 2;

    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(title_x, title_y))?;
    write!(stdout, "{title}")?;

    let prompt = "Enter your name:";
    let prompt_x = (game.config.width / 2).saturating_sub(u16::try_from(prompt.len()).unwrap() / 2);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(prompt_x, title_y + 1))?;
    write!(stdout, "{prompt}")?;

    let name_display = format!("> {}_", game.entered_name);
    let name_x = (game.config.width / 2).saturating_sub(u16::try_from(name_display.len()).unwrap() / 2);
    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo(name_x, title_y + 3))?;
    write!(stdout, "{name_display}")?;

    let enter_msg = "Press Enter to save";
    let enter_x = (game.config.width / 2).saturating_sub(u16::try_from(enter_msg.len()).unwrap() / 2);
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(enter_x, title_y + 5))?;
    write!(stdout, "{enter_msg}")?;

    stdout.queue(SetForegroundColor(Color::Reset))?;
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
    let scores = crate::game::Game::load_high_scores_static();
    if !scores.is_empty() {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(10), game.config.height / 2 + 6))?;
        write!(stdout, "Top Scores:")?;
        for (i, hs) in scores.iter().enumerate().take(5) {
            stdout.queue(cursor::MoveTo(
                (game.config.width / 2).saturating_sub(10),
                game.config.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{}. {} - {}", i + 1, hs.name, hs.score)?;
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

#[expect(clippy::too_many_lines)]
fn draw_game(game: &Game, stdout: &mut Stdout) -> io::Result<()> {
     let (border_color, food_color, snake_color, obs_color) = match game.config.theme.as_str() {
         "dark" => (Color::DarkGrey, Color::DarkRed, Color::Green, Color::DarkMagenta),
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

    // Draw food
    stdout.queue(cursor::MoveTo(game.food.location.x, game.food.location.y))?;
    if game.food.f_type == crate::game::FoodType::Shrink {
        stdout.queue(SetForegroundColor(Color::DarkRed))?;
        write!(stdout, "♦")?;
    } else {
        stdout.queue(SetForegroundColor(food_color))?;
        write!(stdout, "●")?;
    }

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
        && power_up.activation_time.is_none() {
            stdout.queue(cursor::MoveTo(power_up.location.x, power_up.location.y))?;
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            write!(stdout, "P")?;
        }

    // Draw snake
    stdout.queue(SetForegroundColor(snake_color))?;
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
             write!(stdout, "{}", game.config.skin)?;
        }
    }

    // Draw score
    let level = game.score / 20 + 1;
    let play_time_s = game.start_time.elapsed().as_secs();
    let mins = play_time_s / 60;
    let secs = play_time_s % 60;

    stdout.queue(SetForegroundColor(Color::Reset))?;
    stdout.queue(cursor::MoveTo(0, game.config.height))?;
    write!(
        stdout,
        "Score: {} | High: {} | Lives: {} | Level: {} | Time: {:02}:{:02}",
        game.score, game.high_score, game.lives, level, mins, secs
    )?;

    if let Some(power_up) = &game.power_up
        && let Some(activation_time) = power_up.activation_time {
            let elapsed = activation_time.elapsed().as_secs();
            if elapsed < 5 {
                let remaining = 5 - elapsed;
                let power_name = match power_up.p_type {
                    crate::game::PowerUpType::SlowDown => "Slowdown",
                    crate::game::PowerUpType::Invincibility => "Invincibility",
                    crate::game::PowerUpType::SpeedBoost => "SpeedBoost",
                };
                let power_up_msg = format!(" | {power_name}: {remaining}s");
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
         stdout.queue(SetForegroundColor(Color::Reset))?;
    }

    if game.state == GameState::Paused {
         let msg = "PAUSED";
         let msg_len = u16::try_from(msg.len()).unwrap();
         let x_pos = (game.config.width / 2).saturating_sub(msg_len / 2);
         let y_pos = game.config.height / 2 - 2;

         stdout.queue(SetForegroundColor(Color::Yellow))?;
         stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
         write!(stdout, "{msg}")?;

         let menu_items = ["Resume", "Save & Quit to Menu", "Quit Desktop"];
         for (i, item) in menu_items.iter().enumerate() {
             if i == game.pause_selection {
                 stdout.queue(SetForegroundColor(Color::Yellow))?;
                 stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2) - 2, y_pos + 2 + u16::try_from(i).unwrap_or(0)))?;
                 write!(stdout, "> {item} <")?;
             } else {
                 stdout.queue(SetForegroundColor(Color::White))?;
                 stdout.queue(cursor::MoveTo((game.config.width / 2).saturating_sub(u16::try_from(item.len()).unwrap() / 2), y_pos + 2 + u16::try_from(i).unwrap_or(0)))?;
                 write!(stdout, "{item}")?;
             }
         }

         stdout.queue(SetForegroundColor(Color::Reset))?;
    }
    Ok(())
}
