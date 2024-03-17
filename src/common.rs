use std::hash::Hash;
use lazy_static::lazy_static;
use crate::min_max::{Player};
use crate::min_max::cache::Cache;
use crate::min_max::stats::NullStats;
use crate::min_max::symmetry::{GridSymmetry3x3};

pub trait BoardStatus {
    fn is_max_won(&self) -> bool;
    fn is_min_won(&self) -> bool;
}

pub trait Cell: Copy + Eq + Hash {
    fn empty() -> Self;
}

pub trait Board: Clone + Eq + Hash {
    type Move;
    type BoardStatus: BoardStatus;
    fn last_player(&self) -> Player;
    fn status(&self) -> Self::BoardStatus;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Board3x3<C: Cell> {
    pub cells: [C; 9],
    pub last_player: Player,
    //pub last_move: Option<u8>,
}

impl<C: Cell> Board3x3<C> {
    pub fn empty() -> Self {
        Self::new([C::empty(); 9], Player::Max)
    }

    pub fn new(cells: [C; 9], last_player: Player) -> Self {
        Self { cells, last_player }
    }
    
    pub fn symmetry(&self) -> GridSymmetry3x3 {
        GridSymmetry3x3::from(&self.cells)
    }

    pub fn winning_indices(&self) -> Option<&[usize; 3]> {
        Self::WIN_INDICES.iter().find(|indices| {
            self.cells[indices[0]] != C::empty() && self.cells[indices[0]] == self.cells[indices[1]] && self.cells[indices[1]] == self.cells[indices[2]]
        })
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

lazy_static! {
    static ref WIN_INDICES: [Vec<[usize; 3]>; 9] = [
        vec![[0, 1, 2], [0, 3, 6],[0, 4, 8]],
        vec![[0, 1, 2], [1, 4, 7]],
        vec![[0, 1, 2], [2, 5, 8],[2, 4, 6]],
        vec![[3, 4, 5], [0, 3, 6]],
        vec![[3, 4, 5], [1, 4, 7],[0, 4, 8],[2, 4, 6]],
        vec![[3, 4, 5], [2, 5, 8]],
        vec![[6, 7, 8], [0, 3, 6],[2, 4, 6]],
        vec![[6, 7, 8], [1, 4, 7]],
        vec![[6, 7, 8], [2, 5, 8],[0, 4, 8]],
    ];
}

pub struct BaseStrategy<B: Board, CACHE: Cache<B>> {
    cache: CACHE,
    stats: NullStats,
    phantom: std::marker::PhantomData<B>,
}

impl<B: Board, CACHE: Cache<B>> BaseStrategy<B, CACHE> {
    pub fn new(cache: CACHE) -> Self {
        Self { cache, phantom: Default::default(), stats: NullStats::default() }
    }
    
    pub fn cache(&mut self) -> &mut CACHE {
        &mut self.cache
    }
    
    pub fn stats(&mut self) -> &mut NullStats {
        &mut self.stats
    }
}

pub fn default_score<S: BoardStatus>(status: S, player: Player) -> i32 {
    if status.is_max_won() {
        if player == Player::Max {
            1
        } else {
            -1
        }
    } else if status.is_min_won() {
        if player == Player::Min {
            1
        } else {
            -1
        }
    } else {
        0
    }
}
