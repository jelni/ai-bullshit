use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn singularity_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Singularity;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing {
        if game.lives == 0 {
            return;
        }
        game.state = GameState::Playing;
    }
}

fn place_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.push_back(dir);
}

#[test]
fn test_singularity_eating_food_feeds_the_void() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 25, y: 25 });
    game.food = Point { x: 10, y: 9 };
    // Obstacles and a mine adjacent to the food cell
    game.obstacles.insert(Point { x: 9, y: 9 });
    game.obstacles.insert(Point { x: 11, y: 9 });
    game.mines.insert(Point { x: 10, y: 8 });
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.state = GameState::Playing;
    game.tick_counter = 0; // next update runs tick 1: no pulse, no drift

    game.update();

    assert_eq!(game.snake.head(), Point { x: 10, y: 9 }, "the snake must eat the food");
    assert!(
        game.singularity_shield == Game::SINGULARITY_SHIELD_TICKS,
        "eating food must still grant the gravity shield"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 9, y: 9 })
            && !game.obstacles.contains(&Point { x: 11, y: 9 }),
        "eating food must swallow the obstacles adjacent to the eaten cell"
    );
    assert!(
        !game.mines.contains(&Point { x: 10, y: 8 }),
        "eating food must swallow the mines adjacent to the eaten cell"
    );
}

#[test]
fn test_singularity_hole_never_despawns_during_play() {
    let mut game = singularity_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert!(
            game.black_hole.is_some(),
            "the singularity must never despawn in Singularity mode"
        );
    }
}

#[test]
fn test_singularity_pulse_never_unfair() {
    let mut game = singularity_game(24, 24);
    game.obstacles.clear();
    game.auto_pilot = true;

    for _ in 0..240 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        let head = game.snake.head();
        assert!(
            head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
            "a pulse must never pull a snake out of bounds"
        );
        assert!(
            !game.obstacles.contains(&head),
            "a pulse must never pull a snake into an obstacle"
        );
        assert_ne!(
            game.black_hole,
            Some(head),
            "a pulse must never pull a snake onto the hole"
        );
    }

    assert!(
        game.singularity_pulses >= 10,
        "expected at least 10 pulses over 240 ticks, got {}",
        game.singularity_pulses
    );
}

#[test]
fn test_singularity_autopilot_rides_out_the_void() {
    let mut game = singularity_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..300 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_ne!(
            game.black_hole,
            Some(game.snake.head()),
            "the bot must never end a tick inside the black hole"
        );
    }
}
