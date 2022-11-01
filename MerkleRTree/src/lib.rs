use std::fmt::Debug;
use crate::shape::Rect;

pub mod shape;

pub struct Leaf<V, const D: usize>
where
    V: Default + Debug + Copy,
{
    depth: u32,
    key: String,
    loc: Rect<V, D>,
    
}
