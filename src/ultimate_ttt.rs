use crate::ttt;
use crate::common::{BaseStrategy, Board, Cell, State};
use crate::iter_util::IterUtil;
use crate::min_max::{MoveSourceSink, Player, Scorer};
use crate::min_max::symmetry::{GridSymmetry3x3, SymmetricMove, SymmetricMove3x3, Symmetry};

pub type BoardStatus = ttt::BoardStatus;
pub type CellState = ttt::CellState;

#[derive(Clone, Debug)]
pub struct Move {
    ttt_board: SymmetricMove3x3,
    ttt_move: SymmetricMove3x3,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct SubBoard {
    cells: [CellState; 9],
    status: BoardStatus,
}

impl SubBoard {
    pub fn new(cells: [CellState; 9]) -> Self {
        let status = ttt::GameBoard { cells, last_player: Player::Max }.status();
        Self { cells, status }
    }
}

impl Cell for SubBoard {
    fn empty() -> Self {
        Self {
            cells: [CellState::EMPTY; 9],
            status: BoardStatus::Ongoing,
        }
    }
}

const CANONICAL_MAX_WIN_SUB_BOARD: SubBoard = SubBoard {
    cells: [CellState::X; 9],
    status: BoardStatus::MaxWon,
};

const CANONICAL_MIN_WIN_SUB_BOARD: SubBoard = SubBoard {
    cells: [CellState::O; 9],
    status: BoardStatus::MinWon,
};

const CANONICAL_DRAW_SUB_BOARD: SubBoard = SubBoard {
    cells: [
        CellState::X, CellState::O, CellState::X,
        CellState::X, CellState::O, CellState::O,
        CellState::O, CellState::X, CellState::X,
    ],
    status: BoardStatus::Draw,
};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameBoard {
    pub sub_boards: [SubBoard; 9],
    pub last_player: Player,
    pub last_move: Option<(u8, u8)>, // (board, cell)
}


struct SymmetricEq(SubBoard);

impl PartialEq for SymmetricEq {
    fn eq(&self, other: &Self) -> bool {
        if self.0.status != other.0.status {
            return false;
        }
        GridSymmetry3x3::is_same(&self.0.cells, &other.0.cells)
    }
}

impl Eq for SymmetricEq {}

impl GameBoard {
    pub fn ttt_board(&self, index: usize) -> ttt::GameBoard {
        return ttt::GameBoard {
            last_player: self.last_player,
            cells: self.sub_boards[index].cells,
        };
    }

    pub fn update_ttt_board(&mut self, index: usize, ttt_board: ttt::GameBoard) {
        self.sub_boards[index] = match ttt_board.status() {
            BoardStatus::MaxWon => CANONICAL_MAX_WIN_SUB_BOARD,
            BoardStatus::MinWon => CANONICAL_MIN_WIN_SUB_BOARD,
            BoardStatus::Draw => CANONICAL_DRAW_SUB_BOARD,
            BoardStatus::Ongoing => SubBoard { cells: ttt_board.cells, status: BoardStatus::Ongoing },
        };
    }

    pub fn symmetry(&self) -> GridSymmetry3x3 {
        GridSymmetry3x3::from(&self.sub_boards.map(|cells| SymmetricEq(cells)))
    }

    pub fn empty() -> Self {
        Self {
            sub_boards: [SubBoard::empty(); 9],
            last_player: Player::Max,
            last_move: None,
        }
    }
}

impl State for GameBoard {
    type BoardStatus = BoardStatus;

