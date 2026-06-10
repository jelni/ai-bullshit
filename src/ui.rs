use std::io::{self, Write};

use crossterm::{
    QueueableCommand, cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    game::{Game, GameState, Weather},
    snake::Direction,
};
use rand::{Rng, SeedableRng};

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
        GameState::SkillTree => draw_skill_tree(game, stdout)?,
        GameState::LevelEditor => draw_level_editor(game, stdout)?,
        GameState::LevelUp => draw_level_up(game, stdout)?,
        GameState::Crafting => draw_crafting(game, stdout)?,
        GameState::BountyBoard => draw_bounty_board(game, stdout)?,
        GameState::MerchantShop => draw_merchant_shop(game, stdout)?,
        GameState::CompanionCamp => draw_companion_camp(game, stdout)?,
        GameState::ClassSelect => draw_class_select(game, stdout)?,
        GameState::Equipment => draw_equipment(game, stdout)?,
        GameState::Casino => draw_casino(game, stdout)?,
        GameState::StockMarket => draw_stock_market(game, stdout)?,
        GameState::RealEstate => draw_real_estate(game, stdout)?,
        GameState::VehicleGarage => draw_vehicle_garage(game, stdout)?,
        GameState::Fishing => draw_fishing(game, stdout)?,
        GameState::BattlePass => draw_battle_pass(game, stdout)?,
        GameState::ArtifactShrine => draw_artifact_shrine(game, stdout)?,
        GameState::Hatchery => draw_hatchery(game, stdout)?,
        GameState::SpacePort => draw_space_port(game, stdout)?,
        GameState::FactionBase => draw_faction_base(game, stdout)?,
        GameState::MagicAcademy => draw_magic_academy(game, stdout)?,
        GameState::QuestLog => draw_quest_log(game, stdout)?,
        GameState::Bestiary => draw_bestiary(game, stdout)?,
        GameState::Tavern => draw_tavern(game, stdout)?,
        GameState::BlackMarket => draw_black_market(game, stdout)?,
        GameState::Bank => draw_bank(game, stdout)?,
        GameState::AuctionHouse => draw_auction_house(game, stdout)?,
        GameState::Gacha => draw_gacha(game, stdout)?,
    }

    stdout.flush()?;
    Ok(())
}

fn draw_space_port<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SPACE PORT";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    let planets = [
        (crate::game::Planet::Earth, "Earth (Normal Gravity)"),
        (crate::game::Planet::Moon, "Moon (Low Gravity, Slower)"),
        (crate::game::Planet::Mars, "Mars (Sandstorms)"),
        (crate::game::Planet::Jupiter, "Jupiter (High Gravity, Faster)"),
    ];

    for (i, (planet, name)) in planets.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_planets.contains(planet);
        let color = if i == game.settings_selection {
            Color::Yellow
        } else if is_unlocked {
            Color::White
        } else {
            Color::DarkGrey
        };
        stdout.queue(SetForegroundColor(color))?;

        let text = if is_unlocked {
            if *planet == game.current_planet {
                format!("{name} [CURRENT]")
            } else {
                (*name).to_string()
            }
        } else {
            format!("{name} (Locked - 50 Coins)")
        };

        let prefix = if i == game.settings_selection {
            "> "
        } else {
            "  "
        };
        let suffix = if i == game.settings_selection {
            " <"
        } else {
            "  "
        };

        let display_text = format!("{prefix}{text}{suffix}");

        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{display_text}")?;
    }

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        game.height / 2 + 3,
    ))?;
    write!(stdout, "{coins_str}")?;

    let help_text = "Up/Down: Select | Enter: Travel/Unlock | Q: Back";
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help_text.len()).unwrap_or(0) / 2),
        game.height / 2 + 5,
    ))?;
    write!(stdout, "{help_text}")?;

    Ok(())
}

fn draw_magic_academy<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "MAGIC ACADEMY";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    let spells = [
        (crate::game::SpellType::Heal, "Heal (+1 Life, 50 Mana)"),
        (crate::game::SpellType::Blink, "Blink (Teleport 3 steps, 30 Mana)"),
        (crate::game::SpellType::Fireball, "Fireball (Shoot laser, 40 Mana)"),
        (crate::game::SpellType::Shield, "Shield (Invincibility, 60 Mana)"),
    ];

    for (i, (spell, desc)) in spells.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_spells.contains(spell);
        let is_equipped = game.stats.equipped_spell == Some(*spell);

        let status = if is_equipped {
            "[EQUIPPED]"
        } else if is_unlocked {
            "[OWNED]"
        } else {
            "[1000 COINS]"
        };

        let prefix = if i == game.settings_selection {
            ">"
        } else {
            " "
        };
        let suffix = if i == game.settings_selection {
            "<"
        } else {
            " "
        };
        let color = if is_equipped {
            Color::Green
        } else if is_unlocked {
            Color::White
        } else if game.stats.coins >= 1000 {
            Color::Yellow
        } else {
            Color::DarkGrey
        };

        let display_text = format!("{prefix} {desc} {status} {suffix}");
        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{display_text}")?;
    }

    // Unequip option
    let unequip_idx = 4;
    let prefix = if unequip_idx == game.settings_selection {
        ">"
    } else {
        " "
    };
    let suffix = if unequip_idx == game.settings_selection {
        "<"
    } else {
        " "
    };
    let color = if game.stats.equipped_spell.is_none() {
        Color::Green
    } else {
        Color::White
    };
    let display_text = format!("{prefix} Unequip Spell {suffix}");

    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
        game.height / 2 - 2 + u16::try_from(unequip_idx).unwrap_or(0) * 2,
    ))?;
    write!(stdout, "{display_text}")?;

    let help = "Up/Down: Select | Enter: Buy/Equip | Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_hatchery<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "PET HATCHERY";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = if let Some((egg_type, timer)) = &game.stats.incubator {
        format!("INCUBATING: {egg_type:?} EGG - {timer} TICKS REMAINING")
    } else {
        "INCUBATOR IS EMPTY".to_string()
    };

    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    let common = game.stats.inventory_eggs.get(&crate::game::EggType::Common).copied().unwrap_or(0);
    let rare = game.stats.inventory_eggs.get(&crate::game::EggType::Rare).copied().unwrap_or(0);
    let legendary =
        game.stats.inventory_eggs.get(&crate::game::EggType::Legendary).copied().unwrap_or(0);

    let items = [
        format!("Incubate Common Egg [Owned: {common}]"),
        format!("Incubate Rare Egg [Owned: {rare}]"),
        format!("Incubate Legendary Egg [Owned: {legendary}]"),
    ];

    for (i, item) in items.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Green))?;
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

    let help = "Up/Down: Select | Space/Enter: Incubate | Q/Esc: Leave";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_vehicle_garage<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "🏎️ VEHICLE GARAGE 🚀";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let vehicles = [
        crate::game::Vehicle::Bike,
        crate::game::Vehicle::Car,
        crate::game::Vehicle::Tank,
        crate::game::Vehicle::Spaceship,
    ];

    let names = [
        "Bike (Speed boost)",
        "Car (Drop obstacles)",
        "Tank (Destroy obstacles)",
        "Spaceship (Pass through walls)",
    ];

    let costs = [1000, 2500, 5000, 10000];

    for (i, v) in vehicles.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_vehicles.contains(v);
        let is_equipped = game.stats.equipped_vehicle == Some(*v);

        let status = if is_equipped {
            "[EQUIPPED]".to_string()
        } else if is_unlocked {
            "[UNLOCKED]".to_string()
        } else {
            format!("[Cost: {}]", costs[i])
        };

        let prefix = if i == game.settings_selection {
            ">"
        } else {
            " "
        };
        let suffix = if i == game.settings_selection {
            "<"
        } else {
            " "
        };
        let color = if is_equipped {
            Color::Green
        } else if is_unlocked {
            Color::White
        } else if game.stats.coins >= costs[i] {
            Color::Yellow
        } else {
            Color::DarkGrey
        };

        let item_str = format!("{prefix} {} {} {suffix}", names[i], status);
        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(item_str.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i * 2).unwrap_or(0),
        ))?;
        write!(stdout, "{item_str}")?;
    }

    // Unequip option
    let unequip_idx = 4;
    let prefix = if unequip_idx == game.settings_selection {
        ">"
    } else {
        " "
    };
    let suffix = if unequip_idx == game.settings_selection {
        "<"
    } else {
        " "
    };
    let color = if game.stats.equipped_vehicle.is_none() {
        Color::Green
    } else {
        Color::White
    };
    let item_str = format!("{prefix} Unequip Vehicle {suffix}");

    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(item_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 2 + u16::try_from(unequip_idx * 2).unwrap_or(0),
    ))?;
    write!(stdout, "{item_str}")?;

    let help_text = "Space/Enter: Select | Esc/Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help_text.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help_text}")?;

    Ok(())
}

