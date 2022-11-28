use crate::node::{ESMTEntry, FromPrimitive, MRTreeDefault, MRTreeFunc, Node, ToPrimitive};
use crate::shape::Rect;

impl<V, const D: usize, const C: usize> Node<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{

    pub fn update(&mut self, key: &str, oloc: &Rect<V, D>, nloc: &Rect<V, D>) {

    }
}

