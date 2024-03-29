pub mod symmetry;
pub mod cache;
pub mod stats;

use itertools::Itertools;
use std::fmt::{Debug, Display};
use std::hash::{Hash};
use std::ops::Not;
pub use crate::min_max::cache::{CacheEntry, CacheFlag};
use crate::min_max::cache::Cache;
use crate::min_max::stats::Stats;

use crate::min_max::symmetry::{SymmetricMove, SymmetricMove3x3, Symmetry};

#[derive(Eq, PartialEq, Hash)]
#[derive(Debug, Copy, Clone)]
pub enum Player {
    Min,
    Max,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ScoredMove<M> {
    pub score: i32,
    pub min_max_move: M,
}

impl<M> ScoredMove<M> {
    pub fn new(score: i32, min_max_move: M) -> ScoredMove<M> {
        ScoredMove { score, min_max_move }
    }
}

impl<I, S: Symmetry<I>> ScoredMove<SymmetricMove<I, S>> {
    pub fn expand(&self) -> Vec<ScoredMove<I>> {
        let score = self.score;
        self.min_max_move.expanded_indices().into_iter()
            .map(move |i| ScoredMove::new(score, i))
            .collect()
    }
}

impl Not for Player {
    type Output = Player;

    fn not(self) -> Player {
        match self {
            Player::Min => Player::Max,
            Player::Max => Player::Min,
        }
    }
}

pub trait Strategy {
    type State;
    type Move;
    type Cache: Cache<Self::State>;
    type Stats: Stats;

    fn possible_moves(state: &Self::State) -> impl IntoIterator<Item=Self::Move>;
    fn do_move(&mut self, state: &Self::State, _move: &Self::Move, player: Player) -> Self::State;
    
    fn score(&mut self, state: &Self::State, player: Player) -> i32;

    fn cache(&mut self) -> &mut Self::Cache;
    fn stats(&mut self) -> &mut Self::Stats;
}

pub fn alpha_beta<STRATEGY: Strategy>(strategy: &mut STRATEGY, state: &mut STRATEGY::State, max_level: u8) -> Vec<ScoredMove<STRATEGY::Move>> {
    score_possible_moves(strategy, state, max_level).into_iter().max_set_by_key(|state| state.score)
}

pub fn score_possible_moves<STRATEGY: Strategy>(strategy: &mut STRATEGY, state: &STRATEGY::State, max_level: u8) -> Vec<ScoredMove<STRATEGY::Move>> {
    let pos_moves = STRATEGY::possible_moves(&state);
    return pos_moves.into_iter().map(|m| {
        let next_state = strategy.do_move(state, &m, Player::Max);
        let score = -alpha_beta_eval_single_move(strategy, &next_state, Player::Min, max_level - 1, -i32::MAX, i32::MAX);
        ScoredMove::new(score, m)
    }).collect();
}

fn alpha_beta_eval_single_move<STRATEGY: Strategy>(strategy: &mut STRATEGY, state: &STRATEGY::State, player: Player, remaining_levels: u8, mut alpha: i32, mut beta: i32) -> i32 {
    if remaining_levels == 0 {
        return strategy.score(state, player) * (i32::from(remaining_levels) + 1);
    }

    let alpha_original = alpha;
    if let Some(entry) = strategy.cache().get(state) {
        if entry.level >= remaining_levels {
            match entry.flag {
                CacheFlag::Exact => return entry.value,
                CacheFlag::LowerBound => alpha = alpha.max(entry.value),
                CacheFlag::UpperBound => beta = beta.min(entry.value),
            }
            if alpha >= beta {
                return entry.value;
            }
        }
    }

    // Check if this state is terminal i.e. no more moves can be made
    let mut moves = STRATEGY::possible_moves(state).into_iter().peekable();
    if moves.peek().is_none() {
        return strategy.score(state, player) * (i32::from(remaining_levels) + 1);
    }

    let mut max_score = -i32::MAX;
    for m in moves {
        let next_state = strategy.do_move(state, &m, player);
        max_score = max_score.max(-alpha_beta_eval_single_move(strategy, &next_state, !player, remaining_levels - 1, -beta, -alpha));
        alpha = alpha.max(max_score);
        if alpha >= beta {
            break;
        }
    }
    let flag = if max_score <= alpha_original {
        CacheFlag::UpperBound
    } else if max_score >= beta {
        CacheFlag::LowerBound
    } else {
        CacheFlag::Exact
    };
    strategy.cache().set(state, CacheEntry {
        level: remaining_levels,
        flag,
        value: max_score,
    });
    max_score
}

pub fn to_score_board(scored_moves: &Vec<ScoredMove<SymmetricMove3x3>>) -> [i32; 9] {
    let mut scores = [0; 9];
    for m in scored_moves.iter() {
        for index in m.min_max_move.expanded_indices() {
            if scores[index] != 0 {
                scores[index] = scores[index].max(m.score);
            }
            scores[index] = m.score;
        }
    }
    scores
}

pub fn print_3_by_3<E: Display>(scored_board: &[E; 9]) {
    let scores = scored_board;
    eprintln!("{:>3}, {:>3}, {:>3}", scores[0], scores[1], scores[2]);
    eprintln!("{:>3}, {:>3}, {:>3}", scores[3], scores[4], scores[5]);
    eprintln!("{:>3}, {:>3}, {:>3}", scores[6], scores[7], scores[8]);
}