fn draw_stock_market<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "📈 STOCK MARKET 📉";
    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let stocks = [
        crate::game::Stock::SnakeCorp,
        crate::game::Stock::GoblinInc,
        crate::game::Stock::BossDynamics,
        crate::game::Stock::LaserTech,
    ];

    let stock_names = ["SnakeCorp", "GoblinInc", "BossDynamics", "LaserTech"];

    for (i, stock) in stocks.iter().enumerate() {
        let price = game.stats.stock_prices.get(stock).copied().unwrap_or(100);
        let owned = game.stats.portfolio.get(stock).copied().unwrap_or(0);

        let prefix = if i == game.settings_selection {
            ">"
        } else {
            " "
        };
        let color = if i == game.settings_selection {
            Color::Cyan
        } else {
            Color::White
        };

        let text = format!("{} {}: {} coins (Owned: {})", prefix, stock_names[i], price, owned);
        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(text.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{text}")?;
    }

    let help_text = "Space/Enter: Buy (1) | B: Buy (10) | L: Sell (1) | D: Sell (10) | Esc/Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help_text.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help_text}")?;

    Ok(())
}

fn draw_casino<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "🎰 CASINO 🎰";
    stdout.queue(SetForegroundColor(Color::Magenta))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let options = ["1. Slot Machine (100 coins)", "2. Roulette [Red/Black] (50 coins)"];
    for (i, opt) in options.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(opt.len() + 3).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, ">> {opt}")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(opt.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0),
            ))?;
            write!(stdout, "{opt}")?;
        }
    }

    let back = "Press 'Q' or 'ESC' to return to menu";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(back.len()).unwrap_or(0) / 2),
        game.height / 2 + 4,
    ))?;
    write!(stdout, "{back}")?;

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

const fn get_elo_rank(elo: u32) -> &'static str {
    if elo < 1200 {
        "Bronze"
    } else if elo < 1400 {
        "Silver"
    } else if elo < 1600 {
        "Gold"
    } else if elo < 1800 {
        "Platinum"
    } else if elo < 2000 {
        "Diamond"
    } else {
        "Grandmaster"
    }
}

#[expect(clippy::too_many_lines, reason = "Menu naturally has many entries to draw")]
fn draw_menu<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SNAKE GAME";

    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    let rank_str =
        format!("Rank: {} ({})", get_elo_rank(game.stats.player_elo), game.stats.player_elo);
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(rank_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{rank_str}")?;

    let menu_items = [
        "Single Player",
        "Daily Challenge",
        "Weekly Challenge",
        "Monthly Challenge",
        "Yearly Challenge",
        "Decade Challenge",
        "Century Challenge",
        "Millennium Challenge",
        "Campaign Mode",
        "Local Multiplayer",
        "Online Multiplayer",
        "Tournament",
        "Player vs Bot",
        "Bot vs Bot",
        "Battle Royale",
        "Time Attack",
        "Survival Mode",
        "Zen Mode",
        "Maze Mode",
        "Cave Mode",
        "Dungeon Mode",
        "Speedrun Mode",
        "Fog Of War Mode",
        "Evolution Mode",
        "Boss Rush Mode",
        "Massive Multiplayer",
        "Mirror Mode",
        "Flood Mode",
        "Vampire Mode",
        "Gravity Mode",
        "Tron Mode",
        "Zombie Mode",
        "Farmstead Mode",
        "PacMan Mode",
        "Capture The Flag Mode",
        "Bullet Hell Mode",
        "Snake Survivor Mode",
        "King Of The Hill Mode",
        "Dodgeball Mode",
        "Load Game",
        "Settings",
        "NFT Shop",
        "Upgrades",
        "Statistics",
        "Achievements",
        "Help",
        "Play Custom Level",
        "Level Editor",
        "Crafting",
        "Bounty Board",
        "Companion Camp",
        "Class Select",
        "Equipment",
        "Casino",
        "Stock Market",
        "Real Estate Office",
        "Vehicle Garage",
        "Fishing Pond",
        "Battle Pass",
        "Artifact Shrine",
        "Pet Hatchery",
        "Space Port",
        "Faction Base",
        "Magic Academy",
        "Quest Log",
        "Bestiary",
        "Tavern",
        "Black Market",
        "Bank",
        "Auction House",
        "Gacha",
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
        (crate::game::Achievement::BossSlayer, "Boss Slayer (Defeat a Boss)"),
        (
            crate::game::Achievement::MassiveMultiplayerEnthusiast,
            "MMO Enthusiast (Play Massive Multiplayer)",
        ),
        (crate::game::Achievement::PoisonEater, "Poison Eater (Eat Poison Food)"),
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
        format!(
            "Competitive Rank: {} ({})",
            get_elo_rank(game.stats.player_elo),
            game.stats.player_elo
        ),
        format!("Bot ELO: {}", game.stats.bot_elo),
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
                let passive = match skin {
                    '💎' => " [+1 Life]",
                    '👾' => " [+5 Lasers]",
                    '🐍' => " [+Speed]",
                    '🚀' => " [Start w/ SpeedBoost]",
                    '🦍' => " [Wall Smasher]",
                    '₿' => " [2x Coins]",
                    'Ξ' => " [+Portals]",
                    'Ð' => " [+Bonus Food]",
                    _ => "",
                };
                let msg = if unlocked {
                    format!("Skin '{skin}'{passive}: Owned")
                } else {
                    format!("Skin '{skin}'{passive}: {price}c")
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

fn draw_skill_tree<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "SKILL TREE UPGRADES";
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

    let upgrades = [
        (
            "Power-Up Duration",
            game.stats.upgrade_powerup_duration,
            500 * (1 + u32::from(game.stats.upgrade_powerup_duration)),
        ),
        (
            "Extra Lives",
            game.stats.upgrade_extra_lives,
            1000 * (1 + u32::from(game.stats.upgrade_extra_lives)),
        ),
        (
            "Laser Capacity",
            game.stats.upgrade_laser_capacity,
            1500 * (1 + u32::from(game.stats.upgrade_laser_capacity)),
        ),
        (
            "Coin Multiplier",
            game.stats.upgrade_coin_multiplier,
            2000 * (1 + u32::from(game.stats.upgrade_coin_multiplier)),
        ),
    ];

    for (i, (name, level, cost)) in upgrades.iter().enumerate() {
        let msg = if *level >= 10 {
            format!("{name} [MAXED]")
        } else {
            format!("{name} [Lvl {level}]: {cost}c")
        };

        let y_pos = game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2;

        if i == game.skill_tree_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
                y_pos,
            ))?;
            write!(stdout, "> {msg} <")?;
        } else {
            if *level >= 10 {
                stdout.queue(SetForegroundColor(Color::Green))?;
            } else if game.stats.coins >= *cost {
                stdout.queue(SetForegroundColor(Color::White))?;
            } else {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            }
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2),
                y_pos,
            ))?;
            write!(stdout, "{msg}")?;
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
        crate::game::Theme::Metaverse => {
            (Color::Magenta, Color::Cyan, Color::White, Color::DarkMagenta)
        },
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

