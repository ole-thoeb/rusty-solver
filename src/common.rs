use std::collections::HashMap;
use std::hash::Hash;
use ahash::{RandomState};
use crate::min_max::{CacheEntry, MoveSourceSink, Player, Scorer, Strategy};
use crate::min_max::symmetry::{GridSymmetry3x3, SymmetricMove3x3, Symmetry};

pub trait BoardStatus {
    fn is_max_won(&self) -> bool;
    fn is_min_won(&self) -> bool;

    fn is_terminal(&self) -> bool {
        self.is_max_won() || self.is_min_won()
    }
}

pub trait Cell: Copy + Eq + Hash {
    fn empty() -> Self;
}

pub trait State {
    type BoardStatus: BoardStatus;
    fn status(&self) -> Self::BoardStatus;
}

pub trait Board: State + Clone + Eq + Hash {
    type Move;
    type Symmetry: Symmetry<Self::Move>;
    fn symmetry(&self) -> Self::Symmetry;
    fn last_player(&self) -> Player;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Board3x3<C: Cell> {
    pub cells: [C; 9],
    pub last_player: Player,
}

impl <C: Cell> Board3x3<C> {

    pub fn empty() -> Self {
        Self::new([C::empty(); 9], Player::Max)
    }

    pub fn new(cells: [C; 9], last_player: Player) -> Self {
        Self { cells, last_player }
    }

    pub fn winning_indices(&self) -> Option<&[usize; 3]> {
        Self::WIN_INDICES.iter().find(|indices| {
            self.cells[indices[0]] == self.cells[indices[1]] && self.cells[indices[1]] == self.cells[indices[2]] && self.cells[indices[0]] != C::empty()
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

impl<C> Board for Board3x3<C> where C: Cell, Self: State {

    type Move = usize;
    type Symmetry = GridSymmetry3x3;

    fn symmetry(&self) -> Self::Symmetry {
        GridSymmetry3x3::from(&self.cells)
    }

    fn last_player(&self) -> Player {
        self.last_player
    }
}

pub struct BaseStrategy<B: Board> {
    cache: HashMap<B, CacheEntry, RandomState>,
}

impl<B: Board> BaseStrategy<B> {
    pub fn new() -> Self {
        Self { cache: HashMap::default() }
    }
}

pub fn default_score<B: Board>(state: &B, player: Player) -> i32 {
    if state.status().is_max_won() {
        debug_assert_eq!(state.last_player(), Player::Max);
        debug_assert_eq!(player, Player::Min);
        debug_assert_ne!(state.last_player(), player);
        -1
    } else if state.status().is_min_won() {
        debug_assert_eq!(state.last_player(), Player::Min);
        debug_assert_eq!(player, Player::Max);
        debug_assert_ne!(state.last_player(), player);
        -1
    }else {
        0
    }
}

impl<B: Board, M> Strategy<B, M> for BaseStrategy<B> where Self: MoveSourceSink<B, M> + Scorer<B> {
    fn is_terminal(state: &B) -> bool {
        state.status().is_terminal()
    }

    fn cache(&mut self, state: &B, entry: CacheEntry) {
        self.cache.insert(state.clone(), entry);
    }

    fn lookup(&mut self, state: &B) -> Option<CacheEntry> {
        self.cache.get(&state).cloned()
    }
}