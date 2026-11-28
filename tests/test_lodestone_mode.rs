use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn lodestone_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Lodestone;
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

// Block the ambient portal/mine spawners so single-tick pulse tests are
// deterministic: an existing portal pair stops manage_portals from spawning,
// and a full mine rack (len >= 5) stops manage_mines from dropping new mines.
fn pin_ambient_spawners(game: &mut Game) {
    game.portals = Some((Point { x: 27, y: 27 }, Point { x: 28, y: 28 }));
    game.mines.insert(Point { x: 23, y: 1 });
    game.mines.insert(Point { x: 24, y: 1 });
    game.mines.insert(Point { x: 25, y: 1 });
    game.mines.insert(Point { x: 26, y: 1 });
    game.mines.insert(Point { x: 27, y: 1 });
}

// Surround the lodestone with obstacles so the scheduled drift cannot move it
// (every candidate cell is either off the playable ring or blocked), making
// the pulse destination deterministic.
fn pin_lodestone(game: &mut Game, m: Point) {
    game.lodestone = Some(m);
    game.obstacles.insert(Point { x: m.x + 1, y: m.y });
    game.obstacles.insert(Point {
        x: m.x.saturating_sub(1),
        y: m.y,
    });
    game.obstacles.insert(Point { x: m.x, y: m.y + 1 });
    game.obstacles.insert(Point {
        x: m.x,
        y: m.y.saturating_sub(1),
    });
}

fn park_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(dir);
}

#[test]
fn test_lodestone_reset_seeds_lodestone_fairly() {
    let game = lodestone_game(30, 30);

    assert_eq!(game.lodestone_pulses, 0);
    assert_eq!(game.lodestone_cooldown, 0);
    let m = game.lodestone.expect("Lodestone mode must seed a lodestone on reset");
    assert!(
        !game.obstacles.contains(&m),
        "the lodestone must never sit on an obstacle: {m:?}"
    );
    assert!(
        !game.snake.body_map.contains_key(&m),
        "the lodestone must never sit on the snake: {m:?}"
    );
    assert_ne!(m, game.food, "the lodestone must never sit on the food");
    assert!(
        m.x > 0 && m.x < game.width - 1 && m.y > 0 && m.y < game.height - 1,
        "the lodestone must spawn inside the playable ring: {m:?}"
    );
}

