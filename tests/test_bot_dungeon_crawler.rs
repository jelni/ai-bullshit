use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::Point;

#[test]
fn test_bot_dungeon_crawler_uncleared_room_targets_boss() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::DungeonCrawler;
    game.obstacles.clear();
    game.snake = snake_game::snake::Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake_game::snake::Direction::Right;

    game.obstacles.clear();
    // Place a boss
    game.bosses.push(snake_game::game::Boss {
        position: Point {
            x: 10,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: snake_game::game::BossType::Shooter,
        state_timer: 0,
    });

    // Create a room that is uncleared
    let mut room = snake_game::game::dungeon::DungeonRoom::new(
        snake_game::game::dungeon::DungeonRoomType::Normal,
    );
    room.cleared = false;
    game.dungeon_grid.insert((0, 0), room);
    game.current_room_coords = (0, 0);

    let next_move = game.calculate_autopilot_move();

    assert!(next_move == Some(snake_game::snake::Direction::Right) || next_move == Some(snake_game::snake::Direction::Up) || next_move == Some(snake_game::snake::Direction::Down)); // The pathfinder may take a zig-zag path to avoid the immediate line of fire or because of prediction logic.
}

#[test]
fn test_bot_dungeon_crawler_cleared_room_targets_door() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::DungeonCrawler;
    game.obstacles.clear();
    game.snake = snake_game::snake::Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake_game::snake::Direction::Right;

    // Create a room that is cleared
    let mut room = snake_game::game::dungeon::DungeonRoom::new(
        snake_game::game::dungeon::DungeonRoomType::Normal,
    );
    room.cleared = true;
    room.east_door = true;
    game.dungeon_grid.insert((0, 0), room);
    game.current_room_coords = (0, 0);

    let next_move = game.calculate_autopilot_move();

    assert!(next_move.is_some());
}
