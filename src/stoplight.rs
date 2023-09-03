use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::hash::Hash;

use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

use crate::min_max::*;
use crate::min_max::symmetry::{GridSymmetry3x3, GridSymmetryAxes, GridSymmetryAxis, Symmetry};

#[derive(Eq, PartialEq)]
#[derive(Debug, Copy, Clone, Hash)]
pub enum CellState {
    EMPTY,
    GREEN,
    YELLOW,
    RED,
}

type Cells = [CellState; 9];

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Board {
    pub cells: Cells,
    last_player: Player,
}

#[derive(Eq, PartialEq)]
#[derive(Debug, Copy, Clone)]
pub enum BoardStatus {
    MaxWon,
    MinWon,
    Ongoing,
}

impl Board {
    fn symmetry(&self) -> GridSymmetry3x3 {
        let axes = GridSymmetryAxis::iter().filter(|symmetry| {
            symmetry.symmetric_indices_3x3().iter().all(|(f, s)| self.cells[*f] == self.cells[*s])
        }).collect::<GridSymmetryAxes>();
        GridSymmetry3x3::new(axes)
    }

    fn status(&self) -> BoardStatus {
        let winning_indices = Self::WIN_INDICES.iter().find(|indices| {
            self.cells[indices[0]] == self.cells[indices[1]] && self.cells[indices[1]] == self.cells[indices[2]] && self.cells[indices[0]] != CellState::EMPTY
        });
        match winning_indices {
            None => BoardStatus::Ongoing,
            Some(_indices) => match self.last_player {
                Player::Min => BoardStatus::MinWon,
                Player::Max => BoardStatus::MaxWon,
            }
        }
    }

    pub fn empty() -> Board {
        Self::new([CellState::EMPTY; 9], Player::Max)
    }

    pub fn new(cells: [CellState; 9], last_player: Player) -> Board {
        Board { cells, last_player }
    }

