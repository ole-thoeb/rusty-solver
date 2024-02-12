use std::array;
use itertools::Itertools;

use crate::ttt;
use crate::common::{BaseStrategy, Board, State};
use crate::min_max::{MoveSourceSink, Player, Scorer};
use crate::min_max::symmetry::{GridSymmetry3x3, SymmetricMove, SymmetricMove3x3, Symmetry};

pub type BoardStatus = ttt::BoardStatus;
pub type CellState = ttt::CellState;

#[derive(Clone, Debug)]
pub struct Move {
    ttt_board: SymmetricMove3x3,
    ttt_move: SymmetricMove3x3,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameBoard {
    pub cells: [[CellState; 9]; 9],
    pub last_player: Player,
    pub last_move: Option<(usize, usize)>, // (board, cell)
}


struct SymmetricEq([CellState; 9]);

impl PartialEq for SymmetricEq {
    fn eq(&self, other: &Self) -> bool {
        GridSymmetry3x3::is_same(&self.0, &other.0)
    }
}

impl Eq for SymmetricEq {}

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

    pub fn symmetry(&self) -> GridSymmetry3x3 {
        GridSymmetry3x3::from(&self.cells.map(|cells| SymmetricEq(cells)))
    }

    pub fn empty() -> Self {
        Self {
            cells: [[CellState::EMPTY; 9]; 9],
            last_player: Player::Max,
            last_move: None,
        }
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
                    let min_wins = statuses.iter().filter(|&status| status == &BoardStatus::MinWon).count();
                    let max_wins = statuses.iter().filter(|&status| status == &BoardStatus::MaxWon).count();
                    if min_wins > max_wins {
                        BoardStatus::MinWon
                    } else if max_wins > min_wins {
                        BoardStatus::MaxWon
                    } else {
                        BoardStatus::Draw
                    }
                }
            }
            _ => status,
        }
    }
}

impl Board for GameBoard {
    type Move = usize;

    fn last_player(&self) -> Player {
        self.last_player
    }
}

pub type Strategy = BaseStrategy<GameBoard>;

impl MoveSourceSink<GameBoard, Move> for Strategy {
    fn possible_moves(state: &GameBoard) -> Vec<Move> {
        let symmetry = state.symmetry();
        let forced_board_and_index = state.last_move.map(|(_, ttt_index)| {
            let canonical_index = symmetry.canonicalize(&ttt_index);
            (state.ttt_board(canonical_index), canonical_index)
        });
        match forced_board_and_index {
            Some((ttt_board, ttt_index)) if ttt_board.status() == BoardStatus::Ongoing => {
                ttt::Strategy::possible_moves(&ttt_board).into_iter()
                    .map(|ttt_move| Move {
                        ttt_board: SymmetricMove(ttt_index, symmetry.clone()),
                        ttt_move,
                    })
                    .collect()
            }
            _ => {
                let symmetry_filter = symmetry.clone();
                let moves_iter = (0..9).map(move |ttt_board_index| symmetry_filter.canonicalize(&ttt_board_index))
                    .unique()
                    .filter_map(|ttt_board_index| {
                        let ttt_board = state.ttt_board(ttt_board_index);
                        if ttt_board.status() != BoardStatus::Ongoing {
                            None
                        } else {
                            Some((ttt_board_index, ttt_board))
                        }
                    }).flat_map(move |(ttt_board_index, ttt_board)| {
                    let symmetry = symmetry.clone();
                    ttt::Strategy::possible_moves(&ttt_board).into_iter().map(move |ttt_move| Move {
                        ttt_move,
                        ttt_board: SymmetricMove(ttt_board_index, symmetry.clone()),
                    })
                });
                moves_iter.collect()
            }
        }
    }

    fn do_move(state: &GameBoard, ultimate_move: &Move, player: Player) -> GameBoard {
        let ttt_board = state.ttt_board(*ultimate_move.ttt_board.index());
        let new_ttt_board = ttt::Strategy::do_move(&ttt_board, &ultimate_move.ttt_move, player);

        let mut new_state = state.clone();
        new_state.update_ttt_board(*ultimate_move.ttt_board.index(), new_ttt_board);
        new_state.last_player = player;
        new_state.last_move = Some((*ultimate_move.ttt_board.index(), *ultimate_move.ttt_move.index()));
        new_state
    }
}