fn draw_level_up<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    draw_game(game, stdout)?;

    let title = "LEVEL UP!";
    let title_len = u16::try_from(title.len()).unwrap_or(0);

    // Dim the screen a bit by putting a background color over the game,
    // or we can just draw a box.
    let popup_width = 50;
    let popup_height = 12;
    let _popup_x = (game.width / 2).saturating_sub(popup_width / 2);
    let popup_y = (game.height / 2).saturating_sub(popup_height / 2);

    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(title_len / 2), popup_y + 1))?;
    write!(stdout, "{title}")?;

    let subtitle = "Choose an upgrade:";
    let subtitle_len = u16::try_from(subtitle.len()).unwrap_or(0);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(subtitle_len / 2), popup_y + 3))?;
    write!(stdout, "{subtitle}")?;

    for (i, upgrade) in game.level_up_options.iter().enumerate() {
        let name = upgrade.name();
        let desc = upgrade.description();
        let display_text = format!("{name}: {desc}");
        let text_len = u16::try_from(display_text.len()).unwrap_or(0);
        let y_pos = popup_y + 5 + u16::try_from(i).unwrap_or(0) * 2;

        if i == game.level_up_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(text_len / 2).saturating_sub(2),
                y_pos,
            ))?;
            write!(stdout, "> {display_text} <")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo((game.width / 2).saturating_sub(text_len / 2), y_pos))?;
            write!(stdout, "{display_text}")?;
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
    let is_visible = |px: u16, py: u16| -> bool {
        if game.mode == crate::game::GameMode::FogOfWar
            || game.time_of_day == crate::game::TimeOfDay::Night
        {
            let head = game.snake.head();
            let dx = f32::from(px) - f32::from(head.x);
            let dy = f32::from(py) - f32::from(head.y);
            let mut visible = f32::hypot(dx, dy) <= 6.0;

            if !visible
                && game.player2.as_ref().is_some_and(|p2| {
                    let head2 = p2.head();
                    f32::hypot(
                        f32::from(px) - f32::from(head2.x),
                        f32::from(py) - f32::from(head2.y),
                    ) <= 6.0
                })
            {
                visible = true;
            }

            if !visible {
                for bot in &game.bots {
                    let head_b = bot.head();
                    if f32::hypot(
                        f32::from(px) - f32::from(head_b.x),
                        f32::from(py) - f32::from(head_b.y),
                    ) <= 6.0
                    {
                        visible = true;
                        break;
                    }
                }
            }

            visible
        } else {
            true
        }
    };

    // Draw floating texts
    for t in &game.floating_texts {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Screen coords are within valid bounds"
        )]
        let px = t.x.round() as u16;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Screen coords are within valid bounds"
        )]
        let py = t.y.round() as u16;

        if px > 0 && px < game.width - 1 && py > 0 && py < game.height - 1 && is_visible(px, py) {
            stdout.queue(cursor::MoveTo(px, py))?;
            stdout.queue(SetForegroundColor(t.color.into()))?;
            write!(stdout, "{} ", t.text)?;
        }
    }

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

        if px > 0 && px < game.width - 1 && py > 0 && py < game.height - 1 && is_visible(px, py) {
            // Fade effect: use DarkGrey when lifetime is low, otherwise base color
            let display_color = if p.lifetime < p.max_lifetime * 0.3 {
                crate::color::Color::DarkGrey
            } else {
                p.color
            };

            stdout.queue(cursor::MoveTo(px, py))?;
            stdout.queue(SetForegroundColor(display_color.into()))?;
            write!(stdout, "{}", p.symbol)?;
        }
    }

    // Draw Weather Effects
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        #[expect(
            clippy::cast_possible_truncation,
            reason = "We only need lower bits for a deterministic PRNG seed"
        )]
        {
            web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        },
    );
    let margin = if game.mode == crate::game::GameMode::BattleRoyale {
        game.safe_zone_margin
    } else {
        0
    };
    match game.weather {
        Weather::Rain => {
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            for _ in 0..15 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(x, y) {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "|")?;
                }
            }
        },
        Weather::Snow => {
            stdout.queue(SetForegroundColor(Color::White))?;
            for _ in 0..10 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(x, y) {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "*")?;
                }
            }
        },
        Weather::Tornado => {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            for _ in 0..15 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(x, y) {
                    let chars = ['@', 'S', '~', '°'];
                    let c = chars[rng.gen_range(0..chars.len())];
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "{c}")?;
                }
            }
        },
        _ => {},
    }

    // Draw Lightning Strike
    if let Some(col) = game.lightning_column {
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        for y in margin + 1..game.height.saturating_sub(margin).saturating_sub(1) {
            if is_visible(col, y) {
                stdout.queue(cursor::MoveTo(col, y))?;
                write!(stdout, "|")?;
            }
        }
    }

    // Draw lasers
    for laser in &game.lasers {
        if is_visible(laser.position.x, laser.position.y) {
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
    }

    // Draw equipment boxes
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    for ebox in &game.equipment_boxes {
        if is_visible(ebox.x, ebox.y) {
            stdout.queue(cursor::MoveTo(ebox.x, ebox.y))?;
            write!(stdout, "E")?;
        }
    }

    // Draw xp_gems
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    for gem in &game.xp_gems {
        if is_visible(gem.x, gem.y) {
            stdout.queue(cursor::MoveTo(gem.x, gem.y))?;
            write!(stdout, "♦")?;
        }
    }

    // Draw autopilot path
    if game.auto_pilot || game.mode == crate::game::GameMode::BotVsBot {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for path_point in &game.autopilot_path {
            if is_visible(path_point.x, path_point.y) {
                stdout.queue(cursor::MoveTo(path_point.x, path_point.y))?;
                write!(stdout, "·")?;
            }
        }
    }
    if game.mode == crate::game::GameMode::PlayerVsBot
        || game.mode == crate::game::GameMode::BotVsBot
    {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for path_point in &game.p2_autopilot_path {
            if is_visible(path_point.x, path_point.y) {
                stdout.queue(cursor::MoveTo(path_point.x, path_point.y))?;
                write!(stdout, "·")?;
            }
        }
    }

    // Draw resources
    for (pos, res) in &game.resources {
        if is_visible(pos.x, pos.y) {
            let (color, symbol) = match res {
                crate::game::Resource::Wood => (Color::Yellow, "🪵"),
                crate::game::Resource::Iron => (Color::White, "🔗"),
                crate::game::Resource::Gold => (Color::Yellow, "💰"),
                crate::game::Resource::Diamond => (Color::Cyan, "💎"),
            };
            stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
            stdout.queue(SetForegroundColor(color))?;
            write!(stdout, "{symbol}")?;
        }
    }

    // Draw eggs
    for (pos, egg) in &game.eggs_on_board {
        if is_visible(pos.x, pos.y) {
            let color = match egg {
                crate::game::EggType::Common => Color::White,
                crate::game::EggType::Rare => Color::Cyan,
                crate::game::EggType::Legendary => Color::Yellow,
            };
            stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
            stdout.queue(SetForegroundColor(color))?;
            write!(stdout, "🥚")?;
        }
    }

    // Draw food
    if is_visible(game.food.x, game.food.y) {
        stdout.queue(cursor::MoveTo(game.food.x, game.food.y))?;
        stdout.queue(SetForegroundColor(food_color))?;
        write!(stdout, "●")?;
    }

    // Draw resources
    for (pos, res) in &game.resources {
        if is_visible(pos.x, pos.y) {
            stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
            let (symbol, color) = match res {
                crate::game::Resource::Wood => ("🪵", Color::Yellow),
                crate::game::Resource::Iron => ("🔗", Color::White),
                crate::game::Resource::Gold => ("💰", Color::Yellow),
                crate::game::Resource::Diamond => ("💎", Color::Cyan),
            };
            stdout.queue(SetForegroundColor(color))?;
            write!(stdout, "{symbol}")?;
        }
    }

    // Draw obstacles
    stdout.queue(SetForegroundColor(obs_color))?;
    for obs in &game.obstacles {
        if is_visible(obs.x, obs.y) {
            stdout.queue(cursor::MoveTo(obs.x, obs.y))?;
            write!(stdout, "X")?;
        }

        for crop in &game.crops {
            stdout.queue(cursor::MoveTo(crop.position.x, crop.position.y))?;
            if crop.growth_stage == 0 {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, ".")?;
            } else if crop.growth_stage == 1 {
                stdout.queue(SetForegroundColor(Color::Green))?;
                write!(stdout, "v")?;
            } else {
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                write!(stdout, "¥")?;
            }
        }
    }

    // Draw Mines
    stdout.queue(SetForegroundColor(Color::Red))?;
    for mine in &game.mines {
        if is_visible(mine.x, mine.y) {
            stdout.queue(cursor::MoveTo(mine.x, mine.y))?;
            write!(stdout, "M")?;
        }
    }

    // Draw Turrets
    stdout.queue(SetForegroundColor(Color::DarkCyan))?;
    for turret in &game.turrets {
        if is_visible(turret.position.x, turret.position.y) {
            stdout.queue(cursor::MoveTo(turret.position.x, turret.position.y))?;
            write!(stdout, "T")?;
        }
    }

    // Draw Black Hole
    if let Some(bh) = game.black_hole
        && is_visible(bh.x, bh.y)
    {
        stdout.queue(cursor::MoveTo(bh.x, bh.y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        write!(stdout, "O")?;
    }

    // Draw Portals
    if let Some((p1, p2)) = game.portals {
        if is_visible(p1.x, p1.y) {
            stdout.queue(cursor::MoveTo(p1.x, p1.y))?;
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            write!(stdout, "O")?;
        }

        if is_visible(p2.x, p2.y) {
            stdout.queue(cursor::MoveTo(p2.x, p2.y))?;
            stdout.queue(SetForegroundColor(Color::Magenta))?;
            write!(stdout, "O")?;
        }
    }

    // Draw Decoy
    if let Some((decoy_pos, _)) = game.decoy
        && is_visible(decoy_pos.x, decoy_pos.y)
    {
        stdout.queue(cursor::MoveTo(decoy_pos.x, decoy_pos.y))?;
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "D")?;
    }

    // Draw Meteors
    for meteor in &game.meteors {
        if is_visible(meteor.position.x, meteor.position.y) {
            stdout.queue(cursor::MoveTo(meteor.position.x, meteor.position.y))?;
            stdout.queue(SetForegroundColor(Color::DarkYellow))?;
            write!(stdout, "*")?;
        }
    }

    // Draw Goblin
    if game.goblin.is_some_and(|goblin| is_visible(goblin.position.x, goblin.position.y)) {
        let goblin = game.goblin.unwrap();
        stdout.queue(cursor::MoveTo(goblin.position.x, goblin.position.y))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "G")?;
    }

    // Draw Bosses
    for boss in &game.bosses {
        if is_visible(boss.position.x, boss.position.y) {
            stdout.queue(cursor::MoveTo(boss.position.x, boss.position.y))?;
            match boss.kind {
                crate::game::BossType::Shooter => {
                    stdout.queue(SetForegroundColor(Color::Magenta))?;
                    write!(stdout, "B")?;
                },
                crate::game::BossType::Charger => {
                    stdout.queue(SetForegroundColor(Color::Red))?;
                    write!(stdout, "C")?;
                },
                crate::game::BossType::Spawner => {
                    stdout.queue(SetForegroundColor(Color::DarkGreen))?;
                    write!(stdout, "S")?;
                },
                crate::game::BossType::Teleporter => {
                    stdout.queue(SetForegroundColor(Color::Cyan))?;
                    write!(stdout, "T")?;
                },
                crate::game::BossType::Splitter => {
                    stdout.queue(SetForegroundColor(Color::Yellow))?;
                    write!(stdout, "P")?;
                },
                crate::game::BossType::Trapper => {
                    stdout.queue(SetForegroundColor(Color::DarkGreen))?;
                    write!(stdout, "W")?;
                },
                crate::game::BossType::Necromancer => {
                    stdout.queue(SetForegroundColor(Color::DarkMagenta))?;
                    write!(stdout, "N")?;
                },
                crate::game::BossType::ShadowClone => {
                    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                    write!(stdout, "X")?;
                },
                crate::game::BossType::Puffer => {
                    stdout.queue(SetForegroundColor(Color::DarkYellow))?;
                    write!(stdout, "P")?;
                },
                crate::game::BossType::Juggernaut => {
                    stdout.queue(SetForegroundColor(Color::DarkRed))?;
                    write!(stdout, "J")?;
                },
                crate::game::BossType::Mimic => {
                    let target_pos = if let Some((decoy_pos, _)) = game.decoy {
                        decoy_pos
                    } else {
                        game.snake.head()
                    };
                    let dist_x = i32::from(target_pos.x).abs_diff(i32::from(boss.position.x));
                    let dist_y = i32::from(target_pos.y).abs_diff(i32::from(boss.position.y));
                    if dist_x <= 3 && dist_y <= 3 {
                        stdout.queue(SetForegroundColor(Color::DarkRed))?;
                        write!(stdout, "M")?;
                    } else {
                        stdout.queue(SetForegroundColor(Color::Yellow))?;
                        write!(stdout, "★")?;
                    }
                },
                crate::game::BossType::Dragon => {
                    stdout.queue(SetForegroundColor(Color::Red))?;
                    write!(stdout, "D")?;
                },
                crate::game::BossType::Mage => {
                    stdout.queue(SetForegroundColor(Color::Cyan))?;
                    write!(stdout, "M")?;
                },
                crate::game::BossType::Gorgon => {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                    write!(stdout, "G")?;
                },
            }
        }
    }

    // Draw merchant
    if let Some(merchant_p) = game.merchant
        && is_visible(merchant_p.x, merchant_p.y)
    {
        stdout.queue(cursor::MoveTo(merchant_p.x, merchant_p.y))?;
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "$")?;
    }

    // Draw flags for Capture The Flag mode
    if game.mode == crate::game::GameMode::CaptureTheFlag {
        if let Some(p1_flag) = game.p1_flag
            && is_visible(p1_flag.x, p1_flag.y)
        {
            stdout.queue(cursor::MoveTo(p1_flag.x, p1_flag.y))?;
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            write!(stdout, "F")?;
        }
        if let Some(p2_flag) = game.p2_flag
            && is_visible(p2_flag.x, p2_flag.y)
        {
            stdout.queue(cursor::MoveTo(p2_flag.x, p2_flag.y))?;
            stdout.queue(SetForegroundColor(Color::Red))?;
            write!(stdout, "F")?;
        }
    }

    // Draw King of the Hill zone
    if game.mode == crate::game::GameMode::KingOfTheHill {
        if let Some(koth_pos) = game.koth_zone {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            for y in koth_pos.y.saturating_sub(1)..=koth_pos.y.saturating_add(1) {
                for x in koth_pos.x.saturating_sub(1)..=koth_pos.x.saturating_add(1) {
                    if is_visible(x, y) {
                        stdout.queue(cursor::MoveTo(x, y))?;
                        write!(stdout, "▒")?;
                    }
                }
            }
        }
    }

    // Draw bonus food
    if let Some((bonus_p, _)) = game.bonus_food
        && is_visible(bonus_p.x, bonus_p.y)
    {
        stdout.queue(cursor::MoveTo(bonus_p.x, bonus_p.y))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "★")?;
    }

    // Draw poison food
    if let Some((poison_p, _)) = game.poison_food
        && is_visible(poison_p.x, poison_p.y)
    {
        stdout.queue(cursor::MoveTo(poison_p.x, poison_p.y))?;
        stdout.queue(SetForegroundColor(Color::DarkMagenta))?;
        write!(stdout, "X")?;
    }

    if let Some(power_up) = &game.power_up
        && power_up.activation_time.is_none()
        && is_visible(power_up.location.x, power_up.location.y)
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
            crate::game::PowerUpType::Reverse => {
                stdout.queue(SetForegroundColor(Color::White))?;
                write!(stdout, "R")?;
            },
            crate::game::PowerUpType::Emp => {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "E")?;
            },
            _ => {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "P")?;
            },
        }
    }

    // Draw companion
    if let Some(comp) = &game.companion
        && is_visible(comp.position.x, comp.position.y)
    {
        stdout.queue(cursor::MoveTo(comp.position.x, comp.position.y))?;
        let (color, symbol) = match comp.kind {
            crate::game::CompanionType::Collector => (Color::Yellow, 'C'),
            crate::game::CompanionType::Fighter => (Color::Red, 'F'),
            crate::game::CompanionType::Healer => (Color::Green, 'H'),
        };
        stdout.queue(SetForegroundColor(color))?;
        write!(stdout, "{symbol}")?;
    }

    // Draw ghost snake
    if let Some(ghost) = &game.ghost_snake {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for (i, part) in ghost.body.iter().enumerate() {
            if is_visible(part.x, part.y) {
                stdout.queue(cursor::MoveTo(part.x, part.y))?;
                if i == 0 {
                    let head_char = match ghost.direction {
                        Direction::Up => '^',
                        Direction::Down => 'v',
                        Direction::Left => '<',
                        Direction::Right => '>',
                    };
                    write!(stdout, "{head_char}")?;
                } else {
                    write!(stdout, "{}", game.skin)?;
                }
            }
        }
    }

    // Draw snake
    stdout.queue(SetForegroundColor(snake_color))?;
    for (i, part) in game.snake.body.iter().enumerate() {
        if is_visible(part.x, part.y) {
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
    }

    // Draw autopilot paths
    if game.auto_pilot || game.mode == crate::game::GameMode::BotVsBot || game.used_bot_this_session
    {
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for p in &game.autopilot_path {
            if is_visible(p.x, p.y) {
                stdout.queue(cursor::MoveTo(p.x, p.y))?;
                write!(stdout, "·")?;
            }
        }
    }

    if game.mode == crate::game::GameMode::PlayerVsBot
        || game.mode == crate::game::GameMode::BotVsBot
    {
        stdout.queue(SetForegroundColor(Color::DarkMagenta))?;
        for p in &game.p2_autopilot_path {
            if is_visible(p.x, p.y) {
                stdout.queue(cursor::MoveTo(p.x, p.y))?;
                write!(stdout, "·")?;
            }
        }
    }

    // Draw bots and their paths
    if game.mode == crate::game::GameMode::MassiveMultiplayer {
        for (i, bot) in game.bots.iter().enumerate() {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            for p in &game.bots_autopilot_paths[i] {
                if is_visible(p.x, p.y) {
                    stdout.queue(cursor::MoveTo(p.x, p.y))?;
                    write!(stdout, "·")?;
                }
            }

            stdout.queue(SetForegroundColor(Color::Cyan))?;
            for (j, part) in bot.body.iter().enumerate() {
                if is_visible(part.x, part.y) {
                    stdout.queue(cursor::MoveTo(part.x, part.y))?;
                    if j == 0 {
                        let head_char = match bot.direction {
                            crate::snake::Direction::Up => '^',
                            crate::snake::Direction::Down => 'v',
                            crate::snake::Direction::Left => '<',
                            crate::snake::Direction::Right => '>',
                        };
                        write!(stdout, "{head_char}")?;
                    } else {
                        write!(stdout, "B")?;
                    }
                }
            }
        }
    }

    // Draw player2
    if let Some(p2) = &game.player2 {
        stdout.queue(SetForegroundColor(Color::Blue))?;
        for (i, part) in p2.body.iter().enumerate() {
            if is_visible(part.x, part.y) {
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
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "Status rendering naturally has many elements")]
fn draw_base_status<W: Write>(
    game: &Game,
    stdout: &mut W,
    bot_str: &str,
    combo_str: &str,
) -> io::Result<()> {
    if game.mode == crate::game::GameMode::CaptureTheFlag || game.mode == crate::game::GameMode::KingOfTheHill {
        write!(
            stdout,
            "P1 Score: {} | P2 Score: {} | Mana: {}/{} | {:?}{}{}",
            game.p1_score,
            game.p2_score,
            game.mana,
            game.max_mana,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::Campaign {
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | Campaign Lvl: {} | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
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
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | {:?}{}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
            game.difficulty,
            bot_str,
            shrink_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::TimeAttack {
        let time_left = 60u64.saturating_sub(game.start_time.elapsed().as_secs());
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | Time: {}s | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
            time_left,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::Speedrun {
        let elapsed = game.start_time.elapsed().as_secs();
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | Time: {}s | Food: {}/50 | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
            elapsed,
            game.food_eaten_session,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else if game.mode == crate::game::GameMode::BossRush {
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | Boss Lvl: {} | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
            game.campaign_level,
            game.difficulty,
            bot_str,
            combo_str
        )?;
    } else {
        let level = game.score / 20 + 1;
        write!(
            stdout,
            "Score: {} | High: {} | Lives: {} | Mana: {}/{} | Lvl: {} | XP: {}/{} | Stage: {} | {:?}{}{}",
            game.score,
            game.high_score,
            game.lives,
            game.mana,
            game.max_mana,
            game.player_level,
            game.xp,
            game.xp_to_next_level,
            level,
            game.difficulty,
            bot_str,
            combo_str
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
        let duration = game.powerup_duration();
        if elapsed < duration {
            let remaining = duration - elapsed;
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
                crate::game::PowerUpType::TimeFreeze => "Time Freeze",
                crate::game::PowerUpType::Reverse => "Reverse",
                crate::game::PowerUpType::Decoy => "Decoy",
                crate::game::PowerUpType::Emp => "Emp",
                crate::game::PowerUpType::Nuke => "Nuke",
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
    let tod_str = match game.time_of_day {
        crate::game::TimeOfDay::Day => " | Day",
        crate::game::TimeOfDay::Night => " | Night",
    };
    let weather_str = match game.weather {
        crate::game::Weather::Clear => "",
        crate::game::Weather::Rain => " | Weather: Rain",
        crate::game::Weather::Snow => " | Weather: Snow",
        crate::game::Weather::Storm => " | Weather: Storm",
        crate::game::Weather::Tornado => " | Weather: Tornado",
        crate::game::Weather::Sandstorm => " | Weather: Sandstorm",
        crate::game::Weather::Earthquake => " | Weather: Earthquake",
    };
    let combo_str =
        if game.combo > 1 && game.last_food_time.is_some_and(|t| t.elapsed().as_secs() < 5) {
            format!(" | Combo: {}x", game.combo)
        } else {
            String::new()
        };

    draw_base_status(game, stdout, bot_str, &combo_str)?;

    write!(stdout, "{weather_str}{tod_str}")?;

    if !game.bosses.is_empty() {
        let total_health: u32 = game.bosses.iter().map(|b| b.health).sum();
        let total_max_health: u32 = game.bosses.iter().map(|b| b.max_health).sum();
        let boss_msg = format!(" | Bosses HP: {total_health}/{total_max_health}");
        write!(stdout, "{boss_msg}")?;
    }

    draw_powerup_status(game, stdout)?;

    let potions = game
        .stats
        .crafted_items
        .get(&crate::game::CraftableItem::SpeedPotion)
        .copied()
        .unwrap_or(0);
    let walls =
        game.stats.crafted_items.get(&crate::game::CraftableItem::IronWall).copied().unwrap_or(0);
    let apples = game
        .stats
        .crafted_items
        .get(&crate::game::CraftableItem::GoldenApple)
        .copied()
        .unwrap_or(0);
    let swords = game
        .stats
        .crafted_items
        .get(&crate::game::CraftableItem::DiamondSword)
        .copied()
        .unwrap_or(0);

    if potions > 0 || walls > 0 || apples > 0 || swords > 0 {
        stdout.queue(cursor::MoveTo(0, game.height + 1))?;
        stdout.queue(SetForegroundColor(Color::Cyan))?;
        write!(
            stdout,
            "Items: [1]Potion: {potions} | [2]Wall: {walls} | [3]Apple: {apples} | [4]Sword: {swords}"
        )?;
    }

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

fn draw_bounty_board<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "BOUNTY BOARD";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = format!("Completed Bounties: {}", game.stats.completed_bounties);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    if let Some(ref active) = game.stats.active_bounty {
        let active_str = format!(
            "Active Bounty: {:?} - Progress: {} / {}",
            active.b_type, active.progress, active.target
        );
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(active_str.len()).unwrap_or(0) / 2),
            game.height / 2,
        ))?;
        write!(stdout, "{active_str}")?;

        let reward_str = format!("Reward: {} Coins", active.reward_coins);
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(reward_str.len()).unwrap_or(0) / 2),
            game.height / 2 + 1,
        ))?;
        write!(stdout, "{reward_str}")?;

        let cancel_str = "Press Enter to Cancel Bounty";
        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(cancel_str.len()).unwrap_or(0) / 2),
            game.height / 2 + 3,
        ))?;
        write!(stdout, "{cancel_str}")?;
    } else {
        let bounties = [
            "Eat 50 Food [Reward: 500 Coins]",
            "Kill 3 Bosses [Reward: 1000 Coins]",
            "Survive 120 Seconds [Reward: 750 Coins]",
        ];

        for (i, text) in bounties.iter().enumerate() {
            if i == game.settings_selection {
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                stdout.queue(cursor::MoveTo(
                    (game.width / 2)
                        .saturating_sub(u16::try_from(text.len()).unwrap_or(0) / 2)
                        .saturating_sub(2),
                    game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
                ))?;
                write!(stdout, "> {text} <")?;
            } else {
                stdout.queue(SetForegroundColor(Color::White))?;
                stdout.queue(cursor::MoveTo(
                    (game.width / 2).saturating_sub(u16::try_from(text.len()).unwrap_or(0) / 2),
                    game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
                ))?;
                write!(stdout, "{text}")?;
            }
        }
    }

    Ok(())
}

