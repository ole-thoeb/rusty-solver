mod min_max;
mod stoplight;
mod ultimate_ttt;
mod ttt;
mod common;

extern crate lazy_static;

use crate::min_max::{Player, score_possible_moves};

fn main() {
    use crate::stoplight::CellState::*;
    let board = stoplight::GameBoard::new([EMPTY, GREEN, EMPTY, EMPTY, EMPTY, EMPTY, GREEN, EMPTY, EMPTY], Player::Min);

    let scored_moves = score_possible_moves(&mut stoplight::Strategy::new(), &stoplight::GameBoard::empty(), u8::MAX);
    println!("{:?}", scored_moves.iter().max_by_key(|m| m.score).unwrap());
}
