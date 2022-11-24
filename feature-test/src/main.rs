use std::collections::BTreeSet;
use std::env;
use std::str::FromStr;
use rand::{Rng, thread_rng};
use types::hash_value::{ESMTHasher, HashValue};
use MerkleRTree::node::{MerkleRTree as Tree, HilbertSorter};
use MerkleRTree::shape::Rect;

fn main() {
    let args: Vec<_> = env::args().collect();
    // let mut rng = thread_rng();
    let points = vec![
        [1usize, 8],
        [3, 9],
        [3, 6],
        [9, 2],
        [2, 7],
        [7, 1],
        [3, 1],
        [5, 8],
    ];
    let mut hashes = vec![];
    for i in 0..8 {
        hashes.push(hash(i));
    }
    if args[1] == "-s" {
        assert!(args.len() >= 3, "at least one point");
        let range_set = args
            .iter()
            .skip(2)
            .map(|s| usize::from_str(s).unwrap())
            .collect::<Vec<_>>();
        let mut range = Rect::<usize, 2>::new_point(points[0]);
        for i in range_set.iter() {
            range.expand(&Rect::<usize,2>::new_point(points[*i]));
        }
        let sorter = HilbertSorter::<2, 3>::new(&range);
        for i in range_set {
            let hilbert_idx = sorter.hilbert_idx(&Rect::<usize, 2>::new_point(points[i]));
            println!("point [{}] hilbert idx = {}", i, hilbert_idx);
        }
    } else {
        assert_eq!(args.len(), 3, "please input correct hash set");
        let mut stack = vec![];
        let mut parse_stack = vec![];
        let parse_str = args[2].clone();
        for &byte in parse_str.as_bytes() {
            match byte {
                91u8 => { // '['
                    parse_stack.push(byte);
                }
                93u8 => { // ']'
                    let mut temp_set = BTreeSet::new();
                    while let Some(ch) = parse_stack.pop() {
                        if ch != 91 {
                            temp_set.insert(stack.pop().unwrap());
                        } else {
                            stack.push(calc_hash(&temp_set));
                            parse_stack.push(92u8);
                            break;
                        }
                    }
                }
                48..=57u8 => {
                    let idx = (byte - 48) as usize;
                    parse_stack.push(byte);
                    stack.push(hashes[idx]);
                }
                _ => { continue; }
            }
        }
        assert_eq!(parse_stack.len(), 1, "parse error");
        println!("hashvalue: {:?}", stack.pop().unwrap());
    }
}

fn hash(data: i32) -> HashValue {
    let bytes = data.to_le_bytes();
    let hasher = ESMTHasher::default();
    hasher.update(&bytes).finish()
}

fn calc_hash(set: &BTreeSet<HashValue>) -> HashValue {
    let hasher = set.iter()
        .fold(ESMTHasher::default(), |h, hash| {
            h.update(hash.as_ref())
        });
    hasher.finish()
}

fn print_hilbert_idx() {
    let points = vec![
        [1usize, 8],
        [3, 9],
        [3, 6],
        [9, 2],
        [2, 7],
        [7, 1],
        [3, 1],
        [5, 8],
        [8, 2],
        [0, 4],
    ];
    let hilbert_sorter = HilbertSorter::<2, 3>::new(&Rect::new([3usize, 1], [9usize, 9]));
    for i in [1usize, 5, 3, 6] {
        println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point(points[i].clone())));
    }
}