pub mod symmetry;

use itertools::Itertools;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::hash::{Hash};
use std::ops::Not;

use strum_macros::EnumIter;
use crate::min_max::symmetry::{GridSymmetry3x3, Symmetry};

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

impl Not for Player {
    type Output = Player;

    fn not(self) -> Player {
        match self {
            Player::Min => Player::Max,
            Player::Max => Player::Min,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CacheFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct CacheEntry {
    value: i32,
    level: u8,
    flag: CacheFlag,
}

pub trait Strategy<S, M> {
    fn possible_moves(state: &S) -> Vec<M>;
    fn is_terminal(state: &S) -> bool;
    fn score(state: &S, player: Player) -> i32;
    fn do_move(state: &S, min_max_move: &M, player: Player) -> S;
    fn cache(&mut self, state: &S, entry: CacheEntry);
    fn lookup(&mut self, state: &S) -> Option<CacheEntry>;
}

pub fn alpha_beta<S, M: Clone, STRATEGY: Strategy<S, M>>(strategy: &mut STRATEGY, state: &mut S, max_level: u8) -> Vec<ScoredMove<M>> {
    score_possible_moves(strategy, state, max_level).into_iter().max_set_by_key(|state| state.score)
}

pub fn score_possible_moves<S, M, STRATEGY: Strategy<S, M>>(strategy: &mut STRATEGY, state: &S, max_level: u8) -> Vec<ScoredMove<M>> {
    let pos_moves = STRATEGY::possible_moves(&state);
    return pos_moves.into_iter().map(|m| {
        let next_state = STRATEGY::do_move(state, &m, Player::Max);
        let score = -alpha_beta_eval_single_move(strategy, &next_state, Player::Min, max_level - 1, -i32::MAX, i32::MAX);
        ScoredMove::new(score, m)
    }).collect();
}

fn alpha_beta_eval_single_move<S, M, STRATEGY: Strategy<S, M>>(strategy: &mut STRATEGY, state: &S, player: Player, remaining_levels: u8, mut alpha: i32, mut beta: i32) -> i32 {
    if STRATEGY::is_terminal(state) || remaining_levels == 0 {
        return STRATEGY::score(state, player) * (i32::from(remaining_levels) + 1);
    }

    let alpha_original = alpha;
    if let Some(entry) = strategy.lookup(state) {
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

    let mut max_score = -i32::MAX;
    for m in STRATEGY::possible_moves(state) {
        let next_state = STRATEGY::do_move(state, &m, player);
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
    strategy.cache(state, CacheEntry {
        level: remaining_levels,
        flag,
        value: max_score,
    });
    max_score
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct SymmetricMove(pub usize, pub GridSymmetry3x3);

impl SymmetricMove {
    pub fn index(&self) -> usize {
        self.0
    }

    pub fn expanded_indices(&self) -> Vec<usize> {
        self.1.expand(&self.0)
    }
}

pub fn to_score_board(scored_moves: &Vec<ScoredMove<SymmetricMove>>) -> [i32; 9] {
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
