use snake_game::game::{Boss, BossType, Difficulty, Game, GameMode, GameState};
use snake_game::snake::Point;

#[test]
fn test_engineer_boss_spawns_turrets() {
    let mut game = Game::new(30, 30, false, 'X', Default::default(), Difficulty::Normal);
    game.mode = GameMode::BossRush;
    game.state = GameState::Playing; // need to set state playing otherwise update does nothing
    game.bosses.clear();
    game.turrets.clear();

    let engineer_pos = Point {
        x: 15,
        y: 15,
    };
    game.bosses.push(Boss {
        position: engineer_pos,
        health: 100,
        max_health: 100,
        move_timer: 0,
        shoot_timer: 59,
        kind: BossType::Engineer,
        state_timer: 0,
    });

    for _ in 0..65 {
        game.update();
    }

    assert!(game.turrets.len() >= 1, "Engineer boss should have spawned a turret");

    for _ in 0..500 {
        game.update();
    }
    assert!(game.turrets.len() <= 5, "Engineer boss should not spawn more than 5 turrets");
}
