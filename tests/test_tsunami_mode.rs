use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn tsunami_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Tsunami;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    // The Druid ignores ambient weather effects, keeping the assertions deterministic.
    game.stats.equipped_class = Some(HeroClass::Druid);
    game
}

fn line_of(edge: u8, w: u16, h: u16, p: Point) -> i32 {
    match edge {
        0 => i32::from(p.y),
        1 => i32::from(h.saturating_sub(1)).saturating_sub(i32::from(p.y)),
        2 => i32::from(p.x),
        _ => i32::from(w.saturating_sub(1)).saturating_sub(i32::from(p.x)),
    }
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing {
        if game.lives == 0 {
            return;
        }
        game.state = GameState::Playing;
    }
}

#[test]
fn test_tsunami_reset_forces_rain_and_clears_wave() {
    let game = tsunami_game(30, 30);

    assert_eq!(game.weather, Weather::Rain);
    assert_eq!(game.tsunami_edge, Game::TSUNAMI_NO_EDGE);
    assert_eq!(game.tsunami_front, -1);
    assert_eq!(game.tsunami_depth, -1);
    assert_eq!(game.tsunami_warning, 0);
    assert_eq!(game.tsunami_waves, 0);
}

#[test]
fn test_tsunami_weather_never_breaks() {
    let mut game = tsunami_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Rain,
            "Tsunami mode must force rain weather on every tick"
        );
    }
}

#[test]
fn test_tsunami_wave_announced_every_period() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    // update() increments tick_counter first, so the next update runs tick 40,
    // which must announce a new wave from a random edge.
    game.tick_counter = Game::TSUNAMI_PERIOD - 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, Game::TSUNAMI_PERIOD);
    assert_eq!(game.tsunami_warning, Game::TSUNAMI_WARNING);
    assert!(game.tsunami_edge < 4, "edge must be a valid side");
    assert_eq!(game.tsunami_front, -1, "the wave starts only after the warning");
    assert!(
        game.tsunami_depth >= 1,
        "the wave depth must stop short of the snake (depth: {})",
        game.tsunami_depth
    );
}

#[test]
fn test_tsunami_wave_sweeps_line_and_smashes_obstacles() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 15, y: 15 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.obstacles.insert(Point { x: 3, y: 0 });
    game.obstacles.insert(Point { x: 7, y: 0 });
    game.obstacles.insert(Point { x: 5, y: 2 });
    game.food = Point { x: 4, y: 0 };
    game.tsunami_edge = 0; // from the top: line == y
    game.tsunami_warning = 1;
    game.tsunami_front = -1;
    game.tsunami_depth = 29;
    game.tick_counter = 5; // not a multiple of TSUNAMI_PERIOD
    game.state = GameState::Playing;

    game.update();

    assert_eq!(game.tsunami_warning, 0);
    assert_eq!(
        game.tsunami_front, 1,
        "the first line is swept as soon as the warning ends"
    );
    assert!(!game.obstacles.contains(&Point { x: 3, y: 0 }), "wave smashes obstacles on the swept line");
    assert!(!game.obstacles.contains(&Point { x: 7, y: 0 }));
    assert!(
        game.obstacles.contains(&Point { x: 5, y: 2 }),
        "other lines are untouched"
    );
    assert_ne!(
        game.food,
        Point { x: 4, y: 0 },
        "food on the swept line must be washed away to a fair cell"
    );
    assert!(!game.obstacles.contains(&game.food));
    assert!(!game.snake.body_map.contains_key(&game.food));
}

#[test]
fn test_tsunami_swept_line_kills_snake() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    // Head at (3,15) moving up stays on x=3; a wave from the left (edge 2:
    // line == x) sweeping line x=3 catches the head (head-only kill, like Storm).
    let mut snake = Snake::new(Point { x: 3, y: 15 });
    snake.direction = Direction::Up;
    snake.direction_queue.push_back(Direction::Up);
    game.snake = snake;
    game.tsunami_edge = 2;
    game.tsunami_warning = 0;
    game.tsunami_front = 3;
    game.tsunami_depth = 29;
    game.tick_counter = 5;
    game.state = GameState::Playing;
    let lives_before = game.lives;

    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "a snake whose head is on the swept line must die (lives before: {lives_before})"
    );
}

