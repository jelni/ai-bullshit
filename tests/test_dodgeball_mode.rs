use snake_game::*;
use game::GameMode;

#[test]
fn test_dodgeball_laser_bounce() {
    let mut game = game::Game::new(20, 20, true, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = GameMode::Dodgeball;

    let initial_pos = snake::Point { x: 1, y: 5 };
    game.lasers.push(game::Laser {
        position: initial_pos,
        direction: snake::Direction::Left,
        player: 3,
    });

    game.state = game::GameState::Playing; game.update(); // Move laser to the left wall (x=0)

    assert_eq!(game.lasers.len(), 1, "Laser should not be destroyed in Dodgeball mode");
    assert_eq!(game.lasers[0].position.x, 2, "Laser should have bounced back");
    assert_eq!(game.lasers[0].direction, snake::Direction::Right, "Laser direction should be inverted");
}
