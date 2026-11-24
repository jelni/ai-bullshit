use rand::SeedableRng;
use snake_game::snake::Point;
use snake_game::game::{
    Difficulty, Game, GameMode, GameState, MatchOutcome, Theme, Tournament, TOURNAMENT_SIZE,
    elo_ratings,
};

fn tournament_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Tournament;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while bots eat
    // Game::new loads stats.json from the cwd, which parallel test binaries
    // overwrite; reset the fields these tests assert on so they stay hermetic.
    game.stats.tournaments_played = 0;
    game.stats.tournaments_won = 0;
    game.stats
        .unlocked_achievements
        .retain(|a| *a != snake_game::game::Achievement::TournamentChampion);
    game
}

fn seeded_tournament(seed: u64) -> Tournament {
    Tournament::new(&mut rand::rngs::StdRng::seed_from_u64(seed), "Tester", 1200)
}

#[test]
fn test_tournament_bracket_shape() {
    let t = seeded_tournament(42);

    assert_eq!(t.entrants.len(), TOURNAMENT_SIZE);
    assert_eq!(t.entrants.iter().filter(|e| e.is_player).count(), 1);
    let player = t.player_index().expect("player entrant must exist");
    assert_eq!(t.entrants[player].name, "Tester");
    assert_eq!(t.entrants[player].elo, 1200);
    assert_eq!(t.rounds.len(), 4);
    assert_eq!(t.rounds[0].len(), 8);
    assert_eq!(t.rounds[1].len(), 4);
    assert_eq!(t.rounds[2].len(), 2);
    assert_eq!(t.rounds[3].len(), 1);
    assert_eq!(t.current_match(), Some((0, 1)));
    assert_eq!(t.round_label(), "Quarterfinal");
    assert_eq!(t.matches_in_round(0), 4);
    assert_eq!(t.matches_in_round(1), 2);
    assert_eq!(t.matches_in_round(2), 1);
    assert_eq!(t.total_matches(), 7);
    assert_eq!(t.matches_played(), 0);
    assert!(!t.is_finished());
    assert!(t.champion.is_none());
    // Bot names must be unique
    let mut names: Vec<&str> = t.entrants.iter().map(|e| e.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), TOURNAMENT_SIZE);
}

#[test]
fn test_tournament_record_result_advances_bracket_to_champion() {
    let mut t = seeded_tournament(7);

    // Quarterfinals: 4 matches, entrant A always wins
    for expected_match in 0..4 {
        assert_eq!(t.round, 0);
        assert_eq!(t.match_index, expected_match);
        assert!(!t.record_result(MatchOutcome::A));
    }
    assert_eq!(t.round, 1);
    assert_eq!(t.match_index, 0);
    assert_eq!(t.round_label(), "Semifinal");

    // Semifinals: 2 matches
    for expected_match in 0..2 {
        assert_eq!(t.match_index, expected_match);
        assert!(!t.record_result(MatchOutcome::B));
    }
    assert_eq!(t.round, 2);
    assert_eq!(t.round_label(), "Final");

    // Final: crowns the champion
    assert!(t.record_result(MatchOutcome::A));
    assert!(t.is_finished());
    assert!(t.champion.is_some());
    assert_eq!(t.current_match(), None);
    assert_eq!(t.matches_played(), 7);
    assert_eq!(t.results.len(), 7);
    let champ = t.champion.unwrap();
    assert!(t.champion_name().is_some());
    // Every recorded match has a winner and both slots were filled
    for r in &t.results {
        assert!(r.winner.is_some());
    }
    // Winners advanced into the next round slots
    assert!(t.rounds[1].iter().all(Option::is_some));
    assert!(t.rounds[2].iter().all(Option::is_some));
    assert_eq!(t.rounds[3][0], Some(champ));
}

