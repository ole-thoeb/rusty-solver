mod min_max;
mod stoplight;
mod ultimate_ttt;
mod ttt;
mod common;
mod iter_util;

extern crate lazy_static;

use crate::min_max::{Player, score_possible_moves};
use crate::min_max::cache::NullCache;

fn main() {
    /*use crate::stoplight::CellState::*;
    let board = stoplight::GameBoard::new([EMPTY, GREEN, EMPTY, EMPTY, EMPTY, EMPTY, GREEN, EMPTY, EMPTY], Player::Min);

    let scored_moves = score_possible_moves(&mut stoplight::Strategy::new(), &stoplight::GameBoard::empty(), u8::MAX);
    println!("{:?}", scored_moves.iter().max_by_key(|m| m.score).unwrap());*/

    let mut strategy = ultimate_ttt::Strategy::new(NullCache::default());
    let now = std::time::Instant::now();
    let scored_moves = score_possible_moves(&mut strategy, &ultimate_ttt::GameBoard::empty(), 15);
    let time = now.elapsed().as_millis();
    println!("{:?}", scored_moves.iter().max_by_key(|m| m.score).unwrap());
    println!("Time: {}ms", time);
    println!("Stats: {:?}", strategy.stats);
}
