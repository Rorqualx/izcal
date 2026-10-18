//! Cost proxy for CS-D3: a borrow-heavy DOM-like loop, TokenCell vs RefCell.
//!
//! usage: bench <token|refcell|lend|nolend> [reps] [iters]
//!
//! - token / refcell: nodes in shuffled pointer-chasing order; per node, a
//!   mutable borrow of the parent, a shared borrow of the node and of its
//!   previous sibling, and a mutable borrow of the node. Prints ns per node
//!   visit for each repetition, then min and median.
//! - lend / nolend: cost of one lend+reenter pair (what every script call and
//!   callback pays under (a)) vs an empty call.
//!
//! Instruction counts come from running each mode under `/usr/bin/time -l`.

use std::cell::RefCell;
use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;
use token_cell::{TokenCell, enter, lend, reenter};

#[derive(Default, Clone)]
struct Data {
    child_count: u32,
    version: u64,
    text_len: u32,
    flags: u32,
}

struct TNode {
    parent: usize,
    prev: usize,
    data: TokenCell<Data>,
}
struct RNode {
    parent: usize,
    prev: usize,
    data: RefCell<Data>,
}

const N: usize = 1 << 14;

fn shape() -> Vec<(usize, usize, u32)> {
    // Deterministic pseudo-random tree: parent < i, prev is some earlier node.
    let mut x: u64 = 0x9E3779B97F4A7C15;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    (0..N)
        .map(|i| {
            let p = if i == 0 { 0 } else { (next() as usize) % i };
            let s = if i == 0 { 0 } else { (next() as usize) % i };
            (p, s, (next() % 64) as u32)
        })
        .collect()
}

fn order() -> Vec<usize> {
    let mut v: Vec<usize> = (0..N).collect();
    let mut x: u64 = 0xD1B54A32D192ED03;
    for i in (1..N).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        v.swap(i, (x as usize) % (i + 1));
    }
    v
}

#[inline(never)]
fn pass_token(nodes: &[Rc<TNode>], ord: &[usize], tok: &mut token_cell::ScriptToken) -> u64 {
    let mut acc = 0u64;
    for &i in ord {
        let n = &nodes[i];
        {
            let p = nodes[n.parent].data.borrow_mut(tok);
            p.child_count = p.child_count.wrapping_add(1);
            p.version = p.version.wrapping_add(i as u64);
        }
        {
            let d = n.data.borrow(tok);
            let s = nodes[n.prev].data.borrow(tok);
            acc = acc.wrapping_add(u64::from(d.text_len) + u64::from(s.flags) + d.version);
        }
        {
            let d = n.data.borrow_mut(tok);
            d.flags ^= acc as u32;
        }
    }
    acc
}

#[inline(never)]
fn pass_refcell(nodes: &[Rc<RNode>], ord: &[usize]) -> u64 {
    let mut acc = 0u64;
    for &i in ord {
        let n = &nodes[i];
        {
            let mut p = nodes[n.parent].data.borrow_mut();
            p.child_count = p.child_count.wrapping_add(1);
            p.version = p.version.wrapping_add(i as u64);
        }
        {
            let d = n.data.borrow();
            let s = nodes[n.prev].data.borrow();
            acc = acc.wrapping_add(u64::from(d.text_len) + u64::from(s.flags) + d.version);
        }
        {
            let mut d = n.data.borrow_mut();
            d.flags ^= acc as u32;
        }
    }
    acc
}

#[inline(never)]
fn lend_pairs(tok: &mut token_cell::ScriptToken, c: &TokenCell<u64>, iters: u64) {
    for _ in 0..iters {
        lend(tok, || reenter(|t| *c.borrow_mut(t) += black_box(1)));
    }
}

#[inline(never)]
fn plain_calls(c: &TokenCell<u64>, tok: &mut token_cell::ScriptToken, iters: u64) {
    for _ in 0..iters {
        *c.borrow_mut(tok) += black_box(1);
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("token");
    let reps: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(15);
    let iters: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(200);
    let sh = shape();
    let ord = order();
    let mut times = Vec::new();
    let mut check = 0u64;
    match mode {
        "token" => {
            let nodes: Vec<Rc<TNode>> = sh
                .iter()
                .map(|&(p, s, t)| {
                    Rc::new(TNode {
                        parent: p,
                        prev: s,
                        data: TokenCell::new(Data {
                            text_len: t,
                            ..Default::default()
                        }),
                    })
                })
                .collect();
            enter(|tok| {
                for _ in 0..reps {
                    let t0 = Instant::now();
                    for _ in 0..iters {
                        check =
                            check.wrapping_add(pass_token(black_box(&nodes), black_box(&ord), tok));
                    }
                    times.push(t0.elapsed().as_nanos() as f64 / (iters * N) as f64);
                }
            });
        }
        "refcell" => {
            let nodes: Vec<Rc<RNode>> = sh
                .iter()
                .map(|&(p, s, t)| {
                    Rc::new(RNode {
                        parent: p,
                        prev: s,
                        data: RefCell::new(Data {
                            text_len: t,
                            ..Default::default()
                        }),
                    })
                })
                .collect();
            for _ in 0..reps {
                let t0 = Instant::now();
                for _ in 0..iters {
                    check = check.wrapping_add(pass_refcell(black_box(&nodes), black_box(&ord)));
                }
                times.push(t0.elapsed().as_nanos() as f64 / (iters * N) as f64);
            }
        }
        "lend" | "nolend" => {
            let c = TokenCell::new(0u64);
            let n = (iters * N) as u64;
            enter(|tok| {
                for _ in 0..reps {
                    let t0 = Instant::now();
                    if mode == "lend" {
                        lend_pairs(tok, &c, n);
                    } else {
                        plain_calls(&c, tok, n);
                    }
                    times.push(t0.elapsed().as_nanos() as f64 / n as f64);
                }
                check = c.get(tok);
            });
        }
        _ => panic!("mode: token|refcell|lend|nolend"),
    }
    let mut s = times.clone();
    let med = median(&mut s);
    let min = s[0];
    println!(
        "mode={mode} reps={reps} iters={iters} nodes={N} ns_per_op: min={min:.4} median={med:.4} all={:?} check={check}",
        times
            .iter()
            .map(|t| (t * 10000.0).round() / 10000.0)
            .collect::<Vec<_>>()
    );
}
