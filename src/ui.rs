use std::io::{self, Write};

use crossterm::{
    QueueableCommand, cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    game::{Game, GameState},
    snake::Direction,
};

/// # Errors
///
/// Returns an error if it fails to write to `stdout` or flush the buffer.
pub fn draw<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    // Clear screen
    stdout.queue(Clear(ClearType::All))?;

    match game.state {
        GameState::Menu => draw_menu(game, stdout)?,
        GameState::Help => draw_help(game, stdout)?,
        GameState::Stats => draw_stats(game, stdout)?,
        GameState::Playing | GameState::GameOver | GameState::GameWon | GameState::Paused => {
            draw_game(game, stdout)?;
        },
        GameState::EnterName => draw_enter_name(game, stdout)?,
        GameState::ConfirmQuit => draw_confirm_quit(game, stdout)?,
        GameState::Settings => draw_settings(game, stdout)?,
        GameState::NftShop => draw_nft_shop(game, stdout)?,
        GameState::Achievements => draw_achievements(game, stdout)?,
        GameState::LevelEditor => draw_level_editor(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

/// # Errors
///
/// Returns an error if it fails to write to `stdout` or flush the buffer.
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

    let menu_items = [
        "Single Player",
        "Campaign Mode",
        "Local Multiplayer",
        "Online Multiplayer",
        "Player vs Bot",
        "Bot vs Bot",
        "Battle Royale",
        "Time Attack",
        "Survival Mode",
        "Zen Mode",
        "Maze Mode",
        "Cave Mode",
        "Speedrun Mode",
        "Load Game",
        "Settings",
        "NFT Shop",
        "Statistics",
        "Achievements",
        "Help",
        "Play Custom Level",
        "Level Editor",
        "Quit",
    ];
    for (i, item) in menu_items.iter().enumerate() {
        if i == game.menu_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
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
        stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(10), game.height / 2 + 6))?;
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

fn draw_achievements<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "ACHIEVEMENTS";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 4,
    ))?;
    write!(stdout, "{title}")?;

    let all_achievements = [
        (crate::game::Achievement::FirstBlood, "First Blood (Play a game)"),
        (crate::game::Achievement::HighScorer, "High Scorer (Score 100+)"),
        (crate::game::Achievement::Rich, "Rich (Accumulate 1000+ coins)"),
        (crate::game::Achievement::BotUser, "Bot User (Use the bot)"),
    ];

    for (i, (ach, desc)) in all_achievements.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_achievements.contains(ach);
        let prefix = if is_unlocked {
            "[X]"
        } else {
            "[ ]"
        };
        let color = if is_unlocked {
            Color::Green
        } else {
            Color::DarkGrey
        };
        let line = format!("{prefix} {desc}");
        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
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
        "Arrow Keys / WASD: Move Snake",
        "P: Pause / Resume",
        "S: Save Game (in Pause)",
        "Space/Enter: Load Game (in Menu)",
        "Q: Quit / Back to Menu",
        "Space/Enter: Select/Start",
        "Space/Enter: Shoot Laser (P1/P2 in-game)",
        "T: Toggle Bot (Autopilot)",
        "Z: Rewind Time",
    ];
    let skin_line = format!("{} : Snake Body", game.skin);
    let legend = [
        "Symbols:",
        skin_line.as_str(),
        "● : Food (+1 Score)",
        "★ : Bonus Food (+5 Score)",
        "X : Obstacle (Avoid!)",
        "♥ : Extra Life",
        "W : Ghost (Pass Walls)",
        "S : Shrink",
        "B : Bomb (Clear Obstacles)",
        "$ : 2x Score",
        "T : Teleport",
        "P : Power-Up",
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
        // String ownership issue with format!, so we reconstruct or handle differently
        // if needed. legend array constructed above creates temporaries.
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

fn draw_nft_shop<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "NFT SHOP";
    let title_len = u16::try_from(title.len()).unwrap_or(0);

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout
        .queue(cursor::MoveTo((game.width / 2).saturating_sub(title_len / 2), game.height / 4))?;
    write!(stdout, "{title}")?;

    let balance_msg = format!("Coins: {}", game.stats.coins);
    let balance_len = u16::try_from(balance_msg.len()).unwrap_or(0);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(balance_len / 2),
        game.height / 4 + 2,
    ))?;
    write!(stdout, "{balance_msg}")?;

    for (i, &(item, price)) in crate::game::AVAILABLE_ITEMS.iter().enumerate() {
        let (is_unlocked, item_msg) = match item {
            crate::game::ShopItem::Skin(skin) => {
                let unlocked = game.stats.unlocked_skins.contains(&skin);
                let msg = if unlocked {
                    format!("Skin '{skin}': Owned")
                } else {
                    format!("Skin '{skin}': {price}c")
                };
                (unlocked, msg)
            },
            crate::game::ShopItem::Theme(theme) => {
                let unlocked = game.stats.unlocked_themes.contains(&theme);
                let theme_name = format!("{theme:?}");
                let msg = if unlocked {
                    format!("Theme '{theme_name}': Owned")
                } else {
                    format!("Theme '{theme_name}': {price}c")
                };
                (unlocked, msg)
            },
        };

        let y_pos = game.height / 2 - 3 + u16::try_from(i).unwrap_or(0);

        if i == game.nft_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(item_msg.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
                y_pos,
            ))?;
            write!(stdout, "> {item_msg} <")?;
        } else {
            if is_unlocked {
                stdout.queue(SetForegroundColor(Color::Green))?;
            } else if game.stats.coins >= price {
                stdout.queue(SetForegroundColor(Color::White))?;
            } else {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            }
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(item_msg.len()).unwrap_or(0) / 2),
                y_pos,
            ))?;
            write!(stdout, "{item_msg}")?;
        }
    }

    let help_msg = "Use UP/DOWN to select, ENTER to buy, Q to go back";
    let help_len = u16::try_from(help_msg.len()).unwrap_or(0);
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(help_len / 2), game.height - 2))?;
    write!(stdout, "{help_msg}")?;

    Ok(())
}

