use std::collections::BTreeSet;
use std::env;
use std::str::FromStr;
use rand::{Rng, thread_rng};
use types::hash_value::{ESMTHasher, HashValue};
use MerkleRTree::node::{HilbertSorter};
use MerkleRTree::shape::Rect;
use MerkleRTree::mrtree::MerkleRTree as Tree;
use types::test_utils::{calc_hash, num_hash};

fn main() {
    let args: Vec<_> = env::args().collect();
    // let mut rng = thread_rng();
    let points = vec![
        [1usize, 6],
        [0, 5],
        [3, 2],
        [4, 5],
        [8, 5],
        [2, 8],
        [2, 3],
        [6, 7],
        [8, 0],
        [1, 1]
    ];
    let mut hashes = vec![];
    for i in 0..10 {
        hashes.push(num_hash(i));
    }
    let mut sorter = HilbertSorter::<usize, 2, 3>::new(&Rect::default());
    if args[1] == "-s" {
        assert!(args.len() == 3, "at least one point");
        let mut range = None;
        let mut range_set = vec![];
        let mut stack: Vec<Rect<usize, 2>> = vec![];
        let mut parse_stack = vec![];
        let parse_str = args[2].clone();
        let mut brack_cnt = 0;
        for &byte in parse_str.as_bytes() {
            match byte {
                91u8 => { // '['
                    parse_stack.push(byte);
                    brack_cnt += 1;
                }
                93u8 => { // ']'
                    if brack_cnt == 1 {
                        while let Some(ch) = parse_stack.pop() {
                            if ch != 91 {
                                range_set.push(stack.pop().unwrap());
                            } else {
                                brack_cnt -= 1;
                                break;
                            }
                        }
                        break;
                    }
                    let mut temp_range = None;
                    while let Some(ch) = parse_stack.pop() {
                        if ch != 91 {
                            if temp_range.is_some() {
                                let r: &mut Rect<usize, 2> = temp_range.as_mut().unwrap();
                                r.expand(&(stack.pop().unwrap()));
                            } else {
                                temp_range = Some(stack.pop().unwrap());
                            }
                        } else {
                            stack.push(temp_range.unwrap());
                            parse_stack.push(92u8);
                            brack_cnt -= 1;
                            break;
                        }
                    }
                }
                48..=57u8 => {
                    let idx = (byte - 48) as usize;
                    parse_stack.push(byte);
                    let rect = Rect::new_point(points[idx].clone());
                    stack.push(rect.clone());
                    if range.is_none() {
                        range = Some(rect);
                    } else {
                        let r: &mut Rect<usize, 2> = range.as_mut().unwrap();
                        r.expand(&rect);
                    }
                }
                _ => { continue; }
            }
        }
        sorter = HilbertSorter::new(range.as_ref().unwrap());
        for i in range_set.into_iter().rev() {
            let hilbert_idx = sorter.hilbert_idx(&i);
            println!("point [{}] hilbert idx = {}", i, hilbert_idx);
        }
    } else if args[1] == "-h" {
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
        println!("{:?}", stack.pop().unwrap());
    } else if args[1] == "-p" {
        assert_eq!(args.len(), 4, "please input correct parameter");
        let total = i32::from_str(&args[2]).unwrap();
        let cap = i32::from_str(&args[3]).unwrap();
        println!("packed node {:?}", pack_node(total, cap));
    } else if args[1] == "-r" {
        assert!(args.len() >= 3, "at least one point");
        let range_set = args
            .iter()
            .skip(2)
            .map(|s| usize::from_str(s).unwrap())
            .collect::<Vec<_>>();
        let mut range = Rect::<usize, 2>::new_point(points[range_set[0]]);
        for i in range_set.iter() {
            range.expand(&Rect::<usize,2>::new_point(points[*i]));
        }
        println!("{:?} area = {}",range, range.area());
        sorter = HilbertSorter::<usize, 2, 3>::new(&range);
    }
}

fn print_hilbert_idx() {
    let points = vec![
        [1usize, 6],
        [0, 5],
        [3, 2],
        [4, 5],
        [8, 5],
        [2, 8],
        [2, 3],
        [6, 7],
        [8, 0],
        [1, 1]
    ];
    let hilbert_sorter = HilbertSorter::<usize ,2, 3>::new(&Rect::new([3usize, 1], [9usize, 9]));
    for i in [1usize, 5, 3, 6] {
        println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point(points[i].clone())));
    }
}

fn pack_node(total: i32, cap: i32) -> Vec<i32> {
    assert!(total <= cap *cap, "total nums must no greater than cap2");
    let down = (cap + 1) / 2;
    let mut full_pack_size = cap;
    let full_pack_remain = total % cap;
    let full_pack_cnt = total / cap;
    if full_pack_remain == 0 {
        return vec![full_pack_size; full_pack_cnt as usize];
    }
    let mut res = vec![full_pack_size; (full_pack_cnt - 1) as usize];
    if full_pack_remain < down {
        res.push(full_pack_size + full_pack_remain - down);
        res.push(down);
    } else {
        res.push(full_pack_size);
        res.push(full_pack_remain);
    }
    res
}