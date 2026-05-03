use crate::min_max::cache::Cache;
use crate::min_max::stats::Stats;
use crate::min_max::{Player, ScoredMove};
use itertools::Itertools;
use std::cmp::{max, min};

pub enum Moves<Move, PMoves> {
    Player(PMoves),
    Chance(Vec<Move>),
}

pub trait Strategy {
    type State;
    type Move;
    type Cache: Cache<Self::State>;
    type Stats: Stats;

    fn possible_moves(
        state: &Self::State,
    ) -> Moves<Self::Move, impl IntoIterator<Item = Self::Move>>;
    fn do_move(&mut self, state: &Self::State, _move: &Self::Move, player: Player) -> Self::State;

    fn score(&mut self, state: &Self::State, player: Player) -> i32;

    // fn cache(&mut self) -> &mut Self::Cache;
    fn stats(&mut self) -> &mut Self::Stats;

    fn lowest_score() -> i32;
    fn highest_score() -> i32;
}

pub fn alpha_beta_star<STRATEGY: Strategy>(
    strategy: &mut STRATEGY,
    state: &STRATEGY::State,
    max_level: u8,
) -> Vec<ScoredMove<STRATEGY::Move>> {
    score_possible_moves(strategy, state, max_level)
        .into_iter()
        .max_set_by_key(|state| state.score)
}

pub fn score_possible_moves<STRATEGY: Strategy>(
    strategy: &mut STRATEGY,
    state: &STRATEGY::State,
    max_level: u8,
) -> Vec<ScoredMove<STRATEGY::Move>> {
    let pos_moves = STRATEGY::possible_moves(&state);
    match pos_moves {
        Moves::Player(moves) => moves
            .into_iter()
            .map(|m| {
                let next_state = strategy.do_move(state, &m, Player::Max);
                let score = -alpha_beta_star_step(
                    strategy,
                    &next_state,
                    Player::Min,
                    max_level - 1,
                    STRATEGY::lowest_score(),
                    STRATEGY::highest_score(),
                );
                ScoredMove::new(score, m)
            })
            .collect(),
        Moves::Chance(_) => panic!("Chance must be resolved before finding optimal move"),
    }
}

// The *-Minimax Search Procedure for Trees Containing Chance Nodes - Section 5
// https://www.cs.uleth.ca/~benkoczi/3750/data/ballard83-star_alpha_beta.pdf
fn alpha_beta_star_step<STRATEGY: Strategy>(
    strategy: &mut STRATEGY,
    state: &STRATEGY::State,
    player: Player,
    remaining_levels: u8,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if remaining_levels == 0 {
        return strategy.score(state, player);
    }

    let moves = STRATEGY::possible_moves(state);
    match moves {
        Moves::Player(moves) => {
            let mut moves = moves.into_iter().peekable();
            // Check if this state is terminal i.e. no more moves can be made
            if moves.peek().is_none() {
                return strategy.score(state, player);
            }

            let mut max_score = -i32::MAX;
            for m in moves {
                let next_state = strategy.do_move(state, &m, player);
                max_score = max_score.max(-alpha_beta_star_step(
                    strategy,
                    &next_state,
                    !player,
                    remaining_levels - 1,
                    -beta,
                    -alpha,
                ));
                alpha = alpha.max(max_score);
                if alpha >= beta {
                    break;
                }
            }
            max_score
        }
        Moves::Chance(moves) => {
            if moves.is_empty() {
                return strategy.score(state, player);
            }
            let n = moves.len() as i32;

            let mut a = n * (alpha - STRATEGY::highest_score());
            let mut b = n * (beta - STRATEGY::lowest_score());

            let mut states = vec![];
            let mut probe_scores = vec![];
            for m in moves {
                let next_state = strategy.do_move(state, &m, player);
                a += STRATEGY::highest_score();
                let ax = max(a, STRATEGY::lowest_score());
                let bx = min(b, STRATEGY::highest_score());
                let probe_score = probe(strategy, &next_state, player, remaining_levels - 1, ax, bx);
                if probe_score <= a {
                    return alpha;
                }
                a -= probe_score;
                probe_scores.push(probe_score);
                states.push(next_state);
            }

            let mut sum = 0;
            for (next_state, probe_score) in states.iter().zip(probe_scores) {
                b += STRATEGY::lowest_score();
                a += probe_score;
                // Limit child α, β to n valid range
                let ax = max(a, STRATEGY::lowest_score());
                let bx = min(b, STRATEGY::highest_score());
                // Search the child with new cutoff values
                let score = alpha_beta_star_step(
                    strategy,
                    next_state,
                    player,
                    remaining_levels - 1,
                    ax,
                    bx,
                );
                // Check for α, β cutoff conditions
                if score <= n {
                    return alpha;
                }
                if score >= b {
                    return beta;
                }
                sum += score;
                // Adjust α, β for the next child
                a -= -score;
                b -= score;
            }
            // No cutoff occurred, return score
            sum / n
        }
    }
}

fn probe<STRATEGY: Strategy>(
    strategy: &mut STRATEGY,
    state: &STRATEGY::State,
    player: Player,
    remaining_levels: u8,
    alpha: i32,
    beta: i32,
) -> i32 {
    if remaining_levels == 0 {
        return strategy.score(state, player);
    }

    let moves = STRATEGY::possible_moves(state);
    match moves {
        Moves::Player(moves) => match moves.into_iter().next() {
            None => strategy.score(state, player),
            Some(m) => {
                let next_state = strategy.do_move(state, &m, player);
                alpha_beta_star_step(
                    strategy,
                    &next_state,
                    player,
                    remaining_levels - 1,
                    alpha,
                    beta,
                )
            }
        },
        Moves::Chance(_) => {
            panic!("Chance node must be followed by min or max node!")
        }
    }
}