fn draw_settings<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SETTINGS";
    let title_len = u16::try_from(title.len()).unwrap_or(0);

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout
        .queue(cursor::MoveTo((game.width / 2).saturating_sub(title_len / 2), game.height / 4))?;
    write!(stdout, "{title}")?;

    let settings_items = [
        format!("Difficulty: {:?}", game.difficulty),
        format!("Theme: {:?}", game.theme),
        format!(
            "Wrap Mode: {}",
            if game.wrap_mode {
                "On"
            } else {
                "Off"
            }
        ),
        format!("Skin: {}", game.skin),
    ];

    for (i, item) in settings_items.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(item.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
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
    stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(help_len / 2), game.height - 2))?;
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

fn draw_game<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let (border_color, food_color, snake_color, obs_color) = match game.theme {
        crate::game::Theme::Dark => {
            (Color::DarkGrey, Color::DarkRed, Color::Green, Color::DarkMagenta)
        },
        crate::game::Theme::Retro => (Color::Green, Color::Green, Color::Green, Color::Green),
        crate::game::Theme::Neon => (Color::Cyan, Color::Magenta, Color::Yellow, Color::Red),
        crate::game::Theme::Classic => (Color::Blue, Color::Red, Color::DarkGreen, Color::Magenta),
        crate::game::Theme::Ocean => (Color::DarkBlue, Color::Yellow, Color::Cyan, Color::White),
        crate::game::Theme::Matrix => {
            (Color::DarkGreen, Color::Green, Color::Green, Color::DarkGreen)
        },
        crate::game::Theme::Galactic => {
            let elapsed = usize::try_from(game.start_time.elapsed().as_secs()).unwrap_or(0);
            let food_c = if elapsed % 2 == 0 {
                Color::White
            } else {
                Color::Yellow
            };
            (Color::DarkBlue, food_c, Color::Cyan, Color::Magenta)
        },
        crate::game::Theme::Premium => (Color::Yellow, Color::Green, Color::Cyan, Color::Red),
        crate::game::Theme::Hacker => {
            (Color::Green, Color::DarkGreen, Color::Green, Color::DarkGrey)
        },
        crate::game::Theme::Cyberpunk => (Color::Magenta, Color::Cyan, Color::Yellow, Color::Red),
        crate::game::Theme::Rainbow => {
            let elapsed = usize::try_from(game.start_time.elapsed().as_secs()).unwrap_or(0);
            let colors =
                [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::Blue, Color::Magenta];
            let border_c = colors[elapsed % colors.len()];
            let food_c = colors[(elapsed + 1) % colors.len()];
            let snake_c = colors[(elapsed + 2) % colors.len()];
            let obs_c = colors[(elapsed + 3) % colors.len()];
            (border_c, food_c, snake_c, obs_c)
        },
        crate::game::Theme::Blockchain => {
            (Color::DarkYellow, Color::Yellow, Color::DarkGrey, Color::DarkCyan)
        },
        crate::game::Theme::Esports => (Color::Red, Color::Blue, Color::Cyan, Color::Magenta),
        crate::game::Theme::Solar => (Color::Yellow, Color::Red, Color::DarkYellow, Color::DarkRed),
        crate::game::Theme::Metaverse => (Color::Magenta, Color::Cyan, Color::White, Color::DarkMagenta),
    };

    draw_background(game, stdout)?;
    draw_borders(game, stdout, border_color)?;
    draw_entities(game, stdout, food_color, snake_color, obs_color)?;
    draw_status(game, stdout)?;
    draw_overlays(game, stdout)?;
    draw_chat(game, stdout)?;

    Ok(())
}