#[test]
fn test_tsunami_threatened_cells_rejected_in_verified_safety() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 15, y: 15 });
    game.tsunami_edge = 0; // from the top: line == y
    game.tsunami_depth = 20;

    // Warning phase: every line within the wave's depth is rejected at any step
    game.tsunami_warning = Game::TSUNAMI_WARNING; // 8
    game.tsunami_front = -1;
    let p = Point { x: 5, y: 3 }; // line 3 <= depth: inside the incoming wave
    assert!(!game.is_safe_final_p(p, 1, 1), "danger-zone cells are unsafe during the warning");
    assert!(!game.is_safe_final_p(p, 11, 1));
    assert!(!game.is_safe_final_p(p, 12, 1));
    let safe_side = Point { x: 5, y: 25 }; // line 25 > depth: beyond the wave, always safe
    assert!(game.is_safe_final_p(safe_side, 1, 1));
    assert!(game.is_safe_final_p(safe_side, 12, 1));

    // Active wave: lines at or ahead of the front are rejected, lines already
    // behind the front are safe at any step
    game.tsunami_warning = 0;
    game.tsunami_front = 2;
    let q = Point { x: 5, y: 5 }; // line 5 >= front 2: not yet swept
    assert!(!game.is_safe_final_p(q, 1, 1));
    assert!(!game.is_safe_final_p(q, 4, 1));
    assert!(!game.is_safe_final_p(q, 9, 1));
    let front_line = Point { x: 5, y: 2 }; // line 2 == front
    assert!(!game.is_safe_final_p(front_line, 1, 1));
    let behind = Point { x: 5, y: 1 }; // line 1 < front: already swept, safe at any step
    assert!(game.is_safe_final_p(behind, 1, 1));
    assert!(game.is_safe_final_p(behind, 9, 1));
}

#[test]
fn test_tsunami_path_is_safe_rejects_unswept_danger_lines() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.tsunami_edge = 0; // from the top: line == y
    game.tsunami_warning = 0;
    game.tsunami_front = 6; // line 6 is swept at step 1
    game.tsunami_depth = 29;

    // Stepping onto the imminent front line is rejected
    assert!(
        !game.path_is_safe(Point { x: 5, y: 5 }, &[Point { x: 5, y: 6 }], &[], 1),
        "path onto the line the wave sweeps at that step must be rejected"
    );
    // Stepping behind the front (already swept) is fine
    assert!(
        game.path_is_safe(Point { x: 5, y: 5 }, &[Point { x: 5, y: 4 }], &[], 1),
        "path behind the wave front must be accepted"
    );
}

#[test]
fn test_tsunami_autopilot_survives_repeated_waves() {
    let mut game = tsunami_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;
    let mut wave_deaths = 0_u32;
    let mut last_lives = game.lives;

    for _ in 0..400 {
        // Predict the line manage_tsunami sweeps at the end of this update: line 0
        // when the warning runs out, otherwise the current front line.
        let swept = if game.tsunami_warning == 1 {
            Some(0_i32)
        } else if game.tsunami_warning == 0 && game.tsunami_front >= 0 {
            Some(game.tsunami_front)
        } else {
            None
        };
        // The post-move head is one of the four neighbors of the current head
        // (the wave kill is head-only, like Storm's lightning column).
        let head = game.snake.head();
        let neighbors = [
            Point { x: head.x, y: head.y.saturating_sub(1) },
            Point { x: head.x, y: head.y.saturating_add(1) },
            Point { x: head.x.saturating_sub(1), y: head.y },
            Point { x: head.x.saturating_add(1), y: head.y },
        ];
        let (edge, w, h) = (game.tsunami_edge, game.width, game.height);
        let head_on_swept_line = swept
            .is_some_and(|l| neighbors.iter().any(|p| line_of(edge, w, h, *p) == l));

        game.update();
        if head_on_swept_line && game.lives < last_lives {
            wave_deaths += 1;
        }
        last_lives = game.lives;
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.tsunami_waves >= 3,
        "several full waves must have crossed the board (waves: {})",
        game.tsunami_waves
    );
    assert_eq!(
        wave_deaths, 0,
        "the telegraphed, depth-capped wave must never catch the autopilot (waves: {})",
        game.tsunami_waves
    );
}
