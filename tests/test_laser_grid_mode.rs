use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn laser_grid_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::LaserGrid;
    game.reset();
    game.state = GameState::Playing;
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    // The Druid ignores ambient weather effects, keeping the assertions deterministic.
    game.stats.equipped_class = Some(HeroClass::Druid);
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

fn hostile_lasers(game: &Game) -> usize {
    game.lasers.iter().filter(|l| l.player != 1).count()
}

#[test]
fn test_laser_grid_reset_clears_state_and_weather() {
    let game = laser_grid_game(30, 30);

    assert_eq!(game.weather, Weather::Clear);
    assert_eq!(game.laser_volleys, 0);
    assert_eq!(game.laser_telegraph, None);
    assert_eq!(hostile_lasers(&game), 0);
}

#[test]
fn test_laser_grid_weather_never_breaks() {
    let mut game = laser_grid_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Laser Grid mode must force clear weather on every tick"
        );
    }
}

#[test]
fn test_laser_grid_telegraph_on_5th_tick() {
    let mut game = laser_grid_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);

    // update() increments tick_counter first: setting it to 4 makes the next
    // update run tick 5, which must telegraph the next beam.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.tick_counter, 5);
    let (is_row, idx) = game
        .laser_telegraph
        .expect("tick 5 must telegraph the next beam line");
    if is_row {
        assert!(
            !game.snake.body_map.keys().any(|p| p.y == idx),
            "a telegraphed row must never cross the snake body"
        );
    } else {
        assert!(
            !game.snake.body_map.keys().any(|p| p.x == idx),
            "a telegraphed column must never cross the snake body"
        );
    }
    assert_eq!(game.laser_volleys, 0, "telegraphing must not fire a beam yet");
}

#[test]
fn test_laser_grid_beam_fires_on_6th_tick() {
    let mut game = laser_grid_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);

    game.tick_counter = 4;
    game.update(); // tick 5: telegraphs a random fair line
    assert!(game.laser_telegraph.is_some());

    // Force a deterministic fair telegraph: row 20 is far from the snake
    game.laser_telegraph = Some((true, 20));
    game.tick_counter = 5;
    game.update(); // tick 6: fires the telegraphed beam

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.laser_telegraph, None, "the telegraph is consumed on fire");
    assert_eq!(game.laser_volleys, 1, "a fired beam must be counted");
    assert_eq!(hostile_lasers(&game), 1, "a hostile beam laser must be on the board");
    let beam = game
        .lasers
        .iter()
        .find(|l| l.player != 1)
        .expect("beam laser must exist");
    assert_eq!(beam.position.y, 20, "a row beam must travel along its row");
    assert!(
        matches!(beam.direction, Direction::Left | Direction::Right),
        "a row beam must travel horizontally"
    );
}

#[test]
fn test_laser_grid_unfair_beam_fizzles() {
    let mut game = laser_grid_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);

    game.tick_counter = 4;
    game.update(); // tick 5: telegraph

    // Force a telegraph onto the snake's own row: the fire must fizzle
    game.laser_telegraph = Some((true, 5));
    game.tick_counter = 5;
    game.update(); // tick 6: fire attempt

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.laser_telegraph, None);
    assert_eq!(
        game.laser_volleys, 0,
        "a beam onto a snake-occupied line must never fire"
    );
    assert_eq!(hostile_lasers(&game), 0);
}

#[test]
fn test_laser_grid_recalibrates_at_tick_200() {
    let mut game = laser_grid_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;
    game.lasers.push(snake_game::game::Laser {
        position: Point { x: 10, y: 20 },
        direction: Direction::Right,
        player: 2,
    });
    game.laser_telegraph = Some((false, 15));
    game.laser_volleys = 7;
    game.tick_counter = 199;

    game.update(); // tick 200: recalibration

    assert_eq!(game.tick_counter, 200);
    assert_eq!(
        hostile_lasers(&game),
        0,
        "recalibration must destroy all active beams"
    );
    assert_eq!(game.laser_telegraph, None, "recalibration clears the telegraph");
    assert_eq!(game.laser_volleys, 7, "the volley counter is not reset by recalibration");
}

#[test]
fn test_laser_grid_fairness_over_many_ticks() {
    let mut game = laser_grid_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..150 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(game.weather, Weather::Clear);
        // A fired beam must never share a line with a snake at fire time; the
        // telegraph is always picked on a line free of snake bodies.
        if let Some((is_row, idx)) = game.laser_telegraph {
            let crosses = game.snake.body_map.keys().any(|p| {
                if is_row { p.y == idx } else { p.x == idx }
            });
            assert!(
                !crosses,
                "the telegraphed line must never cross the snake body (line: {idx})"
            );
        }
    }

    assert!(
        game.lives > 0 && game.state == GameState::Playing,
        "Autopilot should survive the laser grid (lives: {}, volleys: {}, score: {})",
        game.lives,
        game.laser_volleys,
        game.score
    );
    assert!(
        game.laser_volleys >= 10,
        "expected at least 10 beam volleys over 150 ticks, got {}",
        game.laser_volleys
    );
}

#[test]
fn test_laser_grid_volleys_persist_through_save_load() {
    let file_path = format!("laser_grid_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = laser_grid_game(20, 20);
    game.laser_volleys = 11;
    game.laser_telegraph = Some((true, 7));
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"laser_volleys\":11"),
        "the save file must contain the volley counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::LaserGrid);
    assert_eq!(loaded.laser_volleys, 11);
    assert_eq!(loaded.laser_telegraph, Some((true, 7)));
    let _ = std::fs::remove_file("savegame.json");
}
