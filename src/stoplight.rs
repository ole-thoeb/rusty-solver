use std::collections::HashSet;
use std::hash::Hash;

use rand::seq::SliceRandom;
use crate::common::{Board3x3, Cell, Symmetric3x3Strategy};

use crate::min_max::*;
use crate::min_max::symmetry::{SymmetricMove, SymmetricMove3x3, Symmetry};

#[derive(Eq, PartialEq)]
#[derive(Debug, Copy, Clone, Hash)]
pub enum CellState {
    EMPTY,
    GREEN,
    YELLOW,
    RED,
}

impl Cell for CellState {
    fn empty() -> Self {
        Self::EMPTY
    }
}

pub type Board = Board3x3<CellState>;
pub type Cells = [CellState; 9];
pub type Strategy = Symmetric3x3Strategy<CellState>;

impl MoveSourceSink<Board, SymmetricMove3x3> for Strategy {
    fn possible_moves(state: &Board) -> Vec<SymmetricMove3x3> {
        let symmetry = state.symmetry();
        let mut covered_index = [false; 9];
        let moves_iter = state.cells.iter().enumerate().filter_map(move |(index, &cell_state)| {
            if cell_state == CellState::RED {
                return None;
            }
            let normalised = symmetry.canonicalize(&index);
            if covered_index[normalised] {
                return None;
            }
            covered_index[normalised] = true;
            return Some(SymmetricMove(normalised, symmetry.clone()));
        });
        let mut moves = Vec::with_capacity(7);
        moves.extend(moves_iter);
        moves
    }

    fn do_move(state: &Board, SymmetricMove(index, _): &SymmetricMove3x3, player: Player) -> Board {
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
}

pub fn choose_random_move(moves: Vec<ScoredMove<SymmetricMove3x3>>) -> ScoredMove<usize> {
    all_move_indices(moves)
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

pub fn all_move_indices(moves: Vec<ScoredMove<SymmetricMove3x3>>) -> Vec<ScoredMove<usize>> {
    moves.iter()
        .flat_map(|m| m.min_max_move.expanded_indices().into_iter().map(move |i| ScoredMove::new(m.score, i)))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Instant;
    use crate::min_max::{alpha_beta, Player, score_possible_moves};
    use crate::stoplight::{Board, Cells, CellState, print_3_by_3, Strategy, to_score_board};

    fn best_move_index_of(cells: Cells) -> usize {
        let m = alpha_beta(&mut Strategy::new(), &mut Board::new(cells, Player::Max), 30);
        print_3_by_3(&to_score_board(&m));
        return *m[0].min_max_move.index();
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
        assert_eq!(scored_moves.iter().max_by_key(|m| m.score).map(|m| *m.min_max_move.index()), Some(4));

        let scored_expanded_moves = scored_moves.iter()
            .flat_map(|m| m.expand())
            .collect::<HashSet<_>>();

        assert_eq!(scored_expanded_moves.len(), 9);

        let min_score = scored_expanded_moves.iter().map(|m| m.score).min().unwrap();
        for m in scored_expanded_moves {
            // all other moves have the same (lower) score
            assert!(m.score == min_score || m.min_max_move == 4)
        }
    }
}