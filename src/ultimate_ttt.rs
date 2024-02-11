use std::array;

use crate::ttt;
use crate::common::{BaseStrategy, Board, State};
use crate::min_max::{MoveSourceSink, Player, Scorer};
use crate::min_max::symmetry::{GridSymmetry, GridSymmetry3x3, GridSymmetry9x9, SymmetricMove3x3, Symmetry};

pub type BoardStatus = ttt::BoardStatus;
pub type CellState = ttt::CellState;

pub struct Move {
    ttt_board: usize,
    ttt_move: SymmetricMove3x3,
    symmetry: GridSymmetry3x3,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameBoard {
    pub cells: [[CellState; 9]; 9],
    pub last_player: Player,
    pub last_move: Option<(usize, usize)>,
}

impl GameBoard {
    pub fn ttt_board(&self, index: usize) -> ttt::GameBoard {
        return ttt::GameBoard {
            last_player: self.last_player,
            cells: self.cells[index],
        };
    }

    pub fn ttt_boards(&self) -> [ttt::GameBoard; 9] {
        array::from_fn(|index| self.ttt_board(index))
    }

    pub fn update_ttt_board(&mut self, index: usize, ttt_board: ttt::GameBoard) {
        self.cells[index] = ttt_board.cells;
    }

    pub fn symmetry(&self) -> GridSymmetry9x9 {
        !todo!();
        //GridSymmetry9x9::from(&self.cells)
    }
}

impl State for GameBoard {
    type BoardStatus = BoardStatus;

    fn status(&self) -> Self::BoardStatus {
        let statuses = self.ttt_boards().map(|board| board.status());
        let cells = statuses
            .map(|status| {
                match status {
                    BoardStatus::MaxWon => CellState::X,
                    BoardStatus::MinWon => CellState::O,
                    BoardStatus::Ongoing | BoardStatus::Draw => CellState::EMPTY,
                }
            });

        let overall_ttt_board = ttt::GameBoard { cells, last_player: self.last_player };
        let status = overall_ttt_board.status();
        match status {
            BoardStatus::Ongoing => {
                if statuses.iter().any(|status| status == &BoardStatus::Ongoing) {
                    BoardStatus::Ongoing
                } else {
                    BoardStatus::Draw
                }
            }
            _ => status,
        }
    }
}

impl Board for GameBoard {
    type Move = usize;
    type Symmetry = GridSymmetry3x3;

    fn symmetry(&self) -> Self::Symmetry {
        todo!()
    }

    fn last_player(&self) -> Player {
        todo!()
    }
}

pub type Strategy = BaseStrategy<GameBoard>;

impl MoveSourceSink<GameBoard, Move> for Strategy {
    fn possible_moves(state: &GameBoard) -> Vec<Move> {
        let forced_board_and_index = state.last_move.map(|(_, ttt_index)| (state.ttt_board(ttt_index), ttt_index));
        match forced_board_and_index {
            Some((ttt_board, ttt_index)) if ttt_board.status() == BoardStatus::Ongoing => {
                ttt::Strategy::possible_moves(&ttt_board).into_iter()
                    .map(|ttt_move| Move {
                        ttt_board: ttt_index,
                        ttt_move,
                        symmetry: GridSymmetry::none(),
                    })
                    .collect()
            }
            _ => {
                let moves_iter = (0..9).filter_map(|ttt_board_index| {
                    let ttt_board = state.ttt_board(ttt_board_index);
                    if ttt_board.status() != BoardStatus::Ongoing {
                        None
                    } else {
                        Some((ttt_board_index, ttt_board))
                    }
                }).flat_map(move |(ttt_board_index, ttt_board)| {
                    ttt::Strategy::possible_moves(&ttt_board).into_iter().map(move |ttt_move| Move {
                        ttt_move,
                        ttt_board: ttt_board_index,
                        symmetry: GridSymmetry::none(),
                    })
                });
                let mut moves = Vec::with_capacity(64);
                moves.extend(moves_iter);
                moves
            }
        }
    }

    fn do_move(state: &GameBoard, ultimate_move: &Move, player: Player) -> GameBoard {
        let ttt_board = state.ttt_board(ultimate_move.ttt_board);
        let new_ttt_board = ttt::Strategy::do_move(&ttt_board, &ultimate_move.ttt_move, player);

        let mut new_state = state.clone();
        new_state.update_ttt_board(ultimate_move.ttt_board, new_ttt_board);
        new_state
    }
}

impl Scorer<GameBoard> for Strategy {
    fn score(state: &GameBoard, player: Player) -> i32 {
        todo!()
    }
}

#[cfg(test)]
mod test {}