    pub fn from_string(str: &String) -> Option<Board> {
        str.split(",").filter_map(|s| match s {
            "e" => Some(CellState::EMPTY),
            "g" => Some(CellState::GREEN),
            "y" => Some(CellState::YELLOW),
            "r" => Some(CellState::RED),
            _ => None,
        }).collect::<Vec<CellState>>()
            .try_into()
            .map(|cells| Board::new(cells, Player::Max))
            .ok()
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


pub struct Strategy {
    cache: HashMap<Board, CacheEntry>,
}

impl Strategy {
    pub fn new() -> Strategy {
        Strategy { cache: HashMap::new() }
    }
}

impl crate::min_max::Strategy<Board, SymmetricMove> for Strategy {
    fn possible_moves(state: &Board) -> Vec<SymmetricMove> {
        let symmetry = state.symmetry();
        let mut covered_index = [false; 9];
        let mut moves = Vec::new();
        for (index, &cell_state) in state.cells.iter().enumerate() {
            if cell_state == CellState::RED {
                continue;
            }
            let normalised = symmetry.canonicalize(&index);
            if covered_index[normalised] {
                continue;
            }
            covered_index[normalised] = true;
            moves.push(SymmetricMove(index, symmetry.clone()))
        }
        moves
    }

    fn is_terminal(state: &Board) -> bool {
        state.status() != BoardStatus::Ongoing
    }

    fn score(state: &Board, player: Player) -> i32 {
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

    fn do_move(state: &Board, SymmetricMove(index, _): &SymmetricMove, player: Player) -> Board {
        let mut new_state = state.clone();
        new_state.cells[*index] = match state.cells[*index] {
            CellState::EMPTY => CellState::GREEN,
            CellState::GREEN => CellState::YELLOW,
            CellState::YELLOW => CellState::RED,
            CellState::RED => panic!(),
        };
        new_state.last_player = player;
        return new_state;
    }

    fn cache(&mut self, state: &Board, entry: CacheEntry) {
        self.cache.insert(state.clone(), entry);
    }

    fn lookup(&mut self, state: &Board) -> Option<CacheEntry> {
        self.cache.get(&state).cloned()
    }
}

pub fn choose_random_move(moves: Vec<ScoredMove<SymmetricMove>>) -> ScoredMove<usize> {
    all_move_indices(moves)
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

pub fn all_move_indices(moves: Vec<ScoredMove<SymmetricMove>>) -> Vec<ScoredMove<usize>> {
    moves.iter()
        .flat_map(|m| m.min_max_move.expanded_indices().into_iter().map(move |i| ScoredMove::new(m.score, i)))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use itertools::Itertools;
    use crate::min_max::{alpha_beta, Player, score_possible_moves, ScoredMove};
    use crate::stoplight::{Board, Cells, CellState, print_3_by_3, Strategy, to_score_board};

    fn best_move_index_of(cells: [CellState; 9]) -> usize {
        let m = alpha_beta(&mut Strategy::new(), &mut Board::new(cells, Player::Max), 30);
        print_3_by_3(&to_score_board(&m));
        return m[0].min_max_move.index();
    }

    fn score_board(cells: Cells) -> [i32; 9] {
        to_score_board(&score_possible_moves(&mut Strategy::new(), &mut Board::new(cells, Player::Max), 30))
    }

    #[test]
    fn two_green() {
        {
            let cells = [
                CellState::EMPTY, CellState::EMPTY, CellState::GREEN,
                CellState::GREEN, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            let scores = score_board(cells);
            print_3_by_3(&scores)
        }
        {
            let cells = [
                CellState::EMPTY, CellState::GREEN, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::GREEN,
            ];
            let scores = score_board(cells);
            print_3_by_3(&scores)
        }
        {
            let cells = [
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::GREEN,
                CellState::GREEN, CellState::EMPTY, CellState::EMPTY,
            ];
            let scores = score_board(cells);
            print_3_by_3(&scores)
        }
        {
            let cells = [
                CellState::GREEN, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::GREEN, CellState::EMPTY,
            ];
            let scores = score_board(cells);
            print_3_by_3(&scores)
        }
    }

    #[test]
    fn empty_board_inspect() {
        fn score_and_print(cells: Cells) {
            let score_board = score_board(cells);
            print_3_by_3(&score_board);
        }
        {
            let cells = [CellState::EMPTY; 9];
            score_and_print(cells);
        }
        println!("=> Ki plays 0");
        {
            let cells = [
                CellState::GREEN, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            score_and_print(cells);
        }
        println!("=> Human plays 0");
        {
            let cells = [
                CellState::YELLOW, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            score_and_print(cells);
        }
        println!("=> Ki plays 0");
        {
            let cells = [
                CellState::RED, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            score_and_print(cells);
        }
        println!("=> Human plays 1");
        {
            let cells = [
                CellState::RED, CellState::GREEN, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            score_and_print(cells);
        }
        println!("=> Ki plays 1");
        {
            let cells = [
                CellState::RED, CellState::YELLOW, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
                CellState::EMPTY, CellState::EMPTY, CellState::EMPTY,
            ];
            score_and_print(cells);
        }
    }

    #[test]
    fn empty_board() {
        let board = Board::empty();
        let start = Instant::now();
        let scored_moves = score_possible_moves(&mut Strategy::new(), &board, u8::MAX);
        println!("search on empty board took {}ms", start.elapsed().as_millis());
        // center is best move
        assert_eq!(scored_moves.iter().max_by_key(|m| m.score).map(|m| m.min_max_move.index()), Some(4));

        let scored_expanded_moves = scored_moves.iter()
            .flat_map(|m| {
                let score = m.score;
                m.min_max_move.expanded_indices().into_iter().map(move |i| ScoredMove::new(score, i))
            })
            .collect::<Vec<_>>();
        let min_score = scored_expanded_moves.iter().map(|m| m.score).min().unwrap();
        for m in scored_expanded_moves {
            // all other moves have the same (lower) score
            assert!(m.score == min_score || m.min_max_move == 4)
        }

    }
}