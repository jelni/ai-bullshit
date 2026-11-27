use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn steamroller_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Steamroller;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_steamroller_reset_clears_state() {
    let game = steamroller_game(30, 30);

    assert!(game.press_cells.is_empty());
    assert_eq!(game.press_pos, 1);
    assert_eq!(game.press_dir, 1);
    assert_eq!(game.press_moves, 0);
}

#[test]
fn test_steamroller_advances_on_schedule() {
    let mut game = steamroller_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first; early in the 250-tick cycle the
    // roller moves one column every 4th tick, so ticks 4, 8, 12, ... must move it.
    for tick in [3_u32, 7, 11, 19, 27, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let moves_before = game.press_moves;
        let pos_before = game.press_pos;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert_eq!(
            game.press_moves,
            moves_before + 1,
            "the roller must advance on tick {}",
            tick + 1
        );
        assert_eq!(
            game.press_pos,
            pos_before + 1,
            "the roller must sweep right from its home column (tick {})",
            tick + 1
        );
        assert!(
            !game.press_cells.is_empty(),
            "advancing must erect roller cells on tick {}",
            tick + 1
        );
        for cell in &game.press_cells {
            assert!(game.obstacles.contains(cell), "roller cells must be mirrored into obstacles");
            assert_eq!(
                cell.x, game.press_pos,
                "every roller cell must sit on the roller's current column"
            );
        }
    }
    assert_eq!(game.press_moves, 6);
    assert_eq!(game.press_pos, 7);
}

#[test]
fn test_steamroller_does_not_move_off_schedule() {
    let mut game = steamroller_game(30, 30);
    game.auto_pilot = true;

    // Ticks 5, 6, 7 are not multiples of the early-cycle interval (4): no move.
    for tick in [4_u32, 5, 6] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let moves_before = game.press_moves;
        let pos_before = game.press_pos;
        game.update();
        assert_eq!(game.press_moves, moves_before, "no move on tick {}", tick + 1);
        assert_eq!(game.press_pos, pos_before, "no move on tick {}", tick + 1);
    }
}

#[test]
fn test_steamroller_never_walls_snake_or_next_cell() {
    let mut game = steamroller_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.press_cells.clear();
    game.press_pos = 1;
    game.press_dir = 1;
    game.press_moves = 0;
    game.snake = Snake::new(Point {
        x: 1,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 15,
        y: 15,
    };
    game.auto_pilot = false;

    // Tick 4 moves the roller to column 2, which contains the cell (2, 5) the
    // snake is about to enter: that cell must stay open.
    game.tick_counter = 3;
    game.update();

    assert_eq!(game.press_moves, 1);
    assert_eq!(game.press_pos, 2);
    let next = Point {
        x: 2,
        y: 5,
    };
    assert!(
        !game.press_cells.contains(&next),
        "the roller must never wall the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(!game.press_cells.contains(seg), "the roller must never wall the snake's body");
    }
    assert!(!game.press_cells.contains(&game.food), "the roller must never wall the food");
}

#[test]
fn test_steamroller_fairness_over_many_moves() {
    let mut game = steamroller_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point {
        x: 25,
        y: 25,
    };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled advance, so when the roller reaches column 9 it must never
    // wall its next cell (9, 15) or its body.
    let next = Point {
        x: 9,
        y: 15,
    };
    for k in 1..=8_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point {
            x: 8,
            y: 15,
        });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 4 * k - 1;
        let moves_before = game.press_moves;
        game.update();
        if game.press_moves > moves_before {
            assert!(
                !game.press_cells.contains(&next),
                "the roller walled the cell the snake was about to enter ({next:?})"
            );
            for seg in &game.snake.body {
                assert!(
                    !game.press_cells.contains(seg),
                    "the roller walled the snake's body at {seg:?}"
                );
            }
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the roller (moves: {})",
        game.press_moves
    );
    assert!(
        game.press_moves >= 8,
        "expected the roller to advance at least 8 times, got {}",
        game.press_moves
    );
}

