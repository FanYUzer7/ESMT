use std::collections::BinaryHeap;
use std::str::FromStr;
use MerkleRTree::shape::{Point, Rect};

const HILBERT3: [u8;64] = [
    0,3,4,5,58,59,60,63,
    1,2,7,6,57,56,61,62,
    14,13,8,9,54,55,50,49,
    15,12,11,10,53,52,51,48,
    16,17,30,31,32,33,46,47,
    19,18,29,28,25,34,45,44,
    20,23,24,27,36,39,40,43,
    21,22,25,26,37,38,41,42u8,
];

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let dots = args[1..].iter()
        .map(|s| i32::from_str(s).unwrap())
        .collect::<Vec<_>>();
    let mut p1 = Point::new([dots[0], dots[1]]);
    let mut p2 = Point::new([dots[2], dots[3]]);
    let mut r1 = Rect::new(p1, p2);
    let p3 = Point::new([dots[4], dots[5]]);
    let p4 = Point::new([dots[6], dots[7]]);
    let r2 = Rect::new(p3, p4);
    println!("get two rects: r1: {}, r2: {}", r1.display(), r2.display());
    println!("r1.area = {}, r2.area = {}", r1.area(), r2.area());
    println!("r1.largest_axis = {}, r2.largest_axis = {}", r1.largest_axis(), r2.largest_axis());
    println!("r1 contains r2? {}", r1.contains(&r2));
    println!("r1 intersects r2? {}", r1.intersects(&r2));
    println!("r2 is on r1's edge? {}", r1.on_edge(&r2));
    println!("rect distance between r1&r2: {}", r1.rect_dist(&r2));
    println!("r1&r2 overlap area: {}", r1.overlap_area(&r2));
    println!("r1&r2 union area: {}", r1.unioned_area(&r2));
    r1.expand(&r2);
    println!("r1 expand r2, {}", r1.display());
    println!("r1' area :{}", r1.area());
}
