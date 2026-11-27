use crate::game::{Game, GameState};
use web_sys::CanvasRenderingContext2d;

use rand::{Rng, SeedableRng};

#[expect(clippy::too_many_lines)]
/// Draws the game to the given canvas context.
///
/// # Panics
///
/// Panics if the active power-up type is unwrapped but does not exist.
pub fn draw(game: &Game, ctx: &CanvasRenderingContext2d) {
    let cell_size = 15.0;

    let is_visible = |px: f64, py: f64| -> bool {
        if game.mode == crate::game::GameMode::FogOfWar
            || game.mode == crate::game::GameMode::Sandstorm
            || game.time_of_day == crate::game::TimeOfDay::Night
            || game.weather == crate::game::Weather::Fog
            || (game.mode == crate::game::GameMode::Eclipse && game.eclipse_flare == 0)
        {
            let head = game.snake.head();
            let dx = px - f64::from(head.x);
            let dy = py - f64::from(head.y);
            f64::hypot(dx, dy) <= 6.0
        } else {
            true
        }
    };

    // Clear canvas
    ctx.set_fill_style_str("#000000");
    ctx.fill_rect(0.0, 0.0, f64::from(game.width) * cell_size, f64::from(game.height) * cell_size);

    if game.mode == crate::game::GameMode::TurfWar {
        for (pt, &id) in &game.painted_tiles {
            if is_visible(f64::from(pt.x), f64::from(pt.y)) {
                if id == 1 {
                    ctx.set_fill_style_str("#000033");
                } else if id == 2 {
                    ctx.set_fill_style_str("#330033");
                } else if id == 3 {
                    ctx.set_fill_style_str("#330000");
                } else if id == 4 {
                    ctx.set_fill_style_str("#003300");
                } else {
                    ctx.set_fill_style_str("#333300");
                }
                ctx.fill_rect(
                    f64::from(pt.x) * cell_size,
                    f64::from(pt.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

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

    // Draw telegraphed meteor impact zones
    ctx.set_fill_style_str("#8B0000"); // DarkRed
    for cell in game.meteor_impact_cells() {
        if cell.x < game.width
            && cell.y < game.height
            && is_visible(f64::from(cell.x), f64::from(cell.y))
        {
            ctx.fill_rect(
                f64::from(cell.x) * cell_size,
                f64::from(cell.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw food
    ctx.set_fill_style_str("#00FF00");
    if game.food.x < game.width
        && game.food.y < game.height
        && is_visible(f64::from(game.food.x), f64::from(game.food.y))
    {
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
        if obs.x < game.width
            && obs.y < game.height
            && is_visible(f64::from(obs.x), f64::from(obs.y))
        {
            ctx.fill_rect(
                f64::from(obs.x) * cell_size,
                f64::from(obs.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw sand drifts (Sandstorm mode)
    if game.mode == crate::game::GameMode::Sandstorm {
        ctx.set_fill_style_str("#D2B48C"); // Tan
        for sand in &game.sand_tiles {
            if sand.x < game.width
                && sand.y < game.height
                && is_visible(f64::from(sand.x), f64::from(sand.y))
            {
                ctx.fill_rect(
                    f64::from(sand.x) * cell_size,
                    f64::from(sand.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw acid puddles (Acid Rain mode)
    if game.mode == crate::game::GameMode::AcidRain {
        ctx.set_fill_style_str("#00FF00"); // Green
        for puddle in &game.acid_puddles {
            if puddle.x < game.width
                && puddle.y < game.height
                && is_visible(f64::from(puddle.x), f64::from(puddle.y))
            {
                ctx.fill_rect(
                    f64::from(puddle.x) * cell_size,
                    f64::from(puddle.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw the bomb on the carrier's head (Bomb Tag mode)
    if game.mode == crate::game::GameMode::BombTag
        && let Some(carrier) = game.bomb_carrier
    {
        let head = if carrier == 1 {
            game.snake.head()
        } else if carrier == 2 {
            game
                .player2
                .as_ref()
                .map_or_else(|| game.snake.head(), crate::snake::Snake::head)
        } else {
            game
                .bots
                .get(usize::from(carrier.saturating_sub(3)))
                .map_or_else(|| game.snake.head(), crate::snake::Snake::head)
        };
        if head.x < game.width
            && head.y < game.height
            && is_visible(f64::from(head.x), f64::from(head.y))
        {
            ctx.set_fill_style_str("#FF0000"); // Red
            ctx.fill_rect(
                f64::from(head.x) * cell_size,
                f64::from(head.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw the contagion on infected heads (Infection mode)
    if game.mode == crate::game::GameMode::Infection {
        for id in &game.infected_ids {
            let head = if *id == 1 {
                game.snake.head()
            } else if *id == 2 {
                game
                    .player2
                    .as_ref()
                    .map_or_else(|| game.snake.head(), crate::snake::Snake::head)
            } else {
                game
                    .bots
                    .get(usize::from(id.saturating_sub(3)))
                    .map_or_else(|| game.snake.head(), crate::snake::Snake::head)
            };
            if head.x < game.width
                && head.y < game.height
                && is_visible(f64::from(head.x), f64::from(head.y))
            {
                ctx.set_fill_style_str("#00FF00"); // Green
                ctx.fill_rect(
                    f64::from(head.x) * cell_size,
                    f64::from(head.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw snow drifts (Blizzard mode)
    if game.mode == crate::game::GameMode::Blizzard {
        ctx.set_fill_style_str("#FFFFFF");
        for snow in &game.snow_tiles {
            if snow.x < game.width
                && snow.y < game.height
                && is_visible(f64::from(snow.x), f64::from(snow.y))
            {
                ctx.fill_rect(
                    f64::from(snow.x) * cell_size,
                    f64::from(snow.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw lava flows and volcano vents (Volcano mode)
    if game.mode == crate::game::GameMode::Volcano {
        ctx.set_fill_style_str("#FF4500");
        for lava in &game.lava_tiles {
            if lava.x < game.width
                && lava.y < game.height
                && is_visible(f64::from(lava.x), f64::from(lava.y))
            {
                ctx.fill_rect(
                    f64::from(lava.x) * cell_size,
                    f64::from(lava.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
        ctx.set_fill_style_str("#8B0000");
        for vent in &game.volcano_vents {
            if vent.x < game.width
                && vent.y < game.height
                && is_visible(f64::from(vent.x), f64::from(vent.y))
            {
                ctx.fill_rect(
                    f64::from(vent.x) * cell_size,
                    f64::from(vent.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw burning cells and ash (Wildfire mode)
    if game.mode == crate::game::GameMode::Wildfire {
        ctx.set_fill_style_str("#FF0000"); // Red
        for fire in &game.fire_tiles {
            if fire.x < game.width
                && fire.y < game.height
                && is_visible(f64::from(fire.x), f64::from(fire.y))
            {
                ctx.fill_rect(
                    f64::from(fire.x) * cell_size,
                    f64::from(fire.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
        ctx.set_fill_style_str("#404040"); // Dark grey ash
        for ash in &game.ash_tiles {
            if ash.x < game.width
                && ash.y < game.height
                && is_visible(f64::from(ash.x), f64::from(ash.y))
            {
                ctx.fill_rect(
                    f64::from(ash.x) * cell_size,
                    f64::from(ash.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }


    // Draw inferno flames and hotspots (Inferno mode)
    if game.mode == crate::game::GameMode::Inferno {
        ctx.set_fill_style_str("#FF8C00"); // DarkOrange
        for fire in &game.inferno_tiles {
            if fire.x < game.width
                && fire.y < game.height
                && is_visible(f64::from(fire.x), f64::from(fire.y))
            {
                ctx.fill_rect(
                    f64::from(fire.x) * cell_size,
                    f64::from(fire.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
        ctx.set_fill_style_str("#FF0000"); // Red
        for hotspot in &game.inferno_hotspots {
            if hotspot.x < game.width
                && hotspot.y < game.height
                && is_visible(f64::from(hotspot.x), f64::from(hotspot.y))
            {
                ctx.fill_rect(
                    f64::from(hotspot.x) * cell_size,
                    f64::from(hotspot.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw rubble piles (Earthquake mode)
    if game.mode == crate::game::GameMode::Earthquake {
        ctx.set_fill_style_str("#8B4513"); // SaddleBrown
        for rubble in &game.rubble_tiles {
            if rubble.x < game.width
                && rubble.y < game.height
                && is_visible(f64::from(rubble.x), f64::from(rubble.y))
            {
                ctx.fill_rect(
                    f64::from(rubble.x) * cell_size,
                    f64::from(rubble.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw seismic rubble piles (Seismic mode)
    if game.mode == crate::game::GameMode::Seismic {
        ctx.set_fill_style_str("#696969");
        for rubble in &game.seismic_rubble {
            if rubble.x < game.width
                && rubble.y < game.height
                && is_visible(f64::from(rubble.x), f64::from(rubble.y))
            {
                ctx.fill_rect(
                    f64::from(rubble.x) * cell_size,
                    f64::from(rubble.y) * cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
    }

    // Draw Turrets
    ctx.set_fill_style_str("#008B8B"); // DarkCyan
    for turret in &game.turrets {
        if turret.position.x < game.width
            && turret.position.y < game.height
            && is_visible(f64::from(turret.position.x), f64::from(turret.position.y))
        {
            ctx.fill_rect(
                f64::from(turret.position.x) * cell_size,
                f64::from(turret.position.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Mines
    ctx.set_fill_style_str("#FFA500");
    for mine in &game.mines {
        if mine.x < game.width
            && mine.y < game.height
            && is_visible(f64::from(mine.x), f64::from(mine.y))
        {
            ctx.fill_rect(
                f64::from(mine.x) * cell_size,
                f64::from(mine.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Meteors
    ctx.set_fill_style_str("#FF8C00"); // DarkOrange
    for meteor in &game.meteors {
        if meteor.position.x < game.width
            && meteor.position.y < game.height
            && is_visible(f64::from(meteor.position.x), f64::from(meteor.position.y))
        {
            ctx.fill_rect(
                f64::from(meteor.position.x) * cell_size,
                f64::from(meteor.position.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Goblin
    if let Some(goblin) = game.goblin {
        ctx.set_fill_style_str("#FFD700"); // Gold
        if goblin.position.x < game.width
            && goblin.position.y < game.height
            && is_visible(f64::from(goblin.position.x), f64::from(goblin.position.y))
        {
            ctx.fill_rect(
                f64::from(goblin.position.x) * cell_size,
                f64::from(goblin.position.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Black Hole
    if let Some(bh) = game.black_hole {
        ctx.set_fill_style_str("#1A1A1A"); // DarkGrey/Blackish
        if bh.x < game.width && bh.y < game.height {
            ctx.fill_rect(
                f64::from(bh.x) * cell_size,
                f64::from(bh.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw Tornado vortex (Tornado mode)
    if let Some(t) = game.tornado
        && t.x < game.width
        && t.y < game.height
        && is_visible(f64::from(t.x), f64::from(t.y))
    {
        ctx.set_fill_style_str("#696969"); // DarkGrey
        ctx.fill_rect(
            f64::from(t.x) * cell_size,
            f64::from(t.y) * cell_size,
            cell_size,
            cell_size,
        );
    }

    // Draw poison food
    if let Some((poison_p, _)) = game.poison_food {
        ctx.set_fill_style_str("#800080"); // DarkMagenta
        if poison_p.x < game.width
            && poison_p.y < game.height
            && is_visible(f64::from(poison_p.x), f64::from(poison_p.y))
        {
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
        if p1.x < game.width && p1.y < game.height && is_visible(f64::from(p1.x), f64::from(p1.y)) {
            ctx.set_fill_style_str("#00FFFF"); // Cyan
            ctx.fill_rect(
                f64::from(p1.x) * cell_size,
                f64::from(p1.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
        if p2.x < game.width && p2.y < game.height && is_visible(f64::from(p2.x), f64::from(p2.y)) {
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
    for boss in &game.bosses {
        let is_assassin_hidden = boss.kind == crate::game::BossType::Assassin && {
            let dist_x = i32::from(game.snake.head().x).abs_diff(i32::from(boss.position.x));
            let dist_y = i32::from(game.snake.head().y).abs_diff(i32::from(boss.position.y));
            dist_x + dist_y > 5
        };

        if is_assassin_hidden {
            continue;
        }

        match boss.kind {
            crate::game::BossType::Shooter => ctx.set_fill_style_str("#FF00FF"),
            crate::game::BossType::Charger => ctx.set_fill_style_str("#FF0000"),
            crate::game::BossType::Spawner => ctx.set_fill_style_str("#006400"),
            crate::game::BossType::Teleporter => ctx.set_fill_style_str("#800080"),
            crate::game::BossType::Splitter => ctx.set_fill_style_str("#FF4500"),
            crate::game::BossType::Trapper => ctx.set_fill_style_str("#8B4513"),
            crate::game::BossType::Necromancer => ctx.set_fill_style_str("#2F4F4F"),
            crate::game::BossType::ShadowClone => ctx.set_fill_style_str("#696969"),
            crate::game::BossType::Puffer => ctx.set_fill_style_str("#FFA500"),
            crate::game::BossType::Juggernaut => ctx.set_fill_style_str("#8B0000"),
            crate::game::BossType::Dragon => ctx.set_fill_style_str("#FF0000"),
            crate::game::BossType::Mage => ctx.set_fill_style_str("#0000FF"),
            crate::game::BossType::Gorgon => ctx.set_fill_style_str("#008000"),
            crate::game::BossType::VampireLord => ctx.set_fill_style_str("#FF0000"),
            crate::game::BossType::Kraken => ctx.set_fill_style_str("#00008B"),
            crate::game::BossType::Phantom => ctx.set_fill_style_str("#555555"),
            crate::game::BossType::Alchemist => ctx.set_fill_style_str("#006400"),
            crate::game::BossType::Engineer => ctx.set_fill_style_str("#FFD700"),
            crate::game::BossType::Assassin => ctx.set_fill_style_str("#A9A9A9"), // DarkGray
            crate::game::BossType::Mimic => {
                let target_pos = if let Some((decoy_pos, _)) = game.decoy {
                    decoy_pos
                } else {
                    game.snake.head()
                };
                let dist_x = i32::from(target_pos.x).abs_diff(i32::from(boss.position.x));
                let dist_y = i32::from(target_pos.y).abs_diff(i32::from(boss.position.y));
                if dist_x <= 3 && dist_y <= 3 {
                    ctx.set_fill_style_str("#8B0000"); // DarkRed
                } else {
                    ctx.set_fill_style_str("#FFFF00"); // Yellow (bonus food)
                }
            },
            crate::game::BossType::TimeWeaver => ctx.set_fill_style_str("#00FFFF"), // Cyan
            crate::game::BossType::Illusionist => ctx.set_fill_style_str("#FF00FF"), // Magenta
            crate::game::BossType::IllusionClone => ctx.set_fill_style_str("#00FFFF"), // Cyan
            crate::game::BossType::Wormhole => ctx.set_fill_style_str("#00008B"),   // DarkBlue
            crate::game::BossType::Glitch => ctx.set_fill_style_str("#FF69B4"),     // HotPink
            crate::game::BossType::Mecha => ctx.set_fill_style_str("#00FFFF"),      // Cyan
            crate::game::BossType::Leviathan => ctx.set_fill_style_str("#008080"),  // Teal
            crate::game::BossType::Behemoth => ctx.set_fill_style_str("#8B0000"),   // DarkRed
        }
        if boss.position.x < game.width
            && boss.position.y < game.height
            && is_visible(f64::from(boss.position.x), f64::from(boss.position.y))
        {
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
        if bonus_p.x < game.width
            && bonus_p.y < game.height
            && is_visible(f64::from(bonus_p.x), f64::from(bonus_p.y))
        {
            ctx.fill_rect(
                f64::from(bonus_p.x) * cell_size,
                f64::from(bonus_p.y) * cell_size,
                cell_size,
                cell_size,
            );
        }
    }

    // Draw power_up
    if game.power_up.as_ref().is_some_and(|power_up| {
        power_up.activation_time.is_none()
            && power_up.location.x < game.width
            && power_up.location.y < game.height
            && is_visible(f64::from(power_up.location.x), f64::from(power_up.location.y))
    }) {
        let power_up = game.power_up.as_ref().unwrap();
        ctx.set_fill_style_str("#00FFFF");
        ctx.fill_rect(
            f64::from(power_up.location.x) * cell_size,
            f64::from(power_up.location.y) * cell_size,
            cell_size,
            cell_size,
        );
    }

    // Draw lasers
    for laser in &game.lasers {
        if laser.position.x < game.width
            && laser.position.y < game.height
            && is_visible(f64::from(laser.position.x), f64::from(laser.position.y))
        {
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

    // Draw Weather Effects
    let margin = if game.mode == crate::game::GameMode::BattleRoyale {
        game.safe_zone_margin
    } else {
        0
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "We only need lower bits for a deterministic PRNG seed"
    )]
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    );

    match game.weather {
        crate::game::Weather::Rain => {
            ctx.set_fill_style_str("#00FFFF"); // Cyan
            for _ in 0..15 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y)) {
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, cell_size * 0.4),
                        f64::from(y) * cell_size,
                        cell_size * 0.2,
                        cell_size,
                    );
                }
            }
        },
        crate::game::Weather::Snow => {
            ctx.set_fill_style_str("#FFFFFF"); // White
            for _ in 0..10 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y)) {
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, cell_size * 0.3),
                        f64::from(y).mul_add(cell_size, cell_size * 0.3),
                        cell_size * 0.4,
                        cell_size * 0.4,
                    );
                }
            }
        },
        crate::game::Weather::Sandstorm => {
            ctx.set_fill_style_str("#DAA520"); // Goldenrod
            for _ in 0..20 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y)) {
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, cell_size * 0.2),
                        f64::from(y).mul_add(cell_size, cell_size * 0.5),
                        cell_size * 0.3,
                        cell_size * 0.1,
                    );
                }
            }
        },
        crate::game::Weather::Earthquake => {
            ctx.set_fill_style_str("#8B4513"); // SaddleBrown
            for _ in 0..10 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y)) {
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, 0.0),
                        f64::from(y).mul_add(cell_size, cell_size * 0.8),
                        cell_size,
                        cell_size * 0.2,
                    );
                }
            }
        },
        crate::game::Weather::Eclipse => {
            ctx.set_fill_style_str("#444444");
            for _ in 0..10 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if !game.obstacles.contains(&crate::snake::Point {
                    x,
                    y,
                }) {
                    ctx.fill_rect(
                        f64::from(x) * cell_size,
                        f64::from(y) * cell_size,
                        cell_size,
                        cell_size,
                    );
                }
            }
        },
        crate::game::Weather::Wildfire => {
            ctx.set_fill_style_str("#FF4500"); // OrangeRed embers
            for _ in 0..15 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y))
                    && !game.fire_tiles.contains(&crate::snake::Point { x, y })
                {
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, cell_size * 0.3),
                        f64::from(y).mul_add(cell_size, cell_size * 0.3),
                        cell_size * 0.4,
                        cell_size * 0.4,
                    );
                }
            }
        },
        crate::game::Weather::Fog => {
            ctx.set_fill_style_str("#888888");
            for _ in 0..15 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if !game.obstacles.contains(&crate::snake::Point {
                    x,
                    y,
                }) {
                    ctx.fill_rect(
                        f64::from(x) * cell_size,
                        f64::from(y) * cell_size,
                        cell_size,
                        cell_size * 0.2,
                    );
                }
            }
        },
        crate::game::Weather::Inferno => {
            for _ in 0..12 {
                let x =
                    rng.gen_range(margin + 1..game.width.saturating_sub(margin).max(margin + 2));
                let y =
                    rng.gen_range(margin + 1..game.height.saturating_sub(margin).max(margin + 2));
                if is_visible(f64::from(x), f64::from(y))
                    && !game.obstacles.contains(&crate::snake::Point { x, y })
                {
                    if rng.gen_bool(0.5) {
                        ctx.set_fill_style_str("#FF8C00"); // DarkOrange
                    } else {
                        ctx.set_fill_style_str("#FF0000"); // Red
                    }
                    ctx.fill_rect(
                        f64::from(x).mul_add(cell_size, cell_size * 0.3),
                        f64::from(y).mul_add(cell_size, cell_size * 0.3),
                        cell_size * 0.4,
                        cell_size * 0.4,
                    );
                }
            }
        },
        _ => {},
    }

    // Draw Lightning Strike
    if let Some(col) = game.lightning_column {
        ctx.set_fill_style_str("#FFFF00"); // Yellow
        for y in margin + 1..game.height.saturating_sub(margin).saturating_sub(1) {
            if is_visible(f64::from(col), f64::from(y)) {
                ctx.fill_rect(
                    f64::from(col).mul_add(cell_size, cell_size * 0.4),
                    f64::from(y) * cell_size,
                    cell_size * 0.2,
                    cell_size,
                );
            }
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
        if px < game.width && py < game.height && is_visible(f64::from(px), f64::from(py)) {
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
        if part.x < game.width
            && part.y < game.height
            && is_visible(f64::from(part.x), f64::from(part.y))
        {
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
            if part.x < game.width
                && part.y < game.height
                && is_visible(f64::from(part.x), f64::from(part.y))
            {
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
