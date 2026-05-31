with open('src/ui.rs', 'r') as f:
    content = f.read()

# 1. Add draw_artifact_shrine to dispatcher
content = content.replace(
    'GameState::BattlePass => draw_battle_pass(game, stdout)?,',
    'GameState::BattlePass => draw_battle_pass(game, stdout)?,\n        GameState::ArtifactShrine => draw_artifact_shrine(game, stdout)?,'
)

# 2. Add Artifact Shrine to menu
content = content.replace(
    '"Battle Pass",\n        "Quit",',
    '"Battle Pass",\n        "Artifact Shrine",\n        "Quit",'
)

# 3. Create draw_artifact_shrine function
new_func = """
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
        let status = if is_unlocked { "[UNLOCKED]" } else { "[LOCKED]" };

        let line = format!("{desc} {status}");
        stdout.queue(SetForegroundColor(if is_unlocked { Color::White } else { Color::DarkGrey }))?;
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
"""
content += new_func

with open('src/ui.rs', 'w') as f:
    f.write(content)