fn draw_crafting<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "CRAFTING MENU";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let wood = game.stats.inventory.get(&crate::game::Resource::Wood).copied().unwrap_or(0);
    let iron = game.stats.inventory.get(&crate::game::Resource::Iron).copied().unwrap_or(0);
    let gold = game.stats.inventory.get(&crate::game::Resource::Gold).copied().unwrap_or(0);
    let diamond = game.stats.inventory.get(&crate::game::Resource::Diamond).copied().unwrap_or(0);

    let inv_str = format!("Inventory: 🪵 {wood} | 🔗 {iron} | 💰 {gold} | 💎 {diamond}");
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(inv_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{inv_str}")?;

    let recipes = [
        (
            format!(
                "Speed Potion [3 🪵 ] (Owned: {})",
                game.stats
                    .crafted_items
                    .get(&crate::game::CraftableItem::SpeedPotion)
                    .unwrap_or(&0)
            ),
            wood >= 3,
        ),
        (
            format!(
                "Iron Wall [3 🔗 ] (Owned: {})",
                game.stats.crafted_items.get(&crate::game::CraftableItem::IronWall).unwrap_or(&0)
            ),
            iron >= 3,
        ),
        (
            format!(
                "Golden Apple [5 💰 ] (Owned: {})",
                game.stats
                    .crafted_items
                    .get(&crate::game::CraftableItem::GoldenApple)
                    .unwrap_or(&0)
            ),
            gold >= 5,
        ),
        (
            format!(
                "Diamond Sword [1 💎 ] (Owned: {})",
                game.stats
                    .crafted_items
                    .get(&crate::game::CraftableItem::DiamondSword)
                    .unwrap_or(&0)
            ),
            diamond >= 1,
        ),
    ];

    for (i, (text, can_craft)) in recipes.iter().enumerate() {
        let color = if *can_craft {
            Color::Green
        } else {
            Color::DarkGrey
        };
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(text.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "> {text} <")?;
        } else {
            stdout.queue(SetForegroundColor(color))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(text.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{text}")?;
        }
    }

    let help = "Use Arrow Keys to select, SPACE to craft, Q to go back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_merchant_shop<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "MERCHANT SHOP";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = format!("Your Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    let items = [
        "Extra Life [Cost: 500]",
        "Diamond Sword [Cost: 1000]",
        "Speed Potion [Cost: 300]",
        "Iron Wall [Cost: 100]",
    ];

    for (i, item) in items.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Green))?;
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

    let help = "Up/Down: Select | Space/Enter: Buy | Q/Esc: Leave";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_companion_camp<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "COMPANION CAMP";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    let items = [
        (crate::game::CompanionType::Collector, "Collector [Collects resources/food]"),
        (crate::game::CompanionType::Fighter, "Fighter [Shoots lasers at bosses]"),
        (crate::game::CompanionType::Healer, "Healer [Drops Extra Lives periodically]"),
    ];

    for (i, (comp_type, desc)) in items.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_companions.contains(comp_type);
        let is_equipped = game.stats.equipped_companion == Some(*comp_type);

        let prefix = if is_equipped {
            "[EQUIPPED]"
        } else if is_unlocked {
            "[OWNED]"
        } else {
            "[Cost: 1000c]"
        };

        let display_text = format!("{prefix} {desc}");

        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2)
                    .saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2)
                    .saturating_sub(2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "> {display_text} <")?;
        } else {
            let color = if is_equipped {
                Color::Green
            } else if is_unlocked {
                Color::White
            } else {
                Color::DarkGrey
            };
            stdout.queue(SetForegroundColor(color))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{display_text}")?;
        }
    }

    let help = "Up/Down: Select | Enter: Buy/Equip | Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_class_select<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "CLASS SELECT";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let classes = [
        ("Warrior", crate::game::HeroClass::Warrior, "Extra Lives"),
        ("Mage", crate::game::HeroClass::Mage, "Start with Time Freeze"),
        ("Rogue", crate::game::HeroClass::Rogue, "Dodge Chance"),
        ("Paladin", crate::game::HeroClass::Paladin, "Regenerate Lives"),
        ("Necromancer", crate::game::HeroClass::Necromancer, "Resurrect dead bosses as companions"),
    ];

    for (i, (name, class, desc)) in classes.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_classes.contains(class);
        let prefix = if game.settings_selection == i {
            ">> "
        } else {
            "   "
        };
        let status = if game.stats.equipped_class == Some(*class) {
            "[EQUIPPED]"
        } else if is_unlocked {
            "[UNLOCKED]"
        } else {
            "[500 COINS]"
        };

        let line = format!("{prefix}{name}: {desc} {status}");
        stdout.queue(SetForegroundColor(if is_unlocked {
            Color::White
        } else {
            Color::DarkGrey
        }))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 3 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{line}")?;
    }

    let unequip_prefix = if game.settings_selection == 4 {
        ">> "
    } else {
        "   "
    };
    let unequip_line = format!("{unequip_prefix}Unequip Class");
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(unequip_line.len()).unwrap_or(0) / 2),
        game.height / 2 + 5,
    ))?;
    write!(stdout, "{unequip_line}")?;

    Ok(())
}

