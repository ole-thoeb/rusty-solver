use std::collections::HashMap;
use std::hash::Hash;
use ahash::{RandomState};
use crate::min_max::{CacheEntry, MoveSourceSink, Player, Strategy};
use crate::min_max::symmetry::{GridSymmetry3x3, SymmetricMove3x3};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum BoardStatus {
    MaxWon,
    MinWon,
    Ongoing,
}

pub trait Cell: Copy + Eq {
    fn empty() -> Self;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Board3x3<C: Cell> {
    pub cells: [C; 9],
    pub last_player: Player,
}

impl<C> Board3x3<C> where C: Cell {
    pub fn symmetry(&self) -> GridSymmetry3x3 {
        GridSymmetry3x3::from(&self.cells)
    }

    pub fn status(&self) -> BoardStatus {
        let winning_indices = Self::WIN_INDICES.iter().find(|indices| {
            self.cells[indices[0]] == self.cells[indices[1]] && self.cells[indices[1]] == self.cells[indices[2]] && self.cells[indices[0]] != C::empty()
        });
        match winning_indices {
            None => BoardStatus::Ongoing,
            Some(_indices) => match self.last_player {
                Player::Min => BoardStatus::MinWon,
                Player::Max => BoardStatus::MaxWon,
            }
        }
    }

    pub fn empty() -> Self {
        Self::new([C::empty(); 9], Player::Max)
    }

    pub fn new(cells: [C; 9], last_player: Player) -> Self {
        Self { cells, last_player }
    }

    const WIN_INDICES: [[usize; 3]; 8] = [
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        [0, 3, 6],
        [1, 4, 7],
        [2, 5, 8],
        [0, 4, 8],
        [2, 4, 6],
    ];
}

pub struct Symmetric3x3Strategy<C: Cell> {
    cache: HashMap<Board3x3<C>, CacheEntry, RandomState>,
}

impl<C: Cell> Symmetric3x3Strategy<C> {
    pub fn new() -> Self {
        Self { cache: HashMap::default() }
    }
}

impl<C> Strategy<Board3x3<C>, SymmetricMove3x3> for Symmetric3x3Strategy<C> where C: Cell + Hash, Self: MoveSourceSink<Board3x3<C>, SymmetricMove3x3> {
    fn is_terminal(state: &Board3x3<C>) -> bool {
        state.status() != BoardStatus::Ongoing
    }

    fn score(state: &Board3x3<C>, player: Player) -> i32 {
        match state.status() {
            BoardStatus::MaxWon => {
                debug_assert_eq!(state.last_player, Player::Max);
                debug_assert_eq!(player, Player::Min);
                debug_assert_ne!(state.last_player, player);
                -1
            }
            BoardStatus::MinWon => {
                debug_assert_eq!(state.last_player, Player::Min);
                debug_assert_eq!(player, Player::Max);
                debug_assert_ne!(state.last_player, player);
                -1
            }
            BoardStatus::Ongoing => 0
        }
    }

    fn cache(&mut self, state: &Board3x3<C>, entry: CacheEntry) {
        self.cache.insert(state.clone(), entry);
    }

    fn lookup(&mut self, state: &Board3x3<C>) -> Option<CacheEntry> {
        self.cache.get(&state).cloned()
    }
}