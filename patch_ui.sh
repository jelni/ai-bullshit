#!/bin/bash
sed -i 's/        GameState::CompanionCamp => draw_companion_camp(game, stdout)?,/        GameState::CompanionCamp => draw_companion_camp(game, stdout)?,\n        GameState::ClassSelect => draw_class_select(game, stdout)?,/' src/ui.rs
sed -i 's/"Companion Camp",/"Companion Camp",\n        "Class Select",/' src/ui.rs

cat << 'INNER_EOF' >> src/ui.rs

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
    ];

    for (i, (name, class, desc)) in classes.iter().enumerate() {
        let is_unlocked = game.stats.unlocked_classes.contains(class);
        let prefix = if game.settings_selection == i as u8 { ">> " } else { "   " };
        let status = if game.stats.equipped_class == Some(*class) {
            "[EQUIPPED]"
        } else if is_unlocked {
            "[UNLOCKED]"
        } else {
            "[500 COINS]"
        };

        let line = format!("{prefix}{name}: {desc} {status}");
        stdout.queue(SetForegroundColor(if is_unlocked { Color::White } else { Color::DarkGrey }))?;
        stdout.queue(cursor::MoveTo(
            (game.width / 2).saturating_sub(u16::try_from(line.len()).unwrap_or(0) / 2),
            game.height / 2 - 3 + (i as u16) * 2,
        ))?;
        write!(stdout, "{line}")?;
    }

    let unequip_prefix = if game.settings_selection == 4 { ">> " } else { "   " };
    let unequip_line = format!("{unequip_prefix}Unequip Class");
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(cursor::MoveTo(
        (game.width / 2).saturating_sub(u16::try_from(unequip_line.len()).unwrap_or(0) / 2),
        game.height / 2 + 5,
    ))?;
    write!(stdout, "{unequip_line}")?;

    Ok(())
}
INNER_EOF