fn draw_background<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let elapsed = usize::try_from(game.start_time.elapsed().as_millis() / 100).unwrap_or(0);
    let margin = if game.mode == crate::game::GameMode::BattleRoyale {
        game.safe_zone_margin
    } else {
        0
    };

    let min_x = margin + 1;
    let max_x = (game.width - 1).saturating_sub(margin).max(min_x);
    let min_y = margin + 1;
    let max_y = (game.height - 1).saturating_sub(margin).max(min_y);

    if max_x <= min_x || max_y <= min_y {
        return Ok(());
    }

    match game.theme {
        crate::game::Theme::Matrix => {
            stdout.queue(SetForegroundColor(Color::DarkGreen))?;
            for y in min_y..max_y {
                for x in min_x..max_x {
                    // Simple deterministic pseudo-random logic
                    let noise = (x as usize * 17 + y as usize * 31 + elapsed) % 100;
                    if noise < 5 {
                        let c = u8::try_from(33 + ((x as usize * y as usize + elapsed) % 94))
                            .unwrap_or(33) as char;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "{c}")?;
                    }
                }
            }
        },
        crate::game::Theme::Galactic => {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let noise = (x as usize * 73 + y as usize * 11 + elapsed / 5) % 200;
                    if noise < 2 {
                        stdout.queue(SetForegroundColor(Color::White))?;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, ".")?;
                    } else if noise == 2 {
                        stdout.queue(SetForegroundColor(Color::Yellow))?;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "*")?;
                    }
                }
            }
        },
        crate::game::Theme::Metaverse => {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let val = (x as usize * 29 + y as usize * 37 + elapsed) % 150;
                    if val < 2 {
                        stdout.queue(SetForegroundColor(Color::Magenta))?;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "✦")?;
                    } else if val < 4 {
                        stdout.queue(SetForegroundColor(Color::Cyan))?;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "✧")?;
                    } else if val == 4 {
                        stdout.queue(SetForegroundColor(Color::White))?;
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "∘")?;
                    }
                }
            }
        },
        crate::game::Theme::Ocean => {
            stdout.queue(SetForegroundColor(Color::Blue))?;
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let wave = (x as usize + elapsed / 2) % 20;
                    #[expect(
                        clippy::manual_is_multiple_of,
                        reason = "Using multiple_of requires unstable feature"
                    )]
                    if y as usize % 2 == 0 && wave < 3 {
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "~")?;
                    }
                }
            }
        },
        _ => {},
    }

    Ok(())
}

