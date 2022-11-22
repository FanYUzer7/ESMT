use std::collections::BTreeSet;
use rand::{Rng, thread_rng};
use types::hash_value::{ESMTHasher, HashValue};
use MerkleRTree::node::{MerkleRTree as Tree, HilbertSorter};
use MerkleRTree::shape::Rect;

fn main() {
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
    let mut root_hashes = vec![];
    let mut node_set = BTreeSet::new();
    // 插入0，1，2
    for i in 0..3usize {
        node_set.insert(hashes[i]);
        root_hashes.push(calc_hash(&node_set));
    }
    let h = vec![
        "762a02e8898f0a78ab0b08fcbc5a1a7f6af94f3d8bcc6255f000972c7fb0b835".to_string(),
        "3bc18bc99703ddb4806a4c9b3d77622f868485794555f2a82755b9b058a5853c".to_string(),
        "f1aeb9ad07cf28af64c862e7b5f6dc9b5bd900f81f88812caf651d79720516bc".to_string(),
        "7e061d9ea5d03d4fa8f0bcab2e63e575e978c1833e6e2209aa484ffc7daec65f".to_string(),
        "2dc9ac5321743fd711eba2e6d1bd43d682404f26a0b1f85bd6ea89b3187f180b".to_string(),
    ];
    // // 插入 3， 分裂
    // // hash4: 762a02e8898f0a78ab0b08fcbc5a1a7f6af94f3d8bcc6255f000972c7fb0b835
    // // hash5: 3bc18bc99703ddb4806a4c9b3d77622f868485794555f2a82755b9b058a5853c
    // // hash6: f1aeb9ad07cf28af64c862e7b5f6dc9b5bd900f81f88812caf651d79720516bc
    // // hash7: 4c608cf7438bf2f45c9316b2f12f8ba41b1148113ff3773c7f0907dba2dfc87b
    // // hash8: 9dba5db5c5b9e9212bda9b7561f538f6d1a83135819658d7b8e6ba6f9383c709
    // let mut tmp = vec![];
    // let mut node_set = BTreeSet::new();
    // node_set.insert(hashes[0]);
    // node_set.insert(hashes[4]);
    // node_set.insert(hashes[2]);
    // tmp.push(calc_hash(&node_set));
    // node_set.clear();
    // node_set.insert(hashes[1]);
    // node_set.insert(hashes[3]);
    // node_set.insert(hashes[7]);
    // tmp.push(calc_hash(&node_set));
    // node_set.clear();
    // node_set.insert(hashes[6]);
    // node_set.insert(hashes[5]);
    // tmp.push(calc_hash(&node_set));
    // node_set.clear();
    // node_set.extend(tmp);
    // println!("{}", calc_hash(&node_set).to_hex());
    // print_hilbert_idx();
    for s in h {
        let bytes = hex::decode(s).unwrap();
        root_hashes.push(HashValue::from_slice(&bytes).unwrap());
    }
    let mut tree = Tree::<usize, 2, 3>::new();
    for (idx, (node_hash, expected_root_hash)) in hashes.into_iter().zip(root_hashes.into_iter()).enumerate() {
        tree.insert("test".to_string(), points[idx].clone(), node_hash);
        assert_eq!(expected_root_hash, tree.root_hash().unwrap());
        println!("test-{} pass", idx);
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
    // println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point([1, 7])));
    // println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point([3, 3])));
    // println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point([4, 8])));
    // println!("{}", hilbert_sorter.hilbert_idx(&Rect::new_point([8, 1])));
}