#[test]
fn test_steamroller_ping_pongs_at_edges() {
    let mut game = steamroller_game(12, 12);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    game.food = Point {
        x: 6,
        y: 6,
    };

    // Playable columns are 1..=width-2 = 1..=10. Park the roller at the right
    // edge heading right: it must bounce back left instead of leaving the board.
    game.press_pos = 10;
    game.press_dir = 1;
    game.tick_counter = 3;
    game.update();
    assert_eq!(game.tick_counter, 4);
    assert_eq!(game.press_dir, -1, "the roller must bounce off the right edge");
    assert_eq!(game.press_pos, 9);

    // Park it at the left edge heading left: it must bounce back right.
    game.press_pos = 1;
    game.press_dir = -1;
    game.tick_counter = 7;
    game.update();
    assert_eq!(game.tick_counter, 8);
    assert_eq!(game.press_dir, 1, "the roller must bounce off the left edge");
    assert_eq!(game.press_pos, 2);
}

#[test]
fn test_steamroller_resets_at_tick_200() {
    let mut game = steamroller_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Advance the roller a few columns first.
    for tick in [3_u32, 7, 11] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.press_cells.is_empty());
    assert!(game.press_pos > 1);
    let moves_before = game.press_moves;

    // Tick 200 of the 250-tick cycle sends the roller home.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(game.press_cells.is_empty(), "all roller cells must be vacated at tick 200");
    assert_eq!(game.press_pos, 1);
    assert_eq!(game.press_dir, 1);
    assert!(game.press_moves >= moves_before, "the move counter keeps counting through a reset");
    for cell in &game.obstacles {
        assert_ne!(cell.x, 1, "no roller column may remain at home after reset");
    }
}

#[test]
fn test_steamroller_speeds_up_late_in_cycle() {
    let mut game = steamroller_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.auto_pilot = true;

    // From tick 160 on the interval is 1, so every tick advances the roller.
    game.state = GameState::Playing;
    game.tick_counter = 159;
    let moves_before = game.press_moves;
    game.update();
    assert_eq!(game.tick_counter, 160);
    assert_eq!(
        game.press_moves,
        moves_before + 1,
        "the roller must advance on every tick from 160 on"
    );
    game.state = GameState::Playing;
    game.tick_counter = 160;
    let moves_before = game.press_moves;
    game.update();
    assert_eq!(game.tick_counter, 161);
    assert_eq!(
        game.press_moves,
        moves_before + 1,
        "the roller must advance on every tick from 160 on"
    );
}

#[test]
fn test_steamroller_eating_food_breaches_adjacent_roller() {
    let mut game = steamroller_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.press_cells.clear();
    game.press_pos = 1;
    game.press_dir = 1;
    game.press_moves = 0;
    let wall = Point {
        x: 7,
        y: 5,
    };
    game.press_cells.insert(wall);
    game.obstacles.insert(wall);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 6,
        y: 5,
    };
    game.auto_pilot = false;

    // Tick 5 is not on the early-cycle schedule (interval 4), so no advance interferes.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.press_cells.contains(&wall),
        "Eating food in Steamroller mode should breach the adjacent roller cell"
    );
    assert!(!game.obstacles.contains(&wall), "The breached roller cell must also leave obstacles");
}

#[test]
fn test_steamroller_bot_survives_the_roller() {
    let mut game = steamroller_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = GameState::Playing;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive the roller (lives: {}, moves: {}, score: {})",
        game.lives,
        game.press_moves,
        game.score
    );
    assert!(
        game.press_moves >= 10,
        "expected the roller to advance many times over 120 ticks, got {}",
        game.press_moves
    );
    assert!(
        game.press_pos >= 1 && game.press_pos <= game.width - 2,
        "the roller must stay within the playable columns, got {}",
        game.press_pos
    );
}

#[test]
fn test_steamroller_persists_through_history_rewind() {
    let mut game = steamroller_game(30, 30);
    game.press_moves = 7;
    game.press_pos = 5;
    game.press_dir = -1;
    game.save_history_state();
    game.press_moves = 0;
    game.press_pos = 1;
    game.press_dir = 1;

    game.rewind_time();

    assert_eq!(game.press_moves, 7, "press_moves must survive a time-rewind through HistoryState");
    assert_eq!(game.press_pos, 5, "press_pos must survive a time-rewind through HistoryState");
    assert_eq!(game.press_dir, -1, "press_dir must survive a time-rewind through HistoryState");
}

#[test]
fn test_steamroller_persists_in_save_file() {
    let mut game = steamroller_game(30, 30);
    game.press_moves = 9;
    game.press_pos = 4;
    game.press_dir = 1;
    let path = std::env::temp_dir().join(format!(
        "steamroller_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"press_moves\":9") && json.contains("\"press_pos\":4"),
        "SaveState JSON must contain the roller counters, got: {}",
        &json[..json.len().min(200)]
    );
}
