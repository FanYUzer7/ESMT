use std::fmt::Debug;
use std::marker::PhantomData;
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

pub struct Node<V, const D: usize>
where
    V: Default + Debug + Copy,
{
    height: u32,
    mbr: Rect<V, D>,
    hash: HashValue,
    entry: Vec<ESMTEntry<V, D>>,
}

pub enum ESMTEntry<V, const D: usize>
where
    V: Default + Debug + Copy,
{
    Node(Rc<Node<V, D>>),
    Object(ObjectEntry<V, D>)
}
