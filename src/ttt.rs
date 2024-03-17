use crate::common;
use crate::common::{Board3x3, Cell, State, BaseStrategy, default_score, SymmetricBoard};
use crate::min_max::{MoveSourceSink, Player, Scorer};
use crate::min_max::cache::{NullCache};
use crate::min_max::symmetry::{SymmetricMove, SymmetricMove3x3, Symmetry};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum BoardStatus {
    MaxWon,
    MinWon,
    Draw,
    Ongoing,
}

impl common::BoardStatus for BoardStatus {
    fn is_max_won(&self) -> bool {
        matches!(self, BoardStatus::MaxWon)
    }

    fn is_min_won(&self) -> bool {
        matches!(self, BoardStatus::MinWon)
    }
}

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

pub type GameBoard = Board3x3<CellState>;

impl State for GameBoard {
    type BoardStatus = BoardStatus;

    fn status(&self) -> Self::BoardStatus {
        match self.winning_indices() {
            Some(indices) => match self.cells[indices[0]] {
                CellState::X => BoardStatus::MaxWon,
                CellState::O => BoardStatus::MinWon,
                _ => panic!(),
            },
            None => {
                if self.cells.iter().any(|c| c == &CellState::EMPTY) {
                    BoardStatus::Ongoing
                } else {
                    BoardStatus::Draw
                }
            }
        }
    }
}

pub type Strategy = BaseStrategy<GameBoard, NullCache>;

impl MoveSourceSink<GameBoard, SymmetricMove3x3> for Strategy {
    fn possible_moves(state: &GameBoard) -> impl IntoIterator<Item=SymmetricMove3x3> + 'static {
        let symmetry = state.symmetry();
        let mut covered_index = [false; 9];
        let cells = state.cells;
        [4, 0, 1, 2, 3, 5, 6, 7, 8].into_iter().filter_map(move |index| {
            if cells[index] != CellState::EMPTY {
                return None;
            }
            let normalised = symmetry.canonicalize(&index);
            if covered_index[normalised] {
                return None;
            }
            covered_index[normalised] = true;
            return Some(SymmetricMove(normalised, symmetry.clone()));
        })
    }

    fn do_move(&mut self, state: &GameBoard, SymmetricMove(index, _): &SymmetricMove3x3, player: Player) -> GameBoard {
        let mut new_state = state.clone();
        new_state.cells[*index] = match state.cells[*index] {
            CellState::EMPTY => CellState::from(player),
            _ => panic!(),
        };
        new_state.last_player = player;
        return new_state;
    }
}

impl Scorer<GameBoard> for Strategy {
    fn score(&mut self, state: &GameBoard, player: Player) -> i32 {
        default_score(state.status(), player)
    }
}

impl Strategy {
    pub fn score_board_state(status: BoardStatus, player: Player) -> i32 {
        default_score(status, player)
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new(NullCache::default())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Instant;
    use crate::common::State;

    use crate::min_max::{Player, score_possible_moves};
    use crate::ttt::{BoardStatus, GameBoard, Strategy};

    #[test]
    fn status() {
        use crate::ttt::CellState::*;
        let board = GameBoard::new([EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY], Player::Max);
        assert_eq!(board.status(), BoardStatus::Ongoing);

        let board = GameBoard::new([X, X, X, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY], Player::Max);
        assert_eq!(board.status(), BoardStatus::MaxWon);

        let board = GameBoard::new([O, X, X, X, O, O, X, X, O], Player::Min);
        assert_eq!(board.status(), BoardStatus::MinWon);

        let board = GameBoard::new([O, O, O, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY], Player::Min);
        assert_eq!(board.status(), BoardStatus::MinWon);

        let board = GameBoard::new([X, O, O, O, X, X, EMPTY, O, O], Player::Min);
        assert_eq!(board.status(), BoardStatus::Ongoing);

        let board = GameBoard::new([X, O, O, O, X, X, X, O, O], Player::Max);
        assert_eq!(board.status(), BoardStatus::Draw);
    }

    #[test]
    fn empty_board() {
        let board = GameBoard::empty();
        let start = Instant::now();
        let scored_moves = score_possible_moves(&mut Strategy::default(), &board, u8::MAX);
        println!("search on empty board took {}mus", start.elapsed().as_micros());

        let scored_expanded_moves = scored_moves.iter()
            .flat_map(|m| m.expand())
            .collect::<HashSet<_>>();
        
        // Normal tick-tack-toe always results in a draw if both players play optimally. Therefore, the score of all moves should be 0.
        let scores: Vec<i32> = scored_expanded_moves.iter().map(|scored_move| scored_move.score).collect();
        assert_eq!(scores, vec![0; 9]);
    }
}