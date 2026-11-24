#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
use super::{Deserialize, Serialize};
use rand::Rng;

pub const TOURNAMENT_SIZE: usize = 8;
pub const TOURNAMENT_ELO_K: f64 = 32.0;

const BOT_NAMES: [&str; 10] = [
    "Viper", "Cobra", "Mamba", "Python", "Asp", "Krait", "Boa", "Taipan", "Rattler", "Adder",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TournamentEntrant {
    pub name: String,
    pub elo: u32,
    pub is_player: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchOutcome {
    A,
    B,
    Draw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TournamentResult {
    pub round: usize,
    pub match_index: usize,
    pub a: usize,
    pub b: usize,
    pub winner: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tournament {
    pub entrants: Vec<TournamentEntrant>,
    pub rounds: Vec<Vec<Option<usize>>>,
    pub round: usize,
    pub match_index: usize,
    pub results: Vec<TournamentResult>,
    pub champion: Option<usize>,
}

#[must_use]
pub fn elo_ratings(rating_a: u32, rating_b: u32, score_a: f64) -> (u32, u32) {
    let a = f64::from(rating_a);
    let b = f64::from(rating_b);
    let expected_a = 1.0 / (1.0 + 10.0_f64.powf((b - a) / 400.0));
    let expected_b = 1.0 - expected_a;
    let new_a = ((score_a - expected_a).mul_add(TOURNAMENT_ELO_K, a)).max(0.0).round() as u32;
    let new_b =
        (((1.0 - score_a) - expected_b).mul_add(TOURNAMENT_ELO_K, b)).max(0.0).round() as u32;
    (new_a, new_b)
}

impl Tournament {
    #[must_use]
    pub fn new(rng: &mut rand::rngs::StdRng, player_name: &str, player_elo: u32) -> Self {
        let mut names: Vec<&str> = BOT_NAMES.to_vec();
        for i in (1..names.len()).rev() {
            let j = rng.gen_range(0..=i);
            names.swap(i, j);
        }
        let mut entrants = Vec::with_capacity(TOURNAMENT_SIZE);
        entrants.push(TournamentEntrant {
            name: if player_name.is_empty() {
                "You".to_string()
            } else {
                player_name.to_string()
            },
            elo: player_elo,
            is_player: true,
        });
        for name in names.iter().take(TOURNAMENT_SIZE - 1) {
            let delta = rng.gen_range(-100_i32..=150_i32);
            let elo = (i64::from(player_elo) + i64::from(delta)).max(0) as u32;
            entrants.push(TournamentEntrant {
                name: (*name).to_string(),
                elo,
                is_player: false,
            });
        }
        for i in (1..entrants.len()).rev() {
            let j = rng.gen_range(0..=i);
            entrants.swap(i, j);
        }
        let mut rounds: Vec<Vec<Option<usize>>> = vec![(0..TOURNAMENT_SIZE).map(Some).collect()];
        let mut slots = TOURNAMENT_SIZE / 2;
        while slots >= 1 {
            rounds.push(vec![None; slots]);
            slots /= 2;
        }
        Self {
            entrants,
            rounds,
            round: 0,
            match_index: 0,
            results: Vec::new(),
            champion: None,
        }
    }

    #[must_use]
    pub fn current_match(&self) -> Option<(usize, usize)> {
        if self.champion.is_some() {
            return None;
        }
        let slots = self.rounds.get(self.round)?;
        let a = *slots.get(self.match_index * 2)?;
        let b = *slots.get(self.match_index * 2 + 1)?;
        Some((a?, b?))
    }

    #[must_use]
    pub fn matches_in_round(&self, round: usize) -> usize {
        self.rounds.get(round).map_or(0, |r| r.len() / 2)
    }

    #[must_use]
    pub fn round_label(&self) -> String {
        let match_rounds = self.rounds.len().saturating_sub(1);
        match match_rounds.saturating_sub(self.round) {
            1 => "Final".to_string(),
            2 => "Semifinal".to_string(),
            3 => "Quarterfinal".to_string(),
            _ => format!("Round {}", self.round + 1),
        }
    }

    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.champion.is_some()
    }

    #[must_use]
    pub const fn matches_played(&self) -> usize {
        self.results.len()
    }

    #[must_use]
    pub const fn total_matches(&self) -> usize {
        TOURNAMENT_SIZE - 1
    }

    #[must_use]
    pub fn player_index(&self) -> Option<usize> {
        self.entrants.iter().position(|e| e.is_player)
    }

    #[must_use]
    pub fn champion_name(&self) -> Option<&str> {
        self.champion.map(|c| self.entrants[c].name.as_str())
    }

    /// Records the outcome of the current match, updates entrant ELO ratings and
    /// advances the bracket. Returns `true` once the tournament has a champion.
    /// A draw leaves the bracket untouched, so the same pairing is replayed.
    pub fn record_result(&mut self, outcome: MatchOutcome) -> bool {
        let Some((a, b)) = self.current_match() else {
            return self.is_finished();
        };
        let (score_a, winner) = match outcome {
            MatchOutcome::A => (1.0, Some(a)),
            MatchOutcome::B => (0.0, Some(b)),
            MatchOutcome::Draw => (0.5, None),
        };
        let (new_a, new_b) = elo_ratings(self.entrants[a].elo, self.entrants[b].elo, score_a);
        self.entrants[a].elo = new_a;
        self.entrants[b].elo = new_b;
        self.results.push(TournamentResult {
            round: self.round,
            match_index: self.match_index,
            a,
            b,
            winner,
        });
        if let Some(w) = winner {
            let next_round = self.round + 1;
            if let Some(next_slots) = self.rounds.get_mut(next_round) {
                if next_slots.len() == 1 {
                    next_slots[0] = Some(w);
                    self.champion = Some(w);
                    return true;
                }
                if let Some(slot) = next_slots.get_mut(self.match_index) {
                    *slot = Some(w);
                }
            }
            self.match_index += 1;
            if self.match_index >= self.matches_in_round(self.round) {
                self.match_index = 0;
                self.round += 1;
            }
        }
        false
    }

    #[must_use]
    pub fn status_line(&self) -> String {
        if let Some(name) = self.champion_name() {
            let c = self.champion.unwrap_or(0);
            return format!("Champion: {name} ({})", self.entrants[c].elo);
        }
        if let Some((a, b)) = self.current_match() {
            return format!(
                "{} {}/{}: {} ({}) vs {} ({})",
                self.round_label(),
                self.match_index + 1,
                self.matches_in_round(self.round),
                self.entrants[a].name,
                self.entrants[a].elo,
                self.entrants[b].name,
                self.entrants[b].elo
            );
        }
        String::new()
    }
}
