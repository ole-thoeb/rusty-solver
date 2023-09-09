use crate::common::{Board3x3, Cell, Symmetric3x3Strategy};
use crate::min_max::{MoveSourceSink, Player};
use crate::min_max::symmetry::{SymmetricMove, SymmetricMove3x3, Symmetry};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum CellState {
    EMPTY,
    X,
    O,
}

impl Cell for CellState {
    fn empty() -> Self {
        Self::EMPTY
    }
}

impl From<Player> for CellState {
    fn from(player: Player) -> Self {
        match player {
            Player::Max => CellState::X,
            Player::Min => CellState::O,
        }
    }
}

pub type Board = Board3x3<CellState>;
pub type Strategy = Symmetric3x3Strategy<CellState>;

impl MoveSourceSink<Board, SymmetricMove3x3> for Strategy {
    fn possible_moves(state: &Board) -> Vec<SymmetricMove3x3> {
        let symmetry = state.symmetry();
        let mut covered_index = [false; 9];
        let moves_iter = state.cells.iter().enumerate().filter_map(move |(index, &cell_state)| {
            if cell_state != CellState::EMPTY {
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
            CellState::EMPTY => CellState::from(player),
            _ => panic!(),
        };
        new_state.last_player = player;
        return new_state;
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Instant;
    use crate::min_max::{score_possible_moves};
    use crate::ttt::{Board, Strategy};

    #[test]
    fn empty_board() {
        let board = Board::empty();
        let start = Instant::now();
        let scored_moves = score_possible_moves(&mut Strategy::new(), &board, u8::MAX);
        println!("search on empty board took {}mus", start.elapsed().as_micros());
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