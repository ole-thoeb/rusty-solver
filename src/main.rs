mod min_max;
mod stoplight;
mod ultimate_ttt;
mod ttt;
mod common;
mod iter_util;

extern crate lazy_static;

use crate::min_max::{Player, score_possible_moves};

fn main() {
    /*use crate::stoplight::CellState::*;
    let board = stoplight::GameBoard::new([EMPTY, GREEN, EMPTY, EMPTY, EMPTY, EMPTY, GREEN, EMPTY, EMPTY], Player::Min);

    let scored_moves = score_possible_moves(&mut stoplight::Strategy::new(), &stoplight::GameBoard::empty(), u8::MAX);
    println!("{:?}", scored_moves.iter().max_by_key(|m| m.score).unwrap());*/

    let mut strategy = ultimate_ttt::Strategy::new();
    let scored_moves = score_possible_moves(&mut strategy, &ultimate_ttt::GameBoard::empty(), 15);
    println!("{:?}", scored_moves.iter().max_by_key(|m| m.score).unwrap());
    println!("Stats: {:?}", strategy.stats);
}
