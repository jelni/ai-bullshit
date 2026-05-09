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
        GameState::Stats => draw_stats(game, stdout)?,
        GameState::Playing | GameState::GameOver | GameState::GameWon | GameState::Paused => draw_game(game, stdout)?,
        GameState::EnterName => draw_enter_name(game, stdout)?,
        GameState::ConfirmQuit => draw_confirm_quit(game, stdout)?,
        GameState::Settings => draw_settings(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

pub fn draw_countdown<W: Write>(game: &Game, stdout: &mut W, count: u32) -> io::Result<()> {
    draw_game(game, stdout)?;
    let msg = format!("{count}");
    let x_pos = (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2);
    let y_pos = game.height / 2;

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
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    let menu_items = ["Start Game", "Load Game", "Settings", "Statistics", "Help", "Quit"];
    for (i, item) in menu_items.iter().enumerate() {
        if i == game.menu_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2) - 2,
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "> {item} <")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{item}")?;
        }
    }

    // Draw Leaderboard
    let scores = &game.high_scores;
    if !scores.is_empty() {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(10),
            game.height / 2 + 6,
        ))?;
        write!(stdout, "Top Scores:")?;
        for (i, (name, score)) in scores.iter().enumerate().take(5) {
            let hs_str = format!("{}. {} - {}", i + 1, name, score);
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(10),
                game.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{hs_str}")?;
        }
    }
    Ok(())
}

fn draw_stats<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "STATISTICS";

    let stats = [
        format!("Games Played: {}", game.stats.games_played),
        format!("Total Score: {}", game.stats.total_score),
        format!("Total Food Eaten: {}", game.stats.total_food_eaten),
        format!("Total Time (s): {}", game.stats.total_time_s),
    ];

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    stdout.queue(SetForegroundColor(Color::White))?;
    for (i, line) in stats.iter().enumerate() {
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{line}")?;
    }

    let back = "Press any key to go back";
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(back.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{back}")?;

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
    let skin_line = format!("{} : Snake Body", game.skin);
    let legend = [
        "Symbols:",
        skin_line.as_str(),
        "● : Food (+1 Score)",
        "★ : Bonus Food (+5 Score)",
        "X : Obstacle (Avoid!)",
    ];

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        3,
    ))?;
    write!(stdout, "{title}")?;

    stdout.queue(SetForegroundColor(Color::White))?;
    for (i, line) in controls.iter().enumerate() {
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
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
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            14 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{line}")?;
    }

    let back = "Press 'q' to go back";
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(back.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{back}")?;

    Ok(())
}

fn draw_enter_name<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "NEW HIGH SCORE!";
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 2,
    ))?;
    write!(stdout, "{title}")?;

    let prompt = "Enter your name:";
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(prompt.len()).unwrap_or(0) / 2),
        game.height / 2,
    ))?;
    write!(stdout, "{prompt}")?;

    let name_str = format!("> {} <", game.player_name);
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(name_str.len()).unwrap_or(0) / 2),
        game.height / 2 + 2,
    ))?;
    write!(stdout, "{name_str}")?;

    Ok(())
}


fn draw_settings<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SETTINGS";
    let title_len = u16::try_from(title.len()).unwrap_or(0);

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(title_len / 2),
        game.height / 4,
    ))?;
    write!(stdout, "{title}")?;

    let settings_items = [
        format!("Difficulty: {:?}", game.difficulty),
        format!("Theme: {:?}", game.theme),
        format!("Wrap Mode: {}", if game.wrap_mode { "On" } else { "Off" }),
    ];

    for (i, item) in settings_items.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2) - 2,
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "> {item} <")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{item}")?;
        }
    }

    let help_msg = "Use UP/DOWN to select, LEFT/RIGHT to change, Q to go back";
    let help_len = u16::try_from(help_msg.len()).unwrap_or(0);
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(help_len / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help_msg}")?;

    Ok(())
}