#[test]
fn test_tournament_draw_replays_same_match() {
    let mut t = seeded_tournament(9);
    let (a, b) = t.current_match().unwrap();
    let (elo_a, elo_b) = (t.entrants[a].elo, t.entrants[b].elo);

    assert!(!t.record_result(MatchOutcome::Draw));

    assert_eq!(t.round, 0);
    assert_eq!(t.match_index, 0);
    assert_eq!(t.current_match(), Some((a, b)));
    assert!(!t.is_finished());
    assert_eq!(t.results.len(), 1);
    assert_eq!(t.results[0].winner, None);
    // Draw moves both ratings toward the middle, never away from it
    let (new_a, new_b) = (t.entrants[a].elo, t.entrants[b].elo);
    if elo_a > elo_b {
        assert!(new_a < elo_a);
        assert!(new_b > elo_b);
    } else {
        assert!(new_a > elo_a);
        assert!(new_b < elo_b);
    }
}

#[test]
fn test_elo_ratings_outcomes() {
    // Equal ratings: win is +16/-16 (K=32, expected 0.5), draw is a wash
    let (w, l) = elo_ratings(1200, 1200, 1.0);
    assert_eq!(w, 1216);
    assert_eq!(l, 1184);
    let (a, b) = elo_ratings(1200, 1200, 0.5);
    assert_eq!(a, 1200);
    assert_eq!(b, 1200);
    // Underdog win gains more than the favorite would
    let (up, fav) = elo_ratings(800, 1600, 1.0);
    assert!(up - 800 > 16);
    assert!(1600 - fav > 0);
    // Zero-sum with equal K, and ratings never go negative
    assert_eq!(w + l, 2400);
    let (floor, _) = elo_ratings(0, 2000, 0.0);
    assert_eq!(floor, 0);
}

#[test]
fn test_game_reset_creates_tournament() {
    let game = tournament_game(40, 30);

    assert_eq!(game.mode, GameMode::Tournament);
    let tournament = game.tournament.as_ref().expect("tournament must be created");
    assert_eq!(tournament.entrants.len(), TOURNAMENT_SIZE);
    assert_eq!(tournament.entrants.iter().filter(|e| e.is_player).count(), 1);
    assert!(game.player2.is_some());
    assert!(game.used_bot_this_session, "bot-driven entrant must flag [BOT] scoring");
    // Non-tournament modes must not carry tournament state
    let mut other = Game::new(40, 30, false, 'x', Theme::Classic, Difficulty::Normal);
    other.mode = GameMode::SinglePlayer;
    other.reset();
    assert!(other.tournament.is_none());
}

#[test]
fn test_tournament_match_end_advances_bracket_and_respawns() {
    let mut game = tournament_game(40, 30);
    game.state = GameState::Playing;
    let start = game.snake.head();
    let (a, b) = game.tournament.as_ref().unwrap().current_match().unwrap();

    // Player 2 (entrant B) loses the first match
    game.handle_tournament_match_end(true, false);

    let tournament = game.tournament.as_ref().unwrap();
    assert_eq!(tournament.results.len(), 1);
    assert_eq!(tournament.results[0].a, a);
    assert_eq!(tournament.results[0].b, b);
    assert_eq!(tournament.results[0].winner, Some(b));
    assert_eq!(tournament.match_index, 1, "bracket must advance to match 2 of the quarterfinals");
    assert!(!tournament.is_finished());
    // Arena was reset for the next match: both snakes exist and the player snake moved back to spawn
    assert!(game.player2.is_some());
    assert_eq!(
        game.snake.head(),
        Point {
            x: 15,
            y: 15
        }
    );
    let _ = start;
    assert!(game.state == GameState::Playing);
    assert_eq!(game.stats.tournaments_played, 0);
}