fn draw_equipment<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "EQUIPMENT";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let items = &game.stats.unlocked_equipment;
    for (i, item) in items.iter().enumerate() {
        let name = match item {
            crate::game::Equipment::SpikedHelmet => "Spiked Helmet",
            crate::game::Equipment::HeavyArmor => "Heavy Armor",
            crate::game::Equipment::SpeedTail => "Speed Tail",
            crate::game::Equipment::MagnetRing => "Magnet Ring",
        };
        let desc = match item {
            crate::game::Equipment::SpikedHelmet => "Deal 5 damage to boss & survive hit",
            crate::game::Equipment::HeavyArmor => "Ignore 1 obstacle hit per run",
            crate::game::Equipment::SpeedTail => "Faster tick rate (-10ms)",
            crate::game::Equipment::MagnetRing => "Passive Magnet effect",
        };

        let is_equipped = game.stats.equipped_gear == Some(*item);
        let prefix = if game.settings_selection == i {
            ">> "
        } else {
            "   "
        };
        let status = if is_equipped {
            "[EQUIPPED]"
        } else {
            ""
        };
        let line = format!("{prefix}{name}: {desc} {status}");

        stdout.queue(SetForegroundColor(if is_equipped {
            Color::Green
        } else {
            Color::White
        }))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 3 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{line}")?;
    }

    let unequip_idx = items.len();
    let unequip_prefix = if game.settings_selection == unequip_idx {
        ">> "
    } else {
        "   "
    };
    let unequip_line = format!("{unequip_prefix}Unequip Gear");
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(unequip_line.len()).unwrap_or(0) / 2),
        game.height / 2 - 3 + u16::try_from(items.len()).unwrap_or(0) * 2,
    ))?;
    write!(stdout, "{unequip_line}")?;

    let help = "Up/Down: Select | Enter: Equip | Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}