fn draw_confirm_quit<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "ARE YOU SURE YOU WANT TO QUIT?";
    stdout.queue(SetForegroundColor(Color::Red))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 1,
    ))?;
    write!(stdout, "{title}")?;

    let options = "[Y]es / [N]o";
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(options.len()).unwrap_or(0) / 2),
        game.height / 2 + 1,
    ))?;
    write!(stdout, "{options}")?;

    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "Game drawing requires extensive setup"
)]
fn draw_game<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let (border_color, food_color, snake_color, obs_color) = match game.theme {
        crate::game::Theme::Dark => (
            Color::DarkGrey,
            Color::DarkRed,
            Color::Green,
            Color::DarkMagenta,
        ),
        crate::game::Theme::Retro => (Color::Green, Color::Green, Color::Green, Color::Green),
        crate::game::Theme::Neon => (Color::Cyan, Color::Magenta, Color::Yellow, Color::Red),
        crate::game::Theme::Classic => (Color::Blue, Color::Red, Color::DarkGreen, Color::Magenta),
    };

    // Draw borders
    if game.just_died {
        stdout.queue(SetForegroundColor(Color::Red))?;
    } else {
        stdout.queue(SetForegroundColor(border_color))?;
    }

    // Top border
    stdout.queue(cursor::MoveTo(0, 0))?;
    let mut top_border = String::from("╔");
    top_border.push_str(&"═".repeat(usize::from(game.width).saturating_sub(2)));
    top_border.push('╗');
    write!(stdout, "{top_border}")?;

    // Bottom border
    stdout.queue(cursor::MoveTo(0, game.height - 1))?;
    let mut bottom_border = String::from("╚");
    bottom_border.push_str(&"═".repeat(usize::from(game.width).saturating_sub(2)));
    bottom_border.push('╝');
    write!(stdout, "{bottom_border}")?;

    // Side borders
    for y in 1..game.height - 1 {
        stdout.queue(cursor::MoveTo(0, y))?;
        write!(stdout, "║")?;
        stdout.queue(cursor::MoveTo(game.width - 1, y))?;
        write!(stdout, "║")?;
    }

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

    #[expect(clippy::collapsible_if, reason = "stable rust")]
    if let Some(power_up) = &game.power_up {
        if power_up.activation_time.is_none() {
            stdout.queue(cursor::MoveTo(power_up.location.x, power_up.location.y))?;
            if power_up.p_type == crate::game::PowerUpType::ExtraLife {
                stdout.queue(SetForegroundColor(Color::Magenta))?;
                write!(stdout, "♥")?;
            } else if power_up.p_type == crate::game::PowerUpType::PassThroughWalls {
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                write!(stdout, "W")?;
            } else {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "P")?;
            }
        }
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
            write!(stdout, "{}", game.skin)?;
        }
    }

    // Draw score
    let level = game.score / 20 + 1;
    stdout.queue(SetForegroundColor(Color::Reset))?;
    stdout.queue(cursor::MoveTo(0, game.height))?;
    write!(
        stdout,
        "Score: {} | High: {} | Lives: {} | Level: {}",
        game.score, game.high_score, game.lives, level
    )?;

    #[expect(clippy::collapsible_if, reason = "stable rust")]
    if let Some(power_up) = &game.power_up {
        if let Some(activation_time) = power_up.activation_time {
            let elapsed = activation_time.elapsed().unwrap_or_default().as_secs();
            if elapsed < 5 {
                let remaining = 5 - elapsed;
                let power_up_name = match power_up.p_type {
                    crate::game::PowerUpType::SlowDown => "Slowdown",
                    crate::game::PowerUpType::SpeedBoost => "Speed Boost",
                    crate::game::PowerUpType::Invincibility => "Invincible",
                    crate::game::PowerUpType::ExtraLife => "Extra Life",
                    crate::game::PowerUpType::PassThroughWalls => "Ghost",
                };
                let power_up_msg = format!(" | {power_up_name}: {remaining}s");
                write!(stdout, "{power_up_msg}")?;
            }
        }
    }

    // Draw Game Over
    if game.state == GameState::GameOver {
        let msg = "GAME OVER";
        let msg_len = u16::try_from(msg.len()).unwrap_or(0);
        let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.height / 2;

        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let cause_msg = &game.death_message;
        let cause_len = u16::try_from(cause_msg.len()).unwrap_or(0);
        let x_cause = (game.width / 2).saturating_sub(cause_len / 2);
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(x_cause, y_pos + 1))?;
        write!(stdout, "{cause_msg}")?;

        let sub_msg = "Press 'q' to quit, 'r' to restart";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap_or(0);
        let x_sub = (game.width / 2).saturating_sub(sub_msg_len / 2);
        stdout.queue(cursor::MoveTo(x_sub, y_pos + 2))?;
        write!(stdout, "{sub_msg}")?;
        stdout.queue(SetForegroundColor(Color::Reset))?;
    }

    if game.state == GameState::GameWon {
        let msg = "YOU WIN!";
        let msg_len = u16::try_from(msg.len()).unwrap_or(0);
        let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.height / 2;

        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let sub_msg = "Press 'q' to quit, 'r' to restart";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap_or(0);
        let x_sub = (game.width / 2).saturating_sub(sub_msg_len / 2);
        stdout.queue(cursor::MoveTo(x_sub, y_pos + 2))?;
        write!(stdout, "{sub_msg}")?;
        stdout.queue(SetForegroundColor(Color::Reset))?;
    }

    if game.state == GameState::Paused {
        let msg = "PAUSED";
        let msg_len = u16::try_from(msg.len()).unwrap_or(0);
        let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
        let y_pos = game.height / 2;

        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
        write!(stdout, "{msg}")?;

        let sub_msg = "Press 's' to Save & Quit, 'p' to Resume";
        let sub_msg_len = u16::try_from(sub_msg.len()).unwrap_or(0);
        let x_sub = (game.width / 2).saturating_sub(sub_msg_len / 2);
        stdout.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
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
        expected_buf
            .queue(SetForegroundColor(Color::White))
            .expect("Valid operation in tests");
        expected_buf
            .queue(cursor::MoveTo(x, y))
            .expect("Valid operation in tests");
        write!(expected_buf, "{msg}").expect("Valid operation in tests");
        String::from_utf8(expected_buf).expect("Valid operation in tests")
    }

    #[test]
    fn test_draw_menu() {
        let mut game = Game::new(
            20,
            20,
            false,
            'O',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        game.menu_selection = 0; // "Start Game" selected

        let mut buf = Vec::new();
        draw_menu(&game, &mut buf).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");

        // Check title
        assert!(output.contains("SNAKE GAME"), "Menu should contain title");

        // Check selection indicator
        assert!(
            output.contains("> Start Game <"),
            "Menu should indicate selection"
        );
        assert!(
            output.contains("Load Game"),
            "Menu should contain other items"
        );
        assert!(
            output.contains("Statistics"),
            "Menu should contain Statistics item"
        );
        assert!(
            !output.contains("> Load Game <"),
            "Unselected items should not have brackets"
        );
    }

    #[test]
    fn test_draw_help() {
        let game = Game::new(
            20,
            20,
            false,
            'O',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );

        let mut buf = Vec::new();
        draw_help(&game, &mut buf).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");

        assert!(
            output.contains("HELP & CONTROLS"),
            "Help should contain title"
        );
        assert!(
            output.contains("Arrow Keys: Move Snake"),
            "Help should contain controls"
        );
        assert!(
            output.contains("O : Snake Body"),
            "Help should contain dynamic skin info"
        );
    }

    #[test]
    fn test_draw_countdown() {
        let game = Game::new(
            20,
            20,
            false,
            'O',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );

        // Test single digit (count = 3)
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 3).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        // center is width/2 (10), msg.len() is 1, so 1/2 is 0. 10 - 0 = 10.
        let expected = get_expected_ansi_tail(10, 10, "3");
        assert!(
            output.ends_with(&expected),
            "Expected output to end with drawing '3' at (10, 10)"
        );

        // Test double digit (count = 10) to test centering subtraction
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 10).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        // msg.len() is 2, so 2/2 is 1. 10 - 1 = 9.
        let expected = get_expected_ansi_tail(9, 10, "10");
        assert!(
            output.ends_with(&expected),
            "Expected output to end with drawing '10' at (9, 10)"
        );

        // Test count = 0
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 0).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        let expected = get_expected_ansi_tail(10, 10, "0");
        assert!(
            output.ends_with(&expected),
            "Expected output to end with drawing '0' at (10, 10)"
        );

        // Test large width board
        let large_game = Game::new(
            100,
            100,
            false,
            'O',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        let mut buf = Vec::new();
        draw_countdown(&large_game, &mut buf, 5).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        let expected = get_expected_ansi_tail(50, 50, "5");
        assert!(
            output.ends_with(&expected),
            "Expected output to center correctly on large board"
        );

        // Test large digit (count = 12345)
        let mut buf = Vec::new();
        draw_countdown(&large_game, &mut buf, 12345).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        // msg.len() is 5, so 5/2 is 2. 50 - 2 = 48.
        let expected = get_expected_ansi_tail(48, 50, "12345");
        assert!(
            output.ends_with(&expected),
            "Expected output to center large digits correctly"
        );
    }
}

#[cfg(test)]
mod settings_tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn test_draw_settings() {
        let mut game = Game::new(
            40,
            20,
            false,
            '#',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        game.state = GameState::Settings;
        game.settings_selection = 1; // Theme selected

        let mut buf = Vec::new();
        draw_settings(&game, &mut buf).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");

        assert!(output.contains("SETTINGS"), "Settings should contain title");
        assert!(
            output.contains("Difficulty: Normal"),
            "Settings should show Difficulty"
        );
        assert!(
            output.contains("> Theme: Dark <"),
            "Settings should indicate selected item"
        );
        assert!(
            output.contains("Wrap Mode: Off"),
            "Settings should show Wrap Mode"
        );
    }
}
