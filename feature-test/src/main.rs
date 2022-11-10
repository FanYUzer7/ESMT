use std::collections::BinaryHeap;
use std::str::FromStr;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let num = f32::from_str(&args[1]).unwrap();
    println!("f32: {}",num);
    let unum: usize = num as usize;
    println!("usize: {}", unum);
}