fn draw_chat<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    if let Ok((term_width, _term_height)) = crossterm::terminal::size() {
        let required_width = game.width + 30; // Need at least 30 cols for chat
        if term_width >= required_width {
            let chat_start_x = game.width + 2;
            let chat_width = term_width.saturating_sub(chat_start_x).saturating_sub(1);

            if chat_width >= 10 {
                // Draw chat border/title
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(cursor::MoveTo(chat_start_x, 1))?;
                write!(stdout, "=== LIVE CHAT ===")?;

                // Draw separator line
                for y in 0..game.height {
                    stdout.queue(cursor::MoveTo(game.width, y))?;
                    write!(stdout, "│")?;
                }

                // Draw chat messages
                let start_y = 3;
                for (i, (msg, color)) in game.chat_log.iter().enumerate() {
                    let y = start_y + u16::try_from(i).unwrap_or(0);
                    if y < game.height {
                        stdout.queue(SetForegroundColor((*color).into()))?;
                        stdout.queue(cursor::MoveTo(chat_start_x, y))?;
                        // Truncate message if it's too long for the chat area
                        let display_msg = if msg.len() > usize::from(chat_width) {
                            &msg[..usize::from(chat_width)]
                        } else {
                            msg
                        };
                        write!(stdout, "{display_msg}")?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn draw_level_editor<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    draw_borders(game, stdout, Color::Cyan)?;

    // Draw existing obstacles
    stdout.queue(SetForegroundColor(Color::Red))?;
    for obs in &game.obstacles {
        stdout.queue(cursor::MoveTo(obs.x, obs.y))?;
        write!(stdout, "X")?;
    }

    // Draw cursor
    if let Some(cursor) = game.editor_cursor {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(cursor.x, cursor.y))?;
        write!(stdout, "+")?;
    }

    // Draw instructions at the bottom
    let msg = "WASD/Arrows: Move | Space: Toggle | Q/Esc: Save & Exit";
    let msg_len = u16::try_from(msg.len()).unwrap_or(0);
    let x_pos = (game.width / 2).saturating_sub(msg_len / 2);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(x_pos, game.height))?;
    write!(stdout, "{msg}")?;

    Ok(())
}

fn draw_borders<W: Write>(game: &Game, stdout: &mut W, border_color: Color) -> io::Result<()> {
    let margin = if game.mode == crate::game::GameMode::BattleRoyale {
        game.safe_zone_margin
    } else {
        0
    };

    if margin > 0 {
        stdout.queue(SetForegroundColor(Color::Red))?;
        for y in 0..game.height {
            for x in 0..game.width {
                if x < margin || x >= game.width - margin || y < margin || y >= game.height - margin
                {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "▒")?;
                }
            }
        }
    }

    if game.just_died {
        stdout.queue(SetForegroundColor(Color::Red))?;
    } else {
        stdout.queue(SetForegroundColor(border_color))?;
    }

    let min_x = margin;
    let max_x = (game.width - 1).saturating_sub(margin).max(min_x);
    let min_y = margin;
    let max_y = (game.height - 1).saturating_sub(margin).max(min_y);

    if max_x > min_x && max_y > min_y {
        stdout.queue(cursor::MoveTo(min_x, min_y))?;
        let mut top_border = String::from("╔");
        top_border.push_str(&"═".repeat(usize::from(max_x - min_x).saturating_sub(1)));
        top_border.push('╗');
        write!(stdout, "{top_border}")?;

        stdout.queue(cursor::MoveTo(min_x, max_y))?;
        let mut bottom_border = String::from("╚");
        bottom_border.push_str(&"═".repeat(usize::from(max_x - min_x).saturating_sub(1)));
        bottom_border.push('╝');
        write!(stdout, "{bottom_border}")?;

        for y in min_y + 1..max_y {
            stdout.queue(cursor::MoveTo(min_x, y))?;
            write!(stdout, "║")?;
            stdout.queue(cursor::MoveTo(max_x, y))?;
            write!(stdout, "║")?;
        }
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "Drawing entities involves many distinct cases")]
fn draw_entities<W: Write>(
    game: &Game,
    stdout: &mut W,
    food_color: Color,
    snake_color: Color,
    obs_color: Color,
) -> io::Result<()> {
    // Draw particles
    for p in &game.particles {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Screen coords are within valid bounds"
        )]
        let px = p.x.round() as u16;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Screen coords are within valid bounds"
        )]
        let py = p.y.round() as u16;

        if px > 0 && px < game.width - 1 && py > 0 && py < game.height - 1 {
            // Fade effect: use DarkGrey when lifetime is low, otherwise base color
            let display_color = if p.lifetime < p.max_lifetime * 0.3 {
                Color::DarkGrey
            } else {
                p.color.into()
            };

            stdout.queue(cursor::MoveTo(px, py))?;
            stdout.queue(SetForegroundColor(display_color))?;
            write!(stdout, "{}", p.symbol)?;
        }
    }

    // Draw lasers
    for laser in &game.lasers {
        let symbol = match laser.direction {
            crate::snake::Direction::Up | crate::snake::Direction::Down => '|',
            crate::snake::Direction::Left | crate::snake::Direction::Right => '-',
        };
        let color = if laser.player == 1 {
            snake_color
        } else {
            Color::Blue
        };
        stdout.queue(cursor::MoveTo(laser.position.x, laser.position.y))?;
        stdout.queue(SetForegroundColor(color))?;
        write!(stdout, "{symbol}")?;
    }

    // Draw autopilot path
    if game.auto_pilot || game.mode == crate::game::GameMode::BotVsBot {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for path_point in &game.autopilot_path {
            stdout.queue(cursor::MoveTo(path_point.x, path_point.y))?;
            write!(stdout, "·")?;
        }
    }
    if game.mode == crate::game::GameMode::PlayerVsBot
        || game.mode == crate::game::GameMode::BotVsBot
    {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for path_point in &game.p2_autopilot_path {
            stdout.queue(cursor::MoveTo(path_point.x, path_point.y))?;
            write!(stdout, "·")?;
        }
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

    // Draw Portals
    if let Some((p1, p2)) = game.portals {
        stdout.queue(cursor::MoveTo(p1.x, p1.y))?;
        stdout.queue(SetForegroundColor(Color::Cyan))?;
        write!(stdout, "O")?;

        stdout.queue(cursor::MoveTo(p2.x, p2.y))?;
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "O")?;
    }

    // Draw Boss
    if let Some(boss) = &game.boss {
        stdout.queue(cursor::MoveTo(boss.position.x, boss.position.y))?;
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "B")?;
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
            crate::game::PowerUpType::ExtraLife => {
                stdout.queue(SetForegroundColor(Color::Magenta))?;
                write!(stdout, "♥")?;
            },
            crate::game::PowerUpType::PassThroughWalls => {
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                write!(stdout, "W")?;
            },
            crate::game::PowerUpType::Shrink => {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "S")?;
            },
            crate::game::PowerUpType::ClearObstacles => {
                stdout.queue(SetForegroundColor(Color::Red))?;
                write!(stdout, "B")?;
            },
            crate::game::PowerUpType::ScoreMultiplier => {
                stdout.queue(SetForegroundColor(Color::Green))?;
                write!(stdout, "$")?;
            },
            crate::game::PowerUpType::Teleport => {
                stdout.queue(SetForegroundColor(Color::Blue))?;
                write!(stdout, "T")?;
            },
            _ => {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "P")?;
            },
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

    // Draw player2
    if let Some(p2) = &game.player2 {
        stdout.queue(SetForegroundColor(Color::Blue))?;
        for (i, part) in p2.body.iter().enumerate() {
            stdout.queue(cursor::MoveTo(part.x, part.y))?;
            if i == 0 {
                // Head
                let head_char = match p2.direction {
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
    }

    Ok(())
}

fn draw_base_status<W: Write>(game: &Game, stdout: &mut W, bot_str: &str, combo_str: &str) -> io::Result<()> {
    if game.mode == crate::game::GameMode::Campaign {
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Campaign Lvl: {} | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.campaign_level,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::BattleRoyale {
        let max_margin = (game.width.min(game.height) / 2).saturating_sub(2);
        let shrink_str = if game.safe_zone_margin < max_margin {
            let shrink_in = 10u64.saturating_sub(game.last_shrink_time.elapsed().as_secs());
            format!(" | Shrink in: {shrink_in}s")
        } else {
            " | MAX SHRINK".to_string()
        };
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | {:?}{}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.difficulty,
            bot_str,
            shrink_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::TimeAttack {
        let time_left = 60u64.saturating_sub(game.start_time.elapsed().as_secs());
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Time: {}s | {:?}{}{}",
            game.score, game.high_score, game.lives, time_left, game.difficulty, bot_str, combo_str
        )?;
    } else if game.mode == crate::game::GameMode::Speedrun {
        let elapsed = game.start_time.elapsed().as_secs();
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Time: {}s | Food: {}/50 | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            elapsed,
            game.food_eaten_session,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else {
        let level = game.score / 20 + 1;
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Level: {} | {:?}{}{}",
            game.score, game.high_score, game.lives, level, game.difficulty, bot_str, combo_str
        )?;
    }
    Ok(())
}

fn draw_powerup_status<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    if let Some(power_up) = &game.power_up
        && let Some(activation_time) = power_up.activation_time
    {
        let elapsed = web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(activation_time);
        if elapsed < 5 {
            let remaining = 5 - elapsed;
            let power_up_name = match power_up.p_type {
                crate::game::PowerUpType::SlowDown => "Slowdown",
                crate::game::PowerUpType::SpeedBoost => "Speed Boost",
                crate::game::PowerUpType::Invincibility => "Invincible",
                crate::game::PowerUpType::ExtraLife => "Extra Life",
                crate::game::PowerUpType::PassThroughWalls => "Ghost",
                crate::game::PowerUpType::Shrink => "Shrink",
                crate::game::PowerUpType::ClearObstacles => "Bomb",
                crate::game::PowerUpType::ScoreMultiplier => "2x Score",
                crate::game::PowerUpType::Teleport => "Teleport",
                crate::game::PowerUpType::Magnet => "Magnet",
            };
            let power_up_msg = format!(" | {power_up_name}: {remaining}s");
            write!(stdout, "{power_up_msg}")?;
        }
    }
    Ok(())
}

fn draw_status<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    stdout.queue(SetForegroundColor(Color::Reset))?;
    stdout.queue(cursor::MoveTo(0, game.height))?;
    let bot_str = if game.auto_pilot {
        " | [BOT MODE]"
    } else {
        ""
    };
    let combo_str =
        if game.combo > 1 && game.last_food_time.is_some_and(|t| t.elapsed().as_secs() < 5) {
            format!(" | Combo: {}x", game.combo)
        } else {
            String::new()
        };

    draw_base_status(game, stdout, bot_str, &combo_str)?;

    if let Some(boss) = &game.boss {
        let boss_msg = format!(" | Boss HP: {}/{}", boss.health, boss.max_health);
        write!(stdout, "{boss_msg}")?;
    }

    draw_powerup_status(game, stdout)?;

    Ok(())
}

fn draw_overlays<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
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
        expected_buf.queue(SetForegroundColor(Color::White)).expect("Valid operation in tests");
        expected_buf.queue(cursor::MoveTo(x, y)).expect("Valid operation in tests");
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
        game.menu_selection = 0; // "Single Player" selected

        let mut buf = Vec::new();
        draw_menu(&game, &mut buf).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");

        // Check title
        assert!(output.contains("SNAKE GAME"), "Menu should contain title");

        // Check selection indicator
        assert!(output.contains("> Single Player <"), "Menu should indicate selection");
        assert!(output.contains("Load Game"), "Menu should contain other items");
        assert!(output.contains("Statistics"), "Menu should contain Statistics item");
        assert!(!output.contains("> Load Game <"), "Unselected items should not have brackets");
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

        assert!(output.contains("HELP & CONTROLS"), "Help should contain title");
        assert!(output.contains("Arrow Keys / WASD: Move Snake"), "Help should contain controls");
        assert!(output.contains("O : Snake Body"), "Help should contain dynamic skin info");
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
        assert!(output.ends_with(&expected), "Expected output to end with drawing '3' at (10, 10)");

        // Test double digit (count = 10) to test centering subtraction
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 10).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        // msg.len() is 2, so 2/2 is 1. 10 - 1 = 9.
        let expected = get_expected_ansi_tail(9, 10, "10");
        assert!(output.ends_with(&expected), "Expected output to end with drawing '10' at (9, 10)");

        // Test count = 0
        let mut buf = Vec::new();
        draw_countdown(&game, &mut buf, 0).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        let expected = get_expected_ansi_tail(10, 10, "0");
        assert!(output.ends_with(&expected), "Expected output to end with drawing '0' at (10, 10)");

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
        assert!(output.ends_with(&expected), "Expected output to center correctly on large board");

        // Test large digit (count = 12345)
        let mut buf = Vec::new();
        draw_countdown(&large_game, &mut buf, 12345).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");
        // msg.len() is 5, so 5/2 is 2. 50 - 2 = 48.
        let expected = get_expected_ansi_tail(48, 50, "12345");
        assert!(output.ends_with(&expected), "Expected output to center large digits correctly");
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
        assert!(output.contains("Difficulty: Normal"), "Settings should show Difficulty");
        assert!(output.contains("> Theme: Dark <"), "Settings should indicate selected item");
        assert!(output.contains("Wrap Mode: Off"), "Settings should show Wrap Mode");
    }
}