    fn status(&self) -> Self::BoardStatus {
        let statuses = self.sub_boards.map(|board| board.status);
        let cells = statuses
            .map(|status| {
                match status {
                    BoardStatus::MaxWon => CellState::X,
                    BoardStatus::MinWon => CellState::O,
                    BoardStatus::Ongoing | BoardStatus::Draw => CellState::EMPTY,
                }
            });

        let overall_ttt_board = ttt::GameBoard { cells, last_player: self.last_player };
        match overall_ttt_board.status() {
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
            status => status,
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
        let canonical_forced_board_index = state.last_move.map(|(_, ttt_index)| {
            let canonical_index = symmetry.canonicalize(&(ttt_index as usize));
            canonical_index
        });
        match canonical_forced_board_index {
            Some(board_index) if state.sub_boards[board_index].status == BoardStatus::Ongoing => {
                ttt::Strategy::possible_moves(&state.ttt_board(board_index)).into_iter()
                    .map(|ttt_move| Move {
                        ttt_board: SymmetricMove(board_index, symmetry.clone()),
                        ttt_move,
                    })
                    .collect_vec_with_capacity(7)
            }
            _ => {
                let symmetry_filter = symmetry.clone();
                let mut covered_index = [false; 9];
                let moves_iter = (0..9)
                    .filter_map(|ttt_board_index| {
                        if state.sub_boards[ttt_board_index].status != BoardStatus::Ongoing {
                            return None;
                        }
                        let canonical_index = symmetry_filter.canonicalize(&ttt_board_index);
                        if covered_index[canonical_index] {
                            return None;
                        }
                        covered_index[canonical_index] = true;
                        return Some(canonical_index);
                    })
                    .flat_map(|ttt_board_index| {
                        let ttt_board = state.ttt_board(ttt_board_index);
                        let symmetry = symmetry.clone();
                        ttt::Strategy::possible_moves(&ttt_board).into_iter().map(move |ttt_move| Move {
                            ttt_move,
                            ttt_board: SymmetricMove(ttt_board_index, symmetry.clone()),
                        }).collect_vec_with_capacity(7)
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
        new_state.last_move = Some((*ultimate_move.ttt_board.index() as u8, *ultimate_move.ttt_move.index() as u8));
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
                state.sub_boards.map(|board| ttt::Strategy::score_board_state(board.status, player)).iter().sum()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use std::time::Instant;
    use ahash::HashSet;
    use itertools::Itertools;
    use crate::min_max::score_possible_moves;
    use crate::ttt::CellState::{EMPTY as E, O, X};
    use super::*;

    #[test]
    fn test_status() {
        let ongoing = SubBoard::new([
            X, O, O,
            O, X, X,
            E, O, O
        ]);
        let draw = SubBoard::new([
            X, O, O,
            O, X, X,
            X, O, O
        ]);
        let min_won = SubBoard::new([
            O, X, X,
            X, O, O,
            X, O, O
        ]);
        let max_won = SubBoard::new([
            X, O, O,
            O, X, X,
            X, O, X
        ]);

        let board = GameBoard {
            sub_boards: [
                ongoing, draw, min_won,
                max_won, ongoing, draw,
                min_won, max_won, ongoing,
            ],
            last_player: Player::Max,
            last_move: None,
        };
        assert_eq!(board.status(), BoardStatus::Ongoing);

        let board = GameBoard {
            sub_boards: [
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
            sub_boards: [
                max_won, draw, min_won,
                max_won, min_won, draw,
                min_won, max_won, draw,
            ],
            last_player: Player::Min,
            last_move: Some((0, 0)),
        };
        assert_eq!(board.status(), BoardStatus::MinWon);

        let board = GameBoard {
            sub_boards: [
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
        let empty = SubBoard::new([
            E, E, E,
            E, E, E,
            E, E, E
        ]);

        let board = GameBoard {
            sub_boards: [
                empty, empty, empty,
                empty, empty, empty,
                empty, empty, empty,
            ],
            last_player: Player::Max,
            last_move: None,
        };

        let start = Instant::now();
        let scored_moves = score_possible_moves(&mut Strategy::new(), &board, 15);
        println!("search on empty board took {}ms", start.elapsed().as_millis());

        let best_move = scored_moves.into_iter().max_by_key(|m| m.score).map(|m| m.min_max_move).unwrap();

        assert_eq!(*best_move.ttt_board.index(), 4);
        assert_eq!(*best_move.ttt_move.index(), 4);
    }
}