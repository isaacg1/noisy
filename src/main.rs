extern crate rand;
use rand::{thread_rng, Rng};

use std::fmt;

const ROUND_LENGTH: usize = 100;
const NUM_ROUNDS: usize = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Move {
    Cooperate,
    Defect,
}
impl Move {
    fn opposite(self) -> Move {
        match self {
            Move::Cooperate => Move::Defect,
            Move::Defect => Move::Cooperate,
        }
    }
    fn flip(self, prob: f64) -> Move {
        assert!(0. <= prob);
        assert!(0.5 >= prob);
        if thread_rng().gen_range(0., 1.) < prob {
            self.opposite()
        } else {
            self
        }
    }
}

fn single_score(m1: Move, m2: Move) -> (u64, u64) {
    match (m1, m2) {
        (Move::Cooperate, Move::Cooperate) => (2, 2),
        (Move::Cooperate, Move::Defect) => (0, 3),
        (Move::Defect, Move::Cooperate) => (3, 0),
        (Move::Defect, Move::Defect) => (1, 1),
    }
}

trait Player: fmt::Debug {
    fn play(&self, my_moves: &Vec<Move>, their_noisy_moves: &Vec<Move>) -> Move;
}

// Output format is:
// let scores = play_all_pairs(players)
// scores[p1][p2] = <p1's score when playing against p2>
fn play_all_pairs(players: &Vec<&Player>) -> Vec<Vec<u64>> {
    let mut scores = vec![vec![0; players.len()]; players.len()];
    for i in 0..players.len() {
        for j in 0..i + 1 {
            let (p1_score, p2_score) = play_pair(players[i], players[j]);
            scores[i][j] = p1_score;
            scores[j][i] = p2_score;
        }
    }
    scores
}

fn play_pair(p1: &Player, p2: &Player) -> (u64, u64) {
    let (res1, res2): (Vec<u64>, Vec<u64>) = (0..NUM_ROUNDS).map(|_| play_round(p1, p2)).unzip();
    (res1.iter().sum(), res2.iter().sum())
}

fn play_round(p1: &Player, p2: &Player) -> (u64, u64) {
    let prob = thread_rng().gen_range(0., 0.5);
    let mut moves1 = vec![];
    let mut noisy1 = vec![];
    let mut score1 = 0;
    let mut moves2 = vec![];
    let mut noisy2 = vec![];
    let mut score2 = 0;
    for _ in 0..ROUND_LENGTH {
        let move1 = p1.play(&moves1, &noisy2);
        let move2 = p2.play(&moves2, &noisy1);
        let noise1 = move1.flip(prob);
        let noise2 = move2.flip(prob);

        let (ss1, ss2) = single_score(noise1, noise2);
        score1 += ss1;
        score2 += ss2;

        moves1.push(move1);
        moves2.push(move2);
        noisy1.push(noise1);
        noisy2.push(noise2);
    }
    (score1, score2)
}

pub fn average_scores(scores: &Vec<Vec<u64>>) -> Vec<f64> {
    scores
        .iter()
        .map(|row| row.iter().sum::<u64>() as f64 / row.len() as f64)
        .collect()
}

const REPS: usize = 5000;
const ALPHA: f64 = 1.;
fn page_rank(scores: &Vec<Vec<u64>>) -> Vec<f64> {
    let mut weights = vec![1.; scores.len()];
    for _ in 0..REPS {
        let unnorm_weights: Vec<f64> = scores
            .iter()
            .zip(&weights)
            .map(|(row, weight)| {
                weight * row.iter()
                    .zip(&weights)
                    .map(|(&s, &w)| (s as f64 * w).powf(ALPHA))
                    .sum::<f64>()
            })
            .collect();
        let weights_sum = unnorm_weights.iter().sum::<f64>();
        weights = unnorm_weights.iter().map(|w| w / weights_sum).collect();
    }
    weights
}

#[derive(Debug)]
struct Constant {
    cooperate_prob: f64,
}
impl Constant {
    fn new(cooperate_prob: f64) -> Self {
        Self { cooperate_prob }
    }
}
impl Player for Constant {
    fn play(&self, _my_moves: &Vec<Move>, _their_noisy_moves: &Vec<Move>) -> Move {
        if thread_rng().gen_range(0., 1.) < self.cooperate_prob {
            Move::Cooperate
        } else {
            Move::Defect
        }
    }
}

#[derive(Debug)]
struct TitForTat {
    default: Move,
    delay: usize,
}
impl TitForTat {
    fn new(default: Move, delay: usize) -> Self {
        Self { default, delay }
    }
}
impl Player for TitForTat {
    fn play(&self, _my_moves: &Vec<Move>, their_noisy_moves: &Vec<Move>) -> Move {
        if their_noisy_moves.len() < self.delay {
            self.default
        } else {
            if their_noisy_moves[their_noisy_moves.len() - self.delay..]
                .iter()
                .any(|&m| m != self.default)
            {
                self.default
            } else {
                self.default.opposite()
            }
        }
    }
}

#[derive(Debug)]
struct Threshold {
    start: usize,
    coop_thresh: f64,
}
impl Threshold {
    fn new(start: usize, coop_thresh: f64) -> Self {
        Self { start, coop_thresh }
    }
}
impl Player for Threshold {
    fn play(&self, _my_moves: &Vec<Move>, their_noisy_moves: &Vec<Move>) -> Move {
        if their_noisy_moves.len() < self.start {
            Move::Cooperate
        } else {
            let freq = their_noisy_moves
                .iter()
                .filter(|&&m| m == Move::Cooperate)
                .count() as f64 / their_noisy_moves.len() as f64;
            if freq >= self.coop_thresh {
                Move::Cooperate
            } else {
                Move::Defect
            }
        }
    }
}

const PLAYS: usize = 20;
fn main() {
    let (c1, c2, c3, c4, c5) = (
        Constant::new(0.),
        Constant::new(0.125),
        Constant::new(0.25),
        Constant::new(0.5),
        Constant::new(1.),
    );
    let (tt1, tt2, tt3, tt4) = (
        TitForTat::new(Move::Cooperate, 1),
        TitForTat::new(Move::Cooperate, 2),
        TitForTat::new(Move::Defect, 1),
        TitForTat::new(Move::Defect, 2),
    );
    let (th1, th2, th3, th4) = (
        Threshold::new(10, 0.5),
        Threshold::new(10, 0.7),
        Threshold::new(20, 0.5),
        Threshold::new(20, 0.7),
    );
    let players: Vec<&Player> = vec![
        &c1, &c2, &c3, &c4, &c5, &tt1, &tt2, &tt3, &tt4, &th1, &th2, &th3, &th4,
    ];
    let mut overall_ranks = vec![0.; players.len()];
    for _ in 0..PLAYS {
        let scores = play_all_pairs(&players);
        let page_ranks = page_rank(&scores);
        for (overall_rank, rank) in overall_ranks.iter_mut().zip(page_ranks) {
            *overall_rank += rank
        }
    }
    let mut players_and_ranks: Vec<(&Player, f64)> = players.iter().cloned().zip(overall_ranks).collect();
    players_and_ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (player, rank) in players_and_ranks {
        println!("{:?}: {:.6}", player, rank);
    }
}
