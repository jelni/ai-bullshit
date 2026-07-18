use snake_game::*;

#[test]
fn test_turf_war_scoring() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::TurfWar;
    game.match_time = 2; // Trigger scoring logic in 2 updates

    // Place snake and players/bots
    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });

    // Player 2
    game.player2 = Some(snake::Snake::new(snake::Point {
        x: 10,
        y: 10,
    }));

    // 2 Bots
    game.bots.clear();
    game.bots_autopilot_paths.clear();
    game.bots.push(snake::Snake::new(snake::Point {
        x: 15,
        y: 15,
    }));
    game.bots.push(snake::Snake::new(snake::Point {
        x: 15,
        y: 16,
    }));

    game.painted_tiles.clear();

    // Ensure bots have autopilot paths
    game.bots_autopilot_paths.push(Vec::new());
    game.bots_autopilot_paths.push(Vec::new());

    // Manually paint some tiles to ensure predictable scoring
    game.painted_tiles.insert(
        snake::Point {
            x: 1,
            y: 1,
        },
        1,
    ); // P1
    game.painted_tiles.insert(
        snake::Point {
            x: 1,
            y: 2,
        },
        1,
    ); // P1

    game.painted_tiles.insert(
        snake::Point {
            x: 2,
            y: 1,
        },
        2,
    ); // P2

    game.painted_tiles.insert(
        snake::Point {
            x: 3,
            y: 1,
        },
        3,
    ); // Bot 1
    game.painted_tiles.insert(
        snake::Point {
            x: 3,
            y: 2,
        },
        3,
    ); // Bot 1
    game.painted_tiles.insert(
        snake::Point {
            x: 3,
            y: 3,
        },
        3,
    ); // Bot 1

    game.painted_tiles.insert(
        snake::Point {
            x: 4,
            y: 1,
        },
        4,
    ); // Bot 2
    game.painted_tiles.insert(
        snake::Point {
            x: 4,
            y: 2,
        },
        4,
    ); // Bot 2
    game.painted_tiles.insert(
        snake::Point {
            x: 4,
            y: 3,
        },
        4,
    ); // Bot 2
    game.painted_tiles.insert(
        snake::Point {
            x: 4,
            y: 4,
        },
        4,
    ); // Bot 2

    // Make them survive so they keep their painted tiles and scores aren't overridden.
    game.state = game::GameState::Playing;

    // 1st update: match_time becomes 1, painted_tiles gets updated with their current heads.
    game.update();

    // Check painted_tiles
    assert_eq!(
        game.painted_tiles.get(&snake::Point {
            x: 5,
            y: 5
        }),
        Some(&1)
    );
    assert_eq!(
        game.painted_tiles.get(&snake::Point {
            x: 10,
            y: 10
        }),
        Some(&2)
    );

    // 2nd update: match_time becomes 0 -> handles death, sets death message.
    game.update();

    assert!(game.just_died);
    assert_eq!(game.death_message, "Time's Up! Bot 1 Wins!");
}

#[test]
fn test_turf_war_painting_and_survival() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::TurfWar;
    game.match_time = 100;

    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Right;

    game.player2 = None;
    game.bots.clear();
    game.bots_autopilot_paths.clear();

    game.state = game::GameState::Playing;

    game.update();

    assert_eq!(
        game.painted_tiles.get(&snake::Point {
            x: 5,
            y: 5
        }),
        Some(&1)
    );
    assert_eq!(
        game.painted_tiles.get(&snake::Point {
            x: 6,
            y: 5
        }),
        Some(&1)
    ); // Moved right
}
