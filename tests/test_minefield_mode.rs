use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn minefield_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Minefield;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while testing
    game
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing && game.lives > 0 {
        game.state = GameState::Playing;
    }
}

#[test]
fn test_minefield_reset_seeds_initial_mines_and_clear_weather() {
    let game = minefield_game(40, 30);

    assert_eq!(game.weather, Weather::Clear);
    assert_eq!(game.mine_drops, 0);
    // A few mines (1/80 of the board) are already buried from the start
    let expected = (u32::from(game.width) * u32::from(game.height)) / 80;
    assert!(
        game.mines.len() > 0,
        "Minefield mode must seed initial mines on reset"
    );
    assert!(u32::try_from(game.mines.len()).unwrap_or(u32::MAX) <= expected);
    for mine in &game.mines {
        assert!(
            !game.snake.body_map.contains_key(mine),
            "mines must never seed on the snake"
        );
        assert!(!game.obstacles.contains(mine), "mines are not obstacles");
    }
}

#[test]
fn test_minefield_weather_stays_clear() {
    let mut game = minefield_game(30, 30);
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
            "Minefield mode must keep clear skies so the mines stay visible"
        );
    }
}

#[test]
fn test_minefield_scheduled_drop_every_6_ticks() {
    let mut game = minefield_game(30, 30);
    game.obstacles.clear();
    game.mines.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled mine drop.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let drops_before = game.mine_drops;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert_eq!(
            game.mine_drops,
            drops_before + 1,
            "a scheduled mine drop must fire on tick {}",
            tick + 1
        );
    }
    // A non-multiple-of-6 tick must not drop.
    game.state = GameState::Playing;
    game.tick_counter = 36;
    let drops_before = game.mine_drops;
    game.update();
    assert_eq!(
        game.mine_drops, drops_before,
        "no mine drop may fire on tick 37"
    );
}

#[test]
fn test_minefield_drops_never_land_on_snake_and_respect_cap() {
    let mut game = minefield_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.power_up = None; // keep bomb/nuke power-ups out of the invariant check
        game.black_hole = None; // keep the ambient hole out of the invariant check
        game.meteors.clear(); // keep meteor mine-clearing out of the invariant check
        let lives_before = game.lives;
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        if game.lives != lives_before {
            continue; // a death tick returns early, before the mine drop
        }
        for mine in &game.mines {
            assert!(
                !game.snake.body_map.contains_key(mine),
                "a mine must never drop on the snake"
            );
        }
        let cap = (u32::from(game.width) * u32::from(game.height)) / 10;
        assert!(u32::try_from(game.mines.len()).unwrap_or(u32::MAX) <= cap);
    }
}

#[test]
fn test_minefield_swept_at_tick_200() {
    let mut game = minefield_game(40, 30);
    game.auto_pilot = true;
    game.tick_counter = 199; // next update runs tick 200 -> sweep
    game.state = GameState::Playing;
    assert!(!game.mines.is_empty());

    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.mines.is_empty(),
        "all mines must be swept at tick 200 of the cycle"
    );
}

#[test]
fn test_minefield_food_eaten_defuses_adjacent_mines() {
    let mut game = minefield_game(30, 30);
    game.obstacles.clear();
    game.mines.clear();
    // Bury mines adjacent to the food cell (10,10)
    for p in [
        Point { x: 10, y: 9 },
        Point { x: 11, y: 10 },
        Point { x: 10, y: 11 },
    ] {
        game.mines.insert(p);
    }
    game.food = Point { x: 10, y: 10 };
    // Snake heads onto the food from the left (body: (9,10),(9,11),(9,12))
    game.snake = Snake::new(Point { x: 9, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.state = GameState::Playing;
    game.tick_counter = 1; // tick 2: no scheduled drop

    game.update();

    assert_eq!(game.snake.head(), Point { x: 10, y: 10 }, "the snake must eat the food");
    assert!(
        game.mines.is_empty(),
        "eating food must defuse all adjacent mines, got {:?}",
        game.mines
    );
}

#[test]
fn test_minefield_snake_dies_entering_mine() {
    let mut game = minefield_game(30, 30);
    game.obstacles.clear();
    game.mines.clear();
    game.mines.insert(Point { x: 6, y: 5 });
    game.food = Point { x: 25, y: 25 };
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right); // moves onto (6,5) = mine
    game.state = GameState::Playing;
    game.tick_counter = 1;
    let lives_before = game.lives;

    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "entering a mine must kill the snake"
    );
}