fn draw_real_estate<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "REAL ESTATE OFFICE";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let options = [
        ("Buy Shack (Cost: 100, Income: 1/s)", crate::game::Property::Shack),
        ("Buy Apartment (Cost: 500, Income: 6/s)", crate::game::Property::Apartment),
        ("Buy Mansion (Cost: 2000, Income: 28/s)", crate::game::Property::Mansion),
        ("Buy Skyscraper (Cost: 10000, Income: 160/s)", crate::game::Property::Skyscraper),
        ("Back", crate::game::Property::Shack),
    ];

    let start_y = game.height / 2 - 2;
    for (i, (text, prop)) in options.iter().enumerate() {
        let owned = if i < 4 {
            game.stats.properties.get(prop).copied().unwrap_or(0)
        } else {
            0
        };

        let mut display_text = if i < 4 {
            format!("{text} (Owned: {owned})")
        } else {
            text.to_string()
        };

        if i == game.settings_selection {
            display_text = format!("> {display_text} <");
            stdout.queue(SetForegroundColor(Color::Green))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
        }

        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
            start_y + u16::try_from(i).unwrap_or(0),
        ))?;
        write!(stdout, "{display_text}")?;
    }
    Ok(())
}

fn draw_fishing<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "🎣 FISHING POND 🐟";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let help = if game.is_fishing {
        "Mash SPACEBAR to reel it in!"
    } else {
        "Press SPACEBAR to cast your line..."
    };
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height / 2 - 3,
    ))?;
    write!(stdout, "{help}")?;

    if game.is_fishing {
        let max_progress = 50;
        let p_bar_len = 20;
        let filled = (game.fishing_progress as usize * p_bar_len) / max_progress;
        let filled_str = "█".repeat(filled);
        let empty_str = "░".repeat(p_bar_len.saturating_sub(filled));

        let bar_str = format!("[{filled_str}{empty_str}]");

        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2)
                .saturating_sub(u16::try_from(bar_str.chars().count()).unwrap_or(0) / 2),
            game.height / 2 + 1,
        ))?;
        write!(stdout, "{bar_str}")?;
    } else {
        let fish_stats = format!("Caught: {:?}", game.stats.fish_caught);
        let display_len =
            u16::try_from(fish_stats.len()).unwrap_or(0).min(game.width.saturating_sub(2));
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(display_len / 2),
            game.height / 2 + 1,
        ))?;
        write!(stdout, "{}", &fish_stats[..display_len as usize])?;
    }

    let back = "Press 'Q' or 'ESC' to return to menu";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(back.len()).unwrap_or(0) / 2),
        game.height / 2 + 4,
    ))?;
    write!(stdout, "{back}")?;

    Ok(())
}

