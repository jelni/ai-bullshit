use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn tag_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BombTag;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn solo_bot_game() -> Game {
    let mut game = tag_game(30, 30);
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
fn test_bomb_tag_reset_spawns_bomb_and_bots() {
    let game = tag_game(30, 30);

    assert_eq!(game.bots.len(), 3, "Bomb Tag should spawn 3 arena bots");
    assert!(
        game.bomb_carrier.is_some(),
        "a bomb carrier must be chosen on reset"
    );
    assert_eq!(game.bomb_timer, Game::BOMB_TIMER_MAX);
    assert_eq!(game.bomb_tags, 0);
}

#[test]
fn test_bomb_tag_timer_counts_down() {
    let mut game = solo_bot_game();
    game.bomb_carrier = Some(1);
    game.bomb_timer = 10;

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.bomb_timer, 9,
        "the bomb timer must tick down once per update"
    );
    assert_eq!(
        game.bomb_carrier,
        Some(1),
        "the bomb must stay put when nobody is adjacent"
    );
    assert_eq!(game.bomb_tags, 0);
}

#[test]
fn test_bomb_tag_passes_on_adjacency() {
    let mut game = solo_bot_game();
    // Player moves (5,5) -> (6,5); the bot moves (20,20) is replaced below.
    let mut bot = Snake::new(Point { x: 7, y: 6 });
    bot.direction = Direction::Left;
    bot.direction_queue.push_back(Direction::Left); // bot moves to (6,6)
    game.bots = vec![bot];
    game.bots_autopilot_paths = vec![Vec::new()];
    game.bomb_carrier = Some(1); // player carries
    game.bomb_timer = 10;

    game.state = GameState::Playing;
    game.update();

    // Post-move heads: player (6,5), bot (6,6) -> Manhattan distance 1 -> pass
    assert_eq!(
        game.bomb_carrier,
        Some(3),
        "the bomb must pass to the adjacent snake (bot id 3)"
    );
    assert_eq!(game.bomb_tags, 1, "a pass must be counted");
    assert_eq!(
        game.bomb_timer,
        Game::BOMB_TIMER_MAX,
        "passing the bomb resets the countdown"
    );
}

#[test]
fn test_bomb_tag_explosion_eliminates_bot_and_last_one_standing_wins() {
    let mut game = solo_bot_game();
    game.bomb_carrier = Some(3); // the bot carries
    game.bomb_timer = 1;

    game.state = GameState::Playing;
    game.update();

    assert!(
        game.bots.is_empty(),
        "the carrier bot must be eliminated when the timer hits zero"
    );
    assert_eq!(
        game.bomb_carrier,
        Some(1),
        "the bomb must respawn on the remaining participant (the player)"
    );
    assert_eq!(game.bomb_timer, Game::BOMB_TIMER_MAX);
    assert!(
        game.state == GameState::GameWon,
        "with every bot eliminated, the player is the last one standing and wins"
    );
}

#[test]
fn test_bomb_tag_player_explosion_costs_a_life() {
    let mut game = solo_bot_game();
    let lives_before = game.lives;
    game.bomb_carrier = Some(1); // the player carries
    game.bomb_timer = 1;

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "the player losing the bomb must cost a life"
    );
    assert_ne!(
        game.bomb_carrier,
        Some(1),
        "the bomb must move away from the eliminated carrier"
    );
    assert!(
        game.bomb_carrier.is_some(),
        "the bomb must respawn on another participant"
    );
}

#[test]
fn test_bomb_tag_state_persists_in_history_state() {
    let mut game = tag_game(30, 30);
    game.bomb_carrier = Some(3);
    game.bomb_timer = 17;
    game.bomb_tags = 5;

    game.save_history_state();

    let last = game.history.back().expect("history should have one entry");
    assert_eq!(last.bomb_carrier, Some(3));
    assert_eq!(last.bomb_timer, 17);
    assert_eq!(
        last.bomb_tags, 5,
        "tag counter must be stored in HistoryState for time-rewind consistency"
    );
}

#[test]
fn test_bomb_tag_autopilot_game_runs_cleanly() {
    let mut game = tag_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != GameState::Playing {
            break;
        }
        assert!(
            game.bomb_timer <= Game::BOMB_TIMER_MAX,
            "bomb timer must never exceed the maximum"
        );
        if let Some(carrier) = game.bomb_carrier {
            let valid = carrier == 1
                || (carrier == 2 && game.player2.is_some())
                || (carrier >= 3 && usize::from(carrier - 3) < game.bots.len());
            assert!(valid, "the carrier must reference an alive participant");
        }
    }

    assert!(
        game.bomb_timer <= Game::BOMB_TIMER_MAX,
        "bomb timer must never exceed the maximum after the run"
    );
}
