use crate::expecti_min_max::{alpha_beta_star, Moves, Strategy as StrategyTrait};
use crate::game_controller::{GameController, Status};
use crate::min_max::cache::NullCache;
use crate::min_max::stats::NullStats;
use crate::min_max::Player;
use crate::{expecti_min_max, game_controller};
use rand::distr::StandardUniform;
use rand::prelude::*;
use std::fmt::{write, Display, Formatter, Write};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum DiceRoll {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl DiceRoll {
    fn as_i32(&self) -> i32 {
        match self {
            DiceRoll::One => 1,
            DiceRoll::Two => 2,
            DiceRoll::Three => 3,
            DiceRoll::Four => 4,
            DiceRoll::Five => 5,
            DiceRoll::Six => 6,
        }
    }
}

impl Display for DiceRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiceRoll::One => f.write_str("1"),
            DiceRoll::Two => f.write_str("2"),
            DiceRoll::Three => f.write_str("3"),
            DiceRoll::Four => f.write_str("4"),
            DiceRoll::Five => f.write_str("5"),
            DiceRoll::Six => f.write_str("6"),
        }
    }
}

impl Distribution<DiceRoll> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DiceRoll {
        match rng.random_range(1..=6) {
            1 => DiceRoll::One,
            2 => DiceRoll::Two,
            3 => DiceRoll::Three,
            4 => DiceRoll::Four,
            5 => DiceRoll::Five,
            6 => DiceRoll::Six,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Cell {
    Empty,
    Dice(DiceRoll),
}

impl Cell {
    fn as_i32(&self) -> i32 {
        match self {
            Cell::Empty => 0,
            Cell::Dice(d) => d.as_i32(),
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::Dice(d) => write!(f, "{}", d),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Row([Cell; 3]);

impl Row {
    fn empty() -> Self {
        Self([Cell::Empty; 3])
    }

    fn score(&self) -> i32 {
        let cells = &self.0;
        let mut i = 1;
        let mut previous = cells[0];
        let mut concecutives = 1;
        let mut score = 0;
        while i < cells.len() {
            if cells[i] == previous {
                concecutives += 1;
            } else {
                score += previous.as_i32() * concecutives * concecutives;
                previous = cells[i];
                concecutives = 1;
            }
            i += 1;
        }
        score += previous.as_i32() * concecutives * concecutives;
        score
    }

    fn add(&self, roll: DiceRoll) -> Row {
        if self.0[0] != Cell::Empty {
            panic!()
        }
        let mut cells = self.0.clone();
        cells[0] = Cell::Dice(roll);
        cells.sort();
        Row(cells)
    }

    fn remove(&self, roll: DiceRoll) -> Row {
        let mut cells = self.0.clone();
        for cell in cells.iter_mut() {
            if *cell == Cell::Dice(roll) {
                *cell = Cell::Empty;
            }
        }
        cells.sort();
        Row(cells)
    }

    fn is_full(&self) -> bool {
        !self.0.iter().any(|&cell| cell == Cell::Empty)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Side {
    rows: [Row; 3],
}

impl Side {
    fn empty() -> Self {
        Self {
            rows: [Row::empty(); 3],
        }
    }
    fn score(&self) -> i32 {
        self.rows.iter().map(|row| row.score()).sum()
    }

    fn update(&self, row: u8, update: impl Fn(&Row) -> Row) -> Side {
        Side {
            rows: core::array::from_fn(|i| {
                if i == row as usize {
                    update(&self.rows[i])
                } else {
                    self.rows[i].clone()
                }
            }),
        }
    }

    fn is_full(&self) -> bool {
        self.rows.iter().all(|row| row.is_full())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct State {
    max_side: Side,
    min_side: Side,
    dice_roll: Option<DiceRoll>,
    last_player: Player,
}

impl State {
    pub fn empty() -> Self {
        Self {
            max_side: Side::empty(),
            min_side: Side::empty(),
            dice_roll: None,
            last_player: Player::Max,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Move {
    Roll(DiceRoll),
    Place(u8),
}

pub struct Strategy {
    stats: NullStats,
}

impl Strategy {
    pub fn new() -> Self {
        Self {
            stats: NullStats::default(),
        }
    }
}

impl expecti_min_max::Strategy for Strategy {
    type State = State;
    type Move = Move;
    type Cache = NullCache;
    type Stats = NullStats;

    fn possible_moves(state: &State) -> Moves<Move, impl IntoIterator<Item = Move>> {
        match state.dice_roll {
            None => {
                if state.max_side.is_full() || state.min_side.is_full() {
                    Moves::Chance(vec![])
                } else {
                    use DiceRoll::*;
                    use Move::*;
                    Moves::Chance(vec![
                        Roll(Six),
                        Roll(Five),
                        Roll(Four),
                        Roll(Three),
                        Roll(Two),
                        Roll(One),
                    ])
                }
            }
            Some(_) => {
                let side = match state.last_player {
                    Player::Min => &state.min_side,
                    Player::Max => &state.max_side,
                };
                Moves::Player(
                    side.rows
                        .iter()
                        .enumerate()
                        .filter(|&(_, row)| !row.is_full())
                        .map(|(i, _)| Move::Place(i as u8)),
                )
            }
        }
    }

    fn do_move(&mut self, state: &State, _move: &Move, player: Player) -> State {
        match _move {
            Move::Roll(roll) => {
                if state.dice_roll.is_some() {
                    panic!()
                }
                State {
                    dice_roll: Some(*roll),
                    max_side: state.max_side.clone(),
                    min_side: state.min_side.clone(),
                    last_player: player,
                }
            }
            Move::Place(row_index) => match state.dice_roll {
                None => panic!(),
                Some(roll) => State {
                    dice_roll: None,
                    max_side: state.max_side.update(*row_index, |row| match player {
                        Player::Min => row.remove(roll),
                        Player::Max => row.add(roll),
                    }),
                    min_side: state.min_side.update(*row_index, |row| match player {
                        Player::Min => row.add(roll),
                        Player::Max => row.remove(roll),
                    }),
                    last_player: player,
                },
            },
        }
    }

    fn score(&mut self, state: &State, player: Player) -> i32 {
        let min_score = state.min_side.score();
        let max_score = state.max_side.score();
        match player {
            Player::Min => min_score - max_score,
            Player::Max => max_score - min_score,
        }
    }

    fn stats(&mut self) -> &mut Self::Stats {
        &mut self.stats
    }

    fn lowest_score() -> i32 {
        -Strategy::highest_score()
    }

    fn highest_score() -> i32 {
        6 * 3 * 3 * 3
    }
}

pub struct Knucklebones {
    rng: SmallRng,
    strategy: Strategy,
}

impl Knucklebones {
    pub fn new_random() -> Self {
        Self {
            rng: rand::make_rng(),
            strategy: Strategy::new(),
        }
    }

    fn roll(&mut self) -> Move {
        Move::Roll(self.rng.sample(StandardUniform))
    }
}

impl GameController for Knucklebones {
    type State = State;
    type Move = Move;

    fn initial(&mut self) -> Self::State {
        let state = State::empty();
        let roll = self.roll();
        self.strategy.do_move(&state, &roll, Player::Min)
    }

    fn do_move(&mut self, state: &Self::State, m: Self::Move) -> Result<Self::State, String> {
        let state = self.strategy.do_move(state, &m, Player::Min);
        let roll = self.roll();
        Ok(self.strategy.do_move(&state, &roll, Player::Max))
    }

    fn do_computer_move(&mut self, state: &Self::State) -> (Self::State, Self::Move) {
        let moves = alpha_beta_star(&mut self.strategy, state, 15);
        let _move = moves.choose(&mut self.rng).unwrap().min_max_move;
        let state = self.strategy.do_move(&state, &_move, Player::Max);
        let roll = self.roll();
        (self.strategy.do_move(&state, &roll, Player::Min), _move)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Roll(r) => {
                write!(f, "Rolled {}", r)
            }
            Move::Place(r) => {
                write!(f, "Placed at row {}", r + 1)
            }
        }
    }
}

impl FromStr for Move {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Move::Place(0)),
            "2" => Ok(Move::Place(1)),
            "3" => Ok(Move::Place(2)),
            _ => Err(format!("Invalid move {}, valid moves are 1, 2, 3", s)),
        }
    }
}

impl game_controller::Move for Move {}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..3 {
            writeln!(f, "-------")?;
            write!(f, "|")?;
            for j in 0..3 {
                write!(f, "{}|", self.max_side.rows[j].0[i])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "-------")?;
        writeln!(f)?;

        for i in 0..3 {
            writeln!(f, "-------")?;
            write!(f, "|")?;
            for j in 0..3 {
                write!(f, "{}|", self.min_side.rows[j].0[2 - i])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "-------")?;
        writeln!(f, " 1 2 3 ")?;
        if let Some(roll) = self.dice_roll {
            writeln!(f, "Roll: {}", roll)?;
        }
        Ok(())
    }
}

impl game_controller::State for State {
    fn player(&self) -> game_controller::Player {
        match self.last_player {
            Player::Min => game_controller::Player::Human,
            Player::Max => game_controller::Player::Computer,
        }
    }

    fn status(&self) -> Status {
        if self.min_side.is_full() || self.max_side.is_full() {
            let min_score = self.min_side.score();
            let max_score = self.max_side.score();
            if min_score > max_score {
                Status::Done {
                    winner: game_controller::Player::Human,
                }
            } else if max_score > min_score {
                Status::Done {
                    winner: game_controller::Player::Computer,
                }
            } else {
                Status::Draw
            }
        } else {
            Status::Playing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expecti_min_max::{alpha_beta_star, Strategy as StrategyT};
    use ahash::HashSet;

    #[test]
    fn row_is_full() {
        let row = Row::empty();
        assert_eq!(row.is_full(), false);

        let one = Cell::Dice(DiceRoll::One);
        let row = Row([Cell::Empty, one, one]);
        assert_eq!(row.is_full(), false);

        let row = Row([one, one, one]);
        assert_eq!(row.is_full(), true);
    }

    #[test]
    fn row_add() {
        let row = Row::empty();
        assert_eq!(row.0, [Cell::Empty, Cell::Empty, Cell::Empty]);

        let row = Row::empty();
        let row = row.add(DiceRoll::Three);
        assert_eq!(
            row.0,
            [Cell::Empty, Cell::Empty, Cell::Dice(DiceRoll::Three)]
        );
        let row = row.add(DiceRoll::One);
        assert_eq!(
            row.0,
            [
                Cell::Empty,
                Cell::Dice(DiceRoll::One),
                Cell::Dice(DiceRoll::Three)
            ]
        );
        let row = row.add(DiceRoll::Six);
        assert_eq!(
            row.0,
            [
                Cell::Dice(DiceRoll::One),
                Cell::Dice(DiceRoll::Three),
                Cell::Dice(DiceRoll::Six)
            ]
        );
    }

    #[test]
    fn row_score() {
        let row = Row::empty();
        assert_eq!(row.score(), 0);

        let row = Row::empty().add(DiceRoll::Six);
        assert_eq!(row.score(), 6);

        let row = Row::empty()
            .add(DiceRoll::One)
            .add(DiceRoll::Two)
            .add(DiceRoll::Three);
        assert_eq!(row.score(), 6);

        let row = Row::empty()
            .add(DiceRoll::Two)
            .add(DiceRoll::Two)
            .add(DiceRoll::Three);
        assert_eq!(row.score(), 11);

        let row = Row::empty()
            .add(DiceRoll::Six)
            .add(DiceRoll::Six)
            .add(DiceRoll::Six);
        assert_eq!(row.score(), 54);
    }

    #[test]
    fn possible_moves() {
        let mut state = State::empty();
        state.dice_roll = Some(DiceRoll::One);
        let moves = Strategy::possible_moves(&state);
        match moves {
            Moves::Player(moves) => {
                assert_eq!(
                    moves.into_iter().collect::<HashSet<_>>(),
                    [Move::Place(0), Move::Place(1), Move::Place(2)]
                        .into_iter()
                        .collect()
                )
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn score() {
        let mut strategy = Strategy::new();
        let mut state = State::empty();
        state.min_side = state.min_side.update(0, |r| r.add(DiceRoll::Six).add(DiceRoll::Three));
        state.max_side = state.max_side.update(2, |r| r.add(DiceRoll::Three));

        assert_eq!(strategy.score(&state, Player::Min), 6);
        assert_eq!(strategy.score(&state, Player::Max), -6);
    }

    #[test]
    fn alpha_beta_second_move() {
        let mut state = State::empty();
        state.min_side = state.min_side.update(0, |r| r.add(DiceRoll::Six));
        let mut strategy = Strategy::new();
        state = strategy.do_move(&state, &Move::Roll(DiceRoll::Six), Player::Max);

        let result = alpha_beta_star(&mut strategy, &state, 2);
        println!("{:?}", result);
    }
}