fn draw_battle_pass<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "BATTLE PASS";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 8,
    ))?;
    write!(stdout, "{title}")?;

    let xp_str = format!("Battle Pass XP: {}", game.stats.battle_pass_xp);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(xp_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{xp_str}")?;

    // Display 5 tiers centered around the selection
    let selection = u32::try_from(game.settings_selection).unwrap_or(0);
    let start_tier = selection.saturating_sub(2);
    let end_tier = (start_tier + 5).min(50);
    let start_tier = end_tier.saturating_sub(5); // Adjust start if at end

    for (i, tier_idx) in (start_tier..end_tier).enumerate() {
        let tier = tier_idx + 1;
        let required_xp = tier * 1000;
        let is_unlocked = game.stats.battle_pass_xp >= required_xp;
        let is_claimed = game.stats.claimed_battle_pass_tiers.contains(&tier);
        let is_selected = tier_idx == selection;

        let reward_str = if tier % 10 == 0 {
            if tier == 50 {
                "Exclusive Skin 🚀"
            } else {
                "5000 Coins"
            }
        } else if tier % 5 == 0 {
            "2000 Coins"
        } else {
            "500 Coins"
        };

        let status = if is_claimed {
            "[CLAIMED]"
        } else if is_unlocked {
            "[UNLOCKED]"
        } else {
            "[LOCKED]"
        };

        let prefix = if is_selected {
            ">> "
        } else {
            "   "
        };
        let color = if is_claimed {
            Color::DarkGrey
        } else if is_unlocked {
            if is_selected {
                Color::Cyan
            } else {
                Color::Green
            }
        } else {
            if is_selected {
                Color::White
            } else {
                Color::Red
            }
        };

        let line = format!("{prefix}Tier {tier} ({required_xp} XP) - {reward_str} {status}");
        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{line}")?;
    }

    let help_text = "Space/Enter: Claim Reward | Up/Down: Scroll | Esc/Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help_text.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help_text}")?;

    Ok(())
}

fn draw_artifact_shrine<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "ARTIFACT SHRINE";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let artifacts = [
        (crate::game::Artifact::CoinAmulet, "Coin Amulet (2x Coins)"),
        (crate::game::Artifact::LifeChalice, "Life Chalice (+1 Extra Life)"),
        (crate::game::Artifact::GhostCloak, "Ghost Cloak (10% dodge obstacle)"),
        (crate::game::Artifact::MagnetStone, "Magnet Stone"),
        (crate::game::Artifact::TimeCrystal, "Time Crystal"),
    ];

    for (i, (artifact, desc)) in artifacts.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_artifacts.contains(artifact);
        let status = if is_unlocked {
            "[UNLOCKED]"
        } else {
            "[LOCKED]"
        };

        let line = format!("{desc} {status}");
        stdout.queue(SetForegroundColor(if is_unlocked {
            Color::White
        } else {
            Color::DarkGrey
        }))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 3 + (i as u16) * 2,
        ))?;
        write!(stdout, "{line}")?;
    }

    let instruction = "Press ENTER to Summon (1000 Coins) - Duplicates refund 500";
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(instruction.len()).unwrap_or(0) / 2),
        game.height / 2 + 7,
    ))?;
    write!(stdout, "{instruction}")?;

    let coins = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins.len()).unwrap_or(0) / 2),
        game.height / 2 + 9,
    ))?;
    write!(stdout, "{coins}")?;

    Ok(())
}

fn draw_faction_base<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "FACTION BASE";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 6,
    ))?;
    write!(stdout, "{title}")?;

    let stat_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stat_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 4,
    ))?;
    write!(stdout, "{stat_str}")?;

    if let Some(faction) = game.stats.faction {
        let current_faction_str = format!("Current Faction: {}", faction.name());
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2)
                .saturating_sub(u16::try_from(current_faction_str.len()).unwrap_or(0) / 2),
            game.height / 2 - 2,
        ))?;
        write!(stdout, "{current_faction_str}")?;

        let rep_str = format!("Reputation: {}", game.stats.faction_rep);
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(rep_str.len()).unwrap_or(0) / 2),
            game.height / 2,
        ))?;
        write!(stdout, "{rep_str}")?;

        let perk_str = format!("Perk: {}", faction.description());
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(perk_str.len()).unwrap_or(0) / 2),
            game.height / 2 + 2,
        ))?;
        write!(stdout, "{perk_str}")?;

        let leave_str = "> Leave Faction <";
        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(leave_str.len()).unwrap_or(0) / 2),
            game.height / 2 + 5,
        ))?;
        write!(stdout, "{leave_str}")?;
    } else {
        let factions = [
            crate::game::Faction::CrimsonVipers,
            crate::game::Faction::AzureCobras,
            crate::game::Faction::EmeraldPythons,
        ];

        for (i, faction) in factions.iter().enumerate() {
            let desc = format!("{}: {}", faction.name(), faction.description());
            let prefix = if i == game.settings_selection {
                ">"
            } else {
                " "
            };
            let suffix = if i == game.settings_selection {
                "<"
            } else {
                " "
            };
            let color = if i == game.settings_selection {
                Color::Yellow
            } else {
                Color::White
            };

            let display_text = format!("{prefix} {desc} {suffix}");

            stdout.queue(SetForegroundColor(color))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(display_text.len()).unwrap_or(0) / 2),
                game.height / 2 - 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{display_text}")?;
        }
    }

    let help = "Up/Down: Select | Enter: Join/Leave | Q: Back";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help}")?;

    Ok(())
}

fn draw_quest_log<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "QUEST LOG";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        2,
    ))?;
    write!(stdout, "{title}")?;

    let mut start_y = 5;
    stdout.queue(SetForegroundColor(Color::White))?;

    if game.stats.active_quests.is_empty() {
        let msg = "No active quests.";
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2),
            start_y,
        ))?;
        write!(stdout, "{msg}")?;
        start_y += 2;
    } else {
        for quest in &game.stats.active_quests {
            let quest_str = format!(
                "{} - {} ({}/{}) - Reward: {} Coins",
                quest.name, quest.description, quest.progress, quest.target, quest.reward
            );
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(quest_str.len()).unwrap_or(0) / 2),
                start_y,
            ))?;
            write!(stdout, "{quest_str}")?;
            start_y += 1;
        }
    }

    start_y += 2;
    let completed_title = "COMPLETED QUESTS:";
    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(completed_title.len()).unwrap_or(0) / 2),
        start_y,
    ))?;
    write!(stdout, "{completed_title}")?;
    start_y += 1;

    if game.stats.completed_quests.is_empty() {
        let msg = "No completed quests.";
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2),
            start_y,
        ))?;
        write!(stdout, "{msg}")?;
    } else {
        for q_type in &game.stats.completed_quests {
            let msg = format!("{q_type:?}");
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2),
                start_y,
            ))?;
            write!(stdout, "{msg}")?;
            start_y += 1;
        }
    }

    let footer = "Press Q or Esc to Return";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(footer.len()).unwrap_or(0) / 2),
        game.height.saturating_sub(2),
    ))?;
    write!(stdout, "{footer}")?;

    Ok(())
}