#[test]
fn test_lodestone_pulse_pulls_snake_one_cell_towards_lodestone() {
    let mut game = lodestone_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    pin_lodestone(&mut game, Point { x: 15, y: 10 });
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // update() runs tick 6: drift (blocked) + pulse
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.lodestone_pulses, 1, "the pulse must be counted on tick 6");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the snake moves to (11,10) and the pulse must drag it one cell towards the lodestone at (15,10) (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pulse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_lodestone_pulse_fizzles_when_destination_is_blocked() {
    let mut game = lodestone_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    pin_lodestone(&mut game, Point { x: 15, y: 10 });
    // The pull destination (12,10) is blocked, so the pulse must fizzle
    game.obstacles.insert(Point { x: 12, y: 10 });
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.lodestone_pulses, 1, "the pulse still fires and is counted");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with the pull destination blocked the snake must stay on its moved head (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fizzled pulse must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_lodestone_pulse_never_drags_snake_off_board() {
    let mut game = lodestone_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    // The lodestone sits on the lethal border ring; from x=0 no drift is possible
    // (every candidate is either off the playable ring or the snake's next cell)
    game.lodestone = Some(Point { x: 0, y: 10 });
    park_snake(&mut game, Point { x: 2, y: 10 }, Direction::Left);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.lodestone, Some(Point { x: 0, y: 10 }), "the lodestone must not drift");
    assert_eq!(
        game.snake.head(),
        Point { x: 1, y: 10 },
        "the pulse must never drag the snake out of bounds (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.lodestone_pulses, 1);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the border pull must fizzle, not kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_lodestone_pulse_never_lands_on_a_hazard() {
    let mut game = lodestone_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    pin_lodestone(&mut game, Point { x: 15, y: 10 });
    // A mine blocks the pull destination (12,10)
    game.mines.insert(Point { x: 12, y: 10 });
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the pulse must skip the mined cell (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the pulse must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_lodestone_eating_food_shorts_out_the_pulse() {
    let mut game = lodestone_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    pin_lodestone(&mut game, Point { x: 20, y: 20 });
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;

    // Tick 1: the snake eats the food at (11,10)
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.lodestone_cooldown,
        Game::LODESTONE_SHORT_TICKS,
        "eating food must short out the lodestone for LODESTONE_SHORT_TICKS"
    );

    // Tick 6: the cooldown ticks down to 1, so no pulse may fire
    game.lodestone_cooldown = 2;
    park_snake(&mut game, Point { x: 10, y: 12 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.lodestone_pulses, 0,
        "no pulse may fire while the lodestone is shorted out"
    );
    assert_eq!(game.lodestone_cooldown, 1, "the cooldown must count down every tick");

    // Tick 12: the cooldown has reached 0, so the pulse fires again
    game.lodestone_cooldown = 1;
    park_snake(&mut game, Point { x: 10, y: 14 }, Direction::Right);
    game.tick_counter = 11;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.lodestone_cooldown, 0);
    assert_eq!(
        game.lodestone_pulses, 1,
        "the pulse must fire again once the short-out has expired"
    );
}

#[test]
fn test_lodestone_pulses_fire_every_6_ticks_and_stay_fair() {
    let mut game = lodestone_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);

    for tick in 0..60_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 20, y: 20 }, Direction::Right);
        game.tick_counter = tick;
        let pulses_before = game.lodestone_pulses;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        if (tick + 1).is_multiple_of(6) {
            assert!(
                game.lodestone_pulses > pulses_before,
                "a pulse must fire on tick {}",
                tick + 1
            );
        } else {
            assert_eq!(
                game.lodestone_pulses, pulses_before,
                "no pulse may fire on non-6th tick {}",
                tick + 1
            );
        }
        if let Some(m) = game.lodestone {
            assert!(
                !game.obstacles.contains(&m),
                "tick {tick}: the lodestone must not sit on an obstacle: {m:?}"
            );
            assert!(
                !game.snake.body_map.contains_key(&m),
                "tick {tick}: the lodestone must not sit on the snake: {m:?}"
            );
            assert_ne!(m, game.food, "tick {tick}: the lodestone must not sit on the food");
        } else {
            panic!("tick {tick}: the lodestone must stay on the board in Lodestone mode");
        }
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up and carry on
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        assert!(
            !game.obstacles.contains(&head),
            "tick {tick}: the pulse landed on an obstacle at {head:?}"
        );
        assert!(
            head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
            "tick {tick}: the pulse dragged the snake off the playable ring at {head:?}"
        );
    }

    assert_eq!(
        game.lodestone_pulses, 10,
        "ticks 6,12,...,60 must fire exactly 10 pulses"
    );
}

#[test]
fn test_lodestone_bot_survives_the_pulses() {
    let mut game = lodestone_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..200 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Lodestone mode (lives: {}, pulses: {}, score: {})",
        game.lives,
        game.lodestone_pulses,
        game.score
    );
}

#[test]
fn test_lodestone_persists_through_history_rewind() {
    let mut game = lodestone_game(30, 30);
    game.lodestone = Some(Point { x: 12, y: 12 });
    game.lodestone_pulses = 7;
    game.lodestone_cooldown = 3;
    game.save_history_state();
    game.lodestone = None;
    game.lodestone_pulses = 0;
    game.lodestone_cooldown = 0;

    game.rewind_time();

    assert_eq!(
        game.lodestone,
        Some(Point { x: 12, y: 12 }),
        "the lodestone must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.lodestone_pulses, 7,
        "lodestone_pulses must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.lodestone_cooldown, 3);
}

#[test]
fn test_lodestone_persists_in_save_file() {
    let mut game = lodestone_game(30, 30);
    game.lodestone = Some(Point { x: 12, y: 12 });
    game.lodestone_pulses = 9;
    game.lodestone_cooldown = 4;
    let path = std::env::temp_dir().join(format!(
        "lodestone_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"lodestone_pulses\":9")
            && json.contains("\"lodestone_cooldown\":4")
            && json.contains("\"lodestone\":{"),
        "SaveState JSON must contain the lodestone fields, got: {}",
        &json[..json.len().min(200)]
    );
}