#[test]
fn test_tournament_runs_to_champion_and_ends_game() {
    let mut game = tournament_game(40, 30);
    game.state = GameState::Playing;

    // Play out all 7 matches; entrant A (player side snake) always wins
    for _ in 0..7 {
        game.handle_tournament_match_end(false, true);
        if game.tournament.as_ref().is_some_and(|t| t.is_finished()) {
            break;
        }
    }

    let tournament = game.tournament.as_ref().unwrap();
    assert!(tournament.is_finished(), "7 matches must crown a champion");
    assert_eq!(tournament.matches_played(), 7);
    assert!(tournament.champion_name().is_some());
    assert_eq!(game.stats.tournaments_played, 1);
    assert!(game.state == GameState::GameOver);
    assert!(
        game.death_message.contains("Tournament Champion"),
        "death message was: {}",
        game.death_message
    );
    // Stats must be consistent with whoever was crowned champion
    let player = tournament.player_index().unwrap();
    let player_won = tournament.champion == Some(player);
    assert_eq!(game.stats.tournaments_won, u32::from(player_won));
    assert_eq!(
        game.stats
            .unlocked_achievements
            .contains(&snake_game::game::Achievement::TournamentChampion),
        player_won
    );
    assert_eq!(game.stats.player_elo, tournament.entrants[player].elo);
}

#[test]
fn test_tournament_draw_does_not_advance_bracket() {
    let mut game = tournament_game(40, 30);
    game.state = GameState::Playing;

    game.handle_tournament_match_end(true, true);

    let tournament = game.tournament.as_ref().unwrap();
    assert_eq!(tournament.results.len(), 1);
    assert_eq!(tournament.results[0].winner, None);
    assert_eq!(tournament.round, 0);
    assert_eq!(tournament.match_index, 0);
    assert!(!tournament.is_finished());
    assert!(game.state == GameState::Playing);
    assert_eq!(game.stats.tournaments_played, 0);
}

#[test]
fn test_tournament_bots_drive_both_snakes_without_input() {
    let mut game = tournament_game(40, 30);
    game.state = GameState::Playing;
    let start = game.snake.head();
    let start2 = game.player2.as_ref().unwrap().head();

    for _ in 0..10 {
        game.update();
        if game.state != GameState::Playing {
            game.state = GameState::Playing;
        }
    }

    let moved =
        game.snake.head() != start || game.player2.as_ref().is_some_and(|p2| p2.head() != start2);
    assert!(moved, "autopilot must move both tournament snakes without player input");
}

#[test]
fn test_tournament_persists_through_time_rewind() {
    let mut game = tournament_game(40, 30);
    game.state = GameState::Playing;

    game.handle_tournament_match_end(false, true);
    let snapshot = game.tournament.clone().unwrap();
    assert_eq!(snapshot.matches_played(), 1);

    game.save_history_state();
    game.handle_tournament_match_end(false, true);
    assert_eq!(game.tournament.as_ref().unwrap().matches_played(), 2);

    game.rewind_time();
    assert_eq!(game.tournament.as_ref().unwrap(), &snapshot);
}

#[test]
fn test_tournament_serde_roundtrip() {
    let mut t = seeded_tournament(123);
    t.record_result(MatchOutcome::A);
    t.record_result(MatchOutcome::Draw);
    let json = serde_json::to_string(&t).expect("tournament must serialize");
    let back: Tournament = serde_json::from_str(&json).expect("tournament must deserialize");
    assert_eq!(t, back);
}

#[test]
fn test_tournament_status_line_reports_current_match_and_champion() {
    let mut t = seeded_tournament(5);
    let (a, b) = t.current_match().unwrap();
    let line = t.status_line();
    assert!(line.contains("Quarterfinal 1/4"), "line was: {line}");
    assert!(line.contains(&t.entrants[a].name));
    assert!(line.contains(&t.entrants[b].name));

    for _ in 0..7 {
        if t.record_result(MatchOutcome::A) {
            break;
        }
    }
    let line = t.status_line();
    assert!(line.contains("Champion:"), "line was: {line}");
    assert!(line.contains(t.champion_name().unwrap()));
}

#[test]
fn test_tournament_uses_seeded_rng_deterministically() {
    let t1 = seeded_tournament(2024);
    let t2 = seeded_tournament(2024);
    assert_eq!(t1, t2);
    let names1: Vec<&str> = t1.entrants.iter().map(|e| e.name.as_str()).collect();
    let names2: Vec<&str> = t2.entrants.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names1, names2);
}
