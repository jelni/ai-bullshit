use snake_game::game::{Boss, BossType, Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::Point;

#[test]
fn test_bot_avoids_phantom() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.mode = GameMode::SinglePlayer;
    game.state = GameState::Playing;
    game.auto_pilot = true;

    // Place the bot at (10, 10)
    let bot_pos = Point {
        x: 10,
        y: 10,
    };
    game.snake = snake_game::snake::Snake::new(bot_pos);

    // Place a Phantom right next to the bot at (10, 11)
    let phantom_pos = Point {
        x: 10,
        y: 11,
    };
    game.bosses.push(Boss {
        position: phantom_pos,
        health: 10,
        max_health: 10,
        move_timer: 10, // Force move
        shoot_timer: 0,
        kind: BossType::Phantom,
        state_timer: 0,
    });

    // Surround the bot with obstacles except for one opening, to see if the bot escapes
    // Or simpler: just let it update and see if the bot runs away from the Phantom

    // The Phantom moves 1 tile per move (move_threshold=1 because its move_threshold gets halved to 1).
    // The bot's head is at (10,10). The Phantom is at (10,11).
    // The bot should NOT try to move to (10,11) or stay near the Phantom.

    game.food = Point {
        x: 10,
        y: 15,
    }; // Food is below, near Phantom

    let is_safe_10_11 = game.is_safe_final_p(phantom_pos, 1, 1);
    assert!(!is_safe_10_11, "The bot should identify the Phantom's position as unsafe.");

    game.update();

    let new_head = game.snake.head();
    assert_ne!(new_head, phantom_pos, "The bot should avoid moving into the Phantom's position.");
}