pub fn draw_bestiary<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "BESTIARY";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        2,
    ))?;
    write!(stdout, "{title}")?;

    let bosses = [
        crate::game::BossType::Shooter,
        crate::game::BossType::Charger,
        crate::game::BossType::Spawner,
        crate::game::BossType::Teleporter,
        crate::game::BossType::Splitter,
        crate::game::BossType::Trapper,
        crate::game::BossType::Necromancer,
        crate::game::BossType::ShadowClone,
        crate::game::BossType::Mimic,
        crate::game::BossType::Puffer,
        crate::game::BossType::Juggernaut,
        crate::game::BossType::Dragon,
    ];

    for (i, boss) in bosses.iter().enumerate() {
        let is_selected = i == game.settings_selection;
        let kills = game.stats.bestiary.get(boss).copied().unwrap_or(0);
        let name = format!("{boss:?}");

        let color = if is_selected {
            Color::Yellow
        } else {
            Color::White
        };
        let prefix = if is_selected {
            "> "
        } else {
            "  "
        };

        let display_str = format!("{prefix}{name} - Kills: {kills}");

        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(4, 5 + u16::try_from(i).unwrap_or(0)))?;
        write!(stdout, "{display_str}")?;

        if is_selected {
            let lore = crate::game::bestiary::get_boss_lore(boss, kills);
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(cursor::MoveTo(4, 5 + u16::try_from(bosses.len()).unwrap_or(0) + 2))?;
            write!(stdout, "Lore: {lore}")?;
        }
    }

    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    let help_msg = "Press W/S to navigate | Q/Esc to go back";
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(help_msg.len()).unwrap_or(0) / 2),
        game.height - 2,
    ))?;
    write!(stdout, "{help_msg}")?;

    Ok(())
}

pub fn draw_tavern<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "THE TAVERN";
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        game.height / 2 - 5,
    ))?;
    write!(stdout, "{title}")?;

    let subtitle = "A place to rest and meet travelers";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(subtitle.len()).unwrap_or(0) / 2),
        game.height / 2 - 3,
    ))?;
    write!(stdout, "{subtitle}")?;

    let stats_str = format!("Coins: {} | Lives: {}", game.stats.coins, game.lives);
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(stats_str.len()).unwrap_or(0) / 2),
        game.height / 2 - 1,
    ))?;
    write!(stdout, "{stats_str}")?;

    // Draw recent chat logs if any
    let chat_start_y = game.height / 2 + 10;
    for (i, (msg, color)) in game.chat_log.iter().rev().take(5).enumerate() {
        let y = chat_start_y + u16::try_from(i).unwrap_or(0);
        if y < game.height {
            stdout.queue(SetForegroundColor((*color).into()))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap_or(0) / 2),
                y,
            ))?;
            write!(stdout, "{msg}")?;
        }
    }

    let menu_items = ["Talk to Barkeep", "Play Dice", "Rest (Restore Lives)", "Leave Tavern"];

    for (i, item) in menu_items.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(cursor::MoveTo(
                game.width / 2 - 10,
                game.height / 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "> {item}")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                game.width / 2 - 8,
                game.height / 2 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{item}")?;
        }
    }

    Ok(())
}

pub fn draw_black_market<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "BLACK MARKET";
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        2,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let options = [
        "1. Buy Shadow Cloak (Artifact) - 5000 Coins",
        "2. Buy Hacker Theme - 2000 Coins",
        "3. Buy Corrupted Egg - 3000 Coins",
        "4. Buy Forbidden Spell (Fireball) - 4000 Coins",
        "5. Sell Max Mana (+1000 Coins)",
        "Leave Black Market",
    ];

    for (i, option) in options.iter().enumerate() {
        let is_selected = i == game.settings_selection;
        let color = if is_selected {
            Color::Red
        } else {
            Color::DarkGrey
        };
        let prefix = if is_selected {
            "> "
        } else {
            "  "
        };

        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(option.len() + 2).unwrap_or(0) / 2),
            6 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{prefix}{option}")?;
    }

    Ok(())
}

pub fn draw_bank<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "BANK";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        2,
    ))?;
    write!(stdout, "{title}")?;

    let bank_str = format!("Bank Balance: {}", game.stats.bank_balance);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(bank_str.len()).unwrap_or(0) / 2),
        4,
    ))?;
    write!(stdout, "{bank_str}")?;

    let coins_str = format!("Coins on Hand: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        6,
    ))?;
    write!(stdout, "{coins_str}")?;

    let options = ["1. Deposit 100 Coins", "2. Withdraw 100 Coins", "Leave Bank"];

    for (i, option) in options.iter().enumerate() {
        let is_selected = i == game.settings_selection;
        let color = if is_selected {
            Color::Green
        } else {
            Color::DarkGrey
        };
        let prefix = if is_selected {
            "> "
        } else {
            "  "
        };

        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(option.len() + 2).unwrap_or(0) / 2),
            8 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{prefix}{option}")?;
    }

    Ok(())
}

pub fn draw_auction_house<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "AUCTION HOUSE";
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(title.len()).unwrap_or(0) / 2),
        2,
    ))?;
    write!(stdout, "{title}")?;

    let coins_str = format!("Coins: {}", game.stats.coins);
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(coins_str.len()).unwrap_or(0) / 2),
        4,
    ))?;
    write!(stdout, "{coins_str}")?;

    let options = [
        "1. Bid on Mystery Artifact (5000 Coins)",
        "2. Bid on Rare Theme (2000 Coins)",
        "3. Bid on Epic Boss Pet (10000 Coins)",
        "Leave Auction House",
    ];

    for (i, option) in options.iter().enumerate() {
        let is_selected = i == game.settings_selection;
        let color = if is_selected {
            Color::Green
        } else {
            Color::DarkGrey
        };
        let prefix = if is_selected {
            "> "
        } else {
            "  "
        };

        stdout.queue(SetForegroundColor(color))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(option.len() + 2).unwrap_or(0) / 2),
            8 + u16::try_from(i).unwrap_or(0) * 2,
        ))?;
        write!(stdout, "{prefix}{option}")?;
    }

    Ok(())
}

#[cfg(test)]
mod auction_tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn test_draw_auction_house() {
        let mut game = Game::new(
            40,
            40,
            false,
            'O',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        game.settings_selection = 1;
        game.stats.coins = 9000;

        let mut buf = Vec::new();
        draw_auction_house(&game, &mut buf).expect("Valid operation in tests");
        let output = String::from_utf8(buf).expect("Valid operation in tests");

        assert!(output.contains("AUCTION HOUSE"), "Should contain title");
        assert!(output.contains("Coins: 9000"), "Should contain coins");
        assert!(output.contains("> 2. Bid on Rare Theme"), "Should highlight selected option");
    }
}

pub fn draw_gacha<W: Write>(game: &Game, stdout: &mut W) -> io::Result<()> {
    let title = "GACHA (PULL FOR RESOURCES)";
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

    let last_pull_msg = if game.death_message.is_empty() {
        "Press Enter to pull!".to_string()
    } else {
        format!("Last Pull: {}", game.death_message)
    };
    let last_pull_len = u16::try_from(last_pull_msg.len()).unwrap_or(0);
    stdout.queue(SetForegroundColor(Color::Magenta))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(last_pull_len / 2),
        game.height / 4 + 4,
    ))?;
    write!(stdout, "{last_pull_msg}")?;

    let options = ["1 Pull (100 Coins)", "10 Pulls (1000 Coins)", "Back to Menu"];

    for (i, opt) in options.iter().enumerate() {
        if i == game.settings_selection {
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(opt.len() + 3).unwrap_or(0) / 2),
                game.height / 4 + 7 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, ">> {opt}")?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(cursor::MoveTo(
                (game.width / 2).saturating_sub(u16::try_from(opt.len()).unwrap_or(0) / 2),
                game.height / 4 + 7 + u16::try_from(i).unwrap_or(0) * 2,
            ))?;
            write!(stdout, "{opt}")?;
        }
    }

    Ok(())
}
