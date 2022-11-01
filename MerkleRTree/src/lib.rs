use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;
use types::hash_value::HashValue;
use crate::shape::Rect;

pub mod shape;

pub struct ObjectEntry<V, const D: usize>
where
    V: Default + Debug + Copy,
{
    key: String,
    loc: Rect<V, D>,
    hash: HashValue,
}

pub struct Node<V, const D: usize, const C: usize>
where
    V: Default + Debug + Copy,
{
    height: u32,
    mbr: Rect<V, D>,
    hash: HashValue,
    entry: Vec<ESMTEntry<V, D, C>>,
}

pub enum ESMTEntry<V, const D: usize, const C: usize>
where
    V: Default + Debug + Copy,
{
    Node(Rc<Node<V, D, C>>),
    Object(ObjectEntry<V, D>)
}

impl<V, const D: usize, const C: usize> Node<V, D, C>
where
    V: Default + Debug + Copy,
    V: PartialOrd + Sub<Output=V> + Add<Output=V> + Mul<Output=V> + Div<Output=V> + From<i32>,
{
    pub const CAPACITY: usize = C;
    pub fn new() -> Self {
        Self {
            height: 0,
            mbr: Rect::default(),
            hash: HashValue::default(),
            entry: vec![],
        }
    }
}