impl Scorer<GameBoard> for Strategy {
    fn score(state: &GameBoard, player: Player) -> i32 {
        match state.status() {
            BoardStatus::MaxWon => {
                if player == Player::Max {
                    1
                } else {
                    -1
                }
            }
            BoardStatus::MinWon => {
                if player == Player::Min {
                    1
                } else {
                    -1
                }
            }
            BoardStatus::Draw => 0,
            BoardStatus::Ongoing => {
                let ttt_scores = state.ttt_boards().map(|ttt_board| ttt::Strategy::score(&ttt_board, player));
                ttt_scores.iter().sum()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use std::time::Instant;
    use ahash::HashSet;
    use crate::min_max::score_possible_moves;
    use crate::ttt::CellState::{EMPTY as E, O, X};
    use super::*;

    #[test]
    fn test_status() {
        let ongoing = [
            X, O, O,
            O, X, X,
            E, O, O
        ];
        let draw = [
            X, O, O,
            O, X, X,
            X, O, O
        ];
        let min_won = [
            O, X, X,
            X, O, O,
            X, O, O
        ];
        let max_won = [
            X, O, O,
            O, X, X,
            X, O, X
        ];

        let board = GameBoard {
            cells: [
                ongoing, draw, min_won,
                max_won, ongoing, draw,
                min_won, max_won, ongoing,
            ],
            last_player: Player::Max,
            last_move: None,
        };
        assert_eq!(board.status(), BoardStatus::Ongoing);

        let board = GameBoard {
            cells: [
                ongoing, draw, min_won,
                max_won, min_won, draw,
                min_won, max_won, ongoing,
            ],
            last_player: Player::Max,
            last_move: Some((0, 0)),
        };
        assert_eq!(board.status(), BoardStatus::MinWon);

        // win by points
        let board = GameBoard {
            cells: [
                max_won, draw, min_won,
                max_won, min_won, draw,
                min_won, max_won, draw,
            ],
            last_player: Player::Min,
            last_move: Some((0, 0)),
        };
        assert_eq!(board.status(), BoardStatus::MinWon);

        let board = GameBoard {
            cells: [
                max_won, draw, min_won,
                min_won, min_won, draw,
                max_won, max_won, draw,
            ],
            last_player: Player::Min,
            last_move: Some((0, 0)),
        };
        assert_eq!(board.status(), BoardStatus::Draw);
    }

    #[test]
    fn first_possible_moves() {
        let board = GameBoard::empty();

        let moves = Strategy::possible_moves(&board);
        assert_eq!(moves.len(), 9);
        let groups = moves.into_iter().group_by(|m| *m.ttt_board.index());
        let moves_per_board = groups.into_iter().collect::<Vec<_>>();

        assert_eq!(moves_per_board.iter().map(|(index, _)| *index).collect::<HashSet<_>>(), HashSet::from_iter(vec![0, 1, 4]));

        for (_, moves) in moves_per_board {
            assert_eq!(moves.map(|m| *m.ttt_move.index()).collect::<HashSet<_>>(), HashSet::from_iter(vec![0, 1, 4]));
        }
    }

    #[test]
    fn first_move() {
        let empty = [
            E, E, E,
            E, E, E,
            E, E, E
        ];

        let board = GameBoard {
            cells: [
                empty, empty, empty,
                empty, empty, empty,
                empty, empty, empty,
            ],
            last_player: Player::Max,
            last_move: None,
        };

        let start = Instant::now();
        let scored_moves = score_possible_moves(&mut Strategy::new(), &board, 10);
        println!("search on empty board took {}ms", start.elapsed().as_millis());

        let best_move = scored_moves.into_iter().max_by_key(|m| m.score).map(|m| m.min_max_move).unwrap();

        assert_eq!(*best_move.ttt_board.index(), 4);
        assert_eq!(*best_move.ttt_move.index(), 4);
    }
}