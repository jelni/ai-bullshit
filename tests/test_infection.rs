use snake_game::game::{Difficulty, Game, GameMode, GameState, SaveState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn infection_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Infection;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn solo_bot_game() -> Game {
    let mut game = infection_game(30, 30);
    game.obstacles.clear();
    // One bot, far away, with a pre-queued move so the autopilot stays out of the way
    let mut bot = Snake::new(Point { x: 20, y: 20 });
    bot.direction = Direction::Left;
    bot.direction_queue.push_back(Direction::Left);
    game.bots = vec![bot];
    game.bots_autopilot_paths = vec![Vec::new()];
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 5 };
    game
}

#[test]
fn test_infection_reset_spawns_bots_and_patient_zero() {
    let game = infection_game(30, 30);

    assert_eq!(game.bots.len(), 3, "Infection should spawn 3 arena bots");
    assert_eq!(
        game.infected_ids.len(),
        1,
        "exactly one participant must start as Patient Zero"
    );
    let patient_zero = game.infected_ids[0];
    assert!(
        patient_zero >= 3 && usize::from(patient_zero - 3) < game.bots.len(),
        "Patient Zero must be one of the arena bots (id >= 3), got {patient_zero}"
    );
    assert_eq!(game.infection_spreads, 0);
}

#[test]
fn test_infection_spreads_on_adjacency_between_bots() {
    let mut game = solo_bot_game();
    // Infected bot 3 moves (10,10) -> (11,10); healthy bot 4 moves (12,11) -> (12,10)
    let mut infected = Snake::new(Point { x: 10, y: 10 });
    infected.direction = Direction::Right;
    infected.direction_queue.push_back(Direction::Right);
    let mut healthy = Snake::new(Point { x: 12, y: 11 });
    healthy.direction = Direction::Up;
    healthy.direction_queue.push_back(Direction::Up);
    game.bots = vec![infected, healthy];
    game.bots_autopilot_paths = vec![Vec::new(), Vec::new()];
    game.infected_ids = vec![3];
    game.infection_spreads = 0;

    game.state = GameState::Playing;
    game.update();

    // Post-move heads: (11,10) and (12,10) -> Manhattan distance 1 -> spread
    assert!(
        game.infected_ids.contains(&4),
        "the healthy bot adjacent to an infected head must become infected"
    );
    assert_eq!(game.infection_spreads, 1, "a spread must be counted");
    assert!(
        game.infected_ids.contains(&3),
        "the original infected bot stays infected"
    );
}

#[test]
fn test_infection_no_spread_without_adjacency() {
    let mut game = solo_bot_game();
    // Add a second, healthy bot so "all bots infected" cannot trigger a win
    let mut healthy = Snake::new(Point { x: 25, y: 20 });
    healthy.direction = Direction::Left;
    healthy.direction_queue.push_back(Direction::Left);
    game.bots.push(healthy);
    game.bots_autopilot_paths.push(Vec::new());
    game.infected_ids = vec![3]; // the bot at (20,20) is infected
    game.infection_spreads = 0;

    game.state = GameState::Playing;
    game.update();

    // Player moves to (6,5), bots move to (19,20) and (24,20): far apart, no spread
    assert_eq!(game.infection_spreads, 0);
    assert_eq!(game.infected_ids, vec![3]);
    assert!(
        game.state == GameState::Playing,
        "nobody gets infected when heads are not adjacent"
    );
}

#[test]
fn test_infection_spread_to_player_costs_a_life() {
    let mut game = solo_bot_game();
    // Infected bot moves (7,6) -> (6,6); player moves (5,5) -> (6,5): Manhattan 1
    let mut bot = Snake::new(Point { x: 7, y: 6 });
    bot.direction = Direction::Left;
    bot.direction_queue.push_back(Direction::Left);
    game.bots = vec![bot];
    game.bots_autopilot_paths = vec![Vec::new()];
    game.infected_ids = vec![3];
    let lives_before = game.lives;

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "the player catching the infection must cost a life"
    );
    assert_eq!(game.infection_spreads, 1);
    assert!(
        !game.infected_ids.contains(&1),
        "the player must not remain in infected_ids after dying (they respawn healthy)"
    );
}

#[test]
fn test_infection_clock_expiry_with_healthy_player_wins() {
    let mut game = solo_bot_game();
    game.infected_ids = vec![3]; // bot far away at (20,20)
    game.match_time = 1;

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.match_time, 0, "the match clock must tick down");
    assert!(
        game.state == GameState::GameWon,
        "surviving uninfected until the clock runs out must win the match"
    );
}

#[test]
fn test_infection_all_bots_infected_player_healthy_wins() {
    let mut game = infection_game(30, 30);
    game.obstacles.clear();
    let ids: Vec<u8> = (0..game.bots.len())
        .map(|i| u8::try_from(i).unwrap_or(255).saturating_add(3))
        .collect();
    game.infected_ids = ids.clone();
    game.infection_spreads = 3;
    game.match_time = 500;
    // Keep the player far away from every bot head so no spread hits the player
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    for (i, bot) in game.bots.iter_mut().enumerate() {
        *bot = Snake::new(Point {
            x: 25,
            y: 10 + u16::try_from(i).unwrap_or(0),
        });
        bot.direction = Direction::Left;
        bot.direction_queue.push_back(Direction::Left);
    }
    game.food = Point { x: 15, y: 25 };

    game.state = GameState::Playing;
    game.update();

    assert!(
        game.state == GameState::GameWon,
        "with every bot infected and the player healthy, the player is the last healthy standing"
    );
}

#[test]
fn test_infection_state_persists_in_history_state() {
    let mut game = infection_game(30, 30);
    game.infected_ids = vec![3, 4];
    game.infection_spreads = 7;

    game.save_history_state();

    let last = game.history.back().expect("history should have one entry");
    assert_eq!(last.infected_ids, vec![3, 4]);
    assert_eq!(
        last.infection_spreads, 7,
        "spread counter must be stored in HistoryState for time-rewind consistency"
    );
}

#[test]
fn test_infection_state_persists_in_save_state() {
    let mut game = infection_game(30, 30);
    game.infected_ids = vec![3, 5];
    game.infection_spreads = 4;

    let path = std::env::temp_dir().join(format!(
        "snake_test_infection_save_{}.json",
        std::process::id()
    ));
    game.save_game_to_file(path.to_str().unwrap());
    let json = std::fs::read_to_string(&path).expect("save file must exist");
    let _ = std::fs::remove_file(&path);

    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    assert_eq!(state.mode, GameMode::Infection);
    assert_eq!(state.infected_ids, vec![3, 5]);
    assert_eq!(state.infection_spreads, 4);
}

#[test]
fn test_infection_autopilot_game_runs_cleanly() {
    let mut game = infection_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != GameState::Playing {
            break;
        }
        let alive_ids: Vec<u8> = std::iter::once(1_u8)
            .chain(game.player2.is_some().then_some(2))
            .chain((0..game.bots.len()).map(|i| u8::try_from(i).unwrap_or(255).saturating_add(3)))
            .collect();
        for id in &game.infected_ids {
            assert!(
                alive_ids.contains(id),
                "every infected id must reference an alive participant, got {id}"
            );
        }
        assert!(
            !game.infected_ids.contains(&1),
            "the player must never be infected while still Playing (infection is lethal)"
        );
    }

    assert!(
        !game.infected_ids.is_empty() || game.state != GameState::Playing,
        "while the match is running there must always be at least one infected participant"
    );
}
