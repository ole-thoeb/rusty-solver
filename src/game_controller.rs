use std::fmt::Display;
use std::io;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Player {
    Human,
    Computer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Draw,
    Done { winner: Player },
}

pub trait State: Display {
    fn player(&self) -> Player;
    fn status(&self) -> Status;
}

pub trait Move: Display + FromStr where <Self as FromStr>::Err: Display {}

pub trait GameController where <Self::Move as FromStr>::Err: Display {
    type State: State;
    type Move: Move;

    fn initial(&mut self) -> Self::State;
    fn do_move(&mut self, state: &Self::State, m: Self::Move) -> Result<Self::State, String>;
    fn do_computer_move(&mut self, state: &Self::State) -> (Self::State, Self::Move);
}

pub fn game_loop<GAME: GameController>(game: &mut GAME) {
    let mut state = game.initial();
    while matches!(state.status(), Status::Playing) {
        println!("{}", state);
        match state.player() {
            Player::Human => loop {
                let _move = read_move();
                match game.do_move(&state, _move) {
                    Ok(new_state) => {
                        state = new_state;
                        break;
                    }
                    Err(e) => {
                        println!("Error: {}", e)
                    }
                }
            },
            Player::Computer => {
                let (new_state, _move) = game.do_computer_move(&state);
                state = new_state;
                println!("Computer move: {}", _move);
            }
        }
    }
    println!(
        "Game ended: {}",
        match state.status() {
            Status::Playing => unreachable!(),
            Status::Draw => "Draw",
            Status::Done {
                winner: Player::Human,
            } => "You Won!",
            Status::Done {
                winner: Player::Computer,
            } => "You Lost!",
        }
    );
}

fn read_move<M: Move>() -> M where <M as FromStr>::Err: Display {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match M::from_str(&buffer.trim()) {
            Ok(m) => return m,
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
