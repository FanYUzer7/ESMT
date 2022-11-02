extern crate core;

use core::panicking::panic;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;
use types::hash_value::{ESMTHasher, HashValue};
use crate::shape::Rect;

pub mod shape;

/// `ObjectEntry`表示`ESMT`中的一个空间对象，只存在于叶子节点中。
pub struct ObjectEntry<V, const D: usize>
where
    V: Default + Debug + Copy,
{
    /// key: 空间对象在区块链数据库中的索引键值，如账户。
    key: String,
    /// 空间对象的空间位置
    loc: Rect<V, D>,
    /// 空间对象在区块链中所有状态集合的哈希值，如账户的哈希值
    hash: HashValue,
    /// 空间对象是否需要压缩，用于lazy update
    stale: bool,
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

impl<V, const D: usize> ObjectEntry<V, D>
where
    V: Default + Debug + Copy,
{
    pub fn new(key: String, loc: [V; D], hash: HashValue) -> Self {
        Self {
            key,
            loc: Rect::new_point(loc),
            hash,
            stale: false,
        }
    }

    pub fn hash(&self) -> HashValue {
        self.hash
    }

    pub fn loc(&self) -> &Rect<V, D> {
        &self.loc
    }

    pub fn is_stale(&self) -> bool {
        self.stale
    }

    pub fn update_loc(&mut self, new_loc: Rect<V, D>) {
        self.loc = new_loc;
    }

    pub fn delete(&mut self) {
        self.stale = true;
    }
}

// todo: 返回Result，进行错误处理
impl<V, const D: usize, const C: usize> ESMTEntry<V, D, C>
where
    V: Default + Debug + Copy,
    V: PartialOrd + Sub<Output=V> + Add<Output=V> + Mul<Output=V> + Div<Output=V> + From<i32>,
{
    pub fn is_node(&self) -> bool {
        if let Self::Node(_) = self {
            return true;
        }
        false
    }

    pub fn is_object(&self) -> bool {
        if let Self::Object(_) = self {
            return true;
        }
        false
    }

    pub fn hash(&self) -> HashValue {
        match self {
            ESMTEntry::Node(n) => {
                n.hash()
            }
            ESMTEntry::Object(o) => {
                o.hash()
            }
        }
    }

    pub fn unpack_node(self) -> Rc<Node<V, D, C>> {
        if let Self::Node(n) = self {
            return n;
        }
        panic!("[ESMTEntry] expect reference of Node, find ObjectEntry");
    }

    pub fn unpack_object(self) -> ObjectEntry<V, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_node(&self) -> Rc<Node<V, D, C>> {
        if let Self::Node(n) = self {
            return Rc::clone(n);
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_object(&self) -> &ObjectEntry<V, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_object_mut(&mut self) -> &mut ObjectEntry<V, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }
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

    pub fn new_with_height(height: u32) -> Self {
        Self {
            height,
            mbr: Rect::default(),
            hash: HashValue::default(),
            entry: vec![],
        }
    }

    pub fn hash(&self) -> HashValue {
        self.hash
    }

    pub fn rehash(&mut self) -> HashValue {
        let hasher = self.entry
            .iter()
            .fold(ESMTHasher::default(), |mut hasher, entry| {
                hasher.update(entry.hash().as_ref());
                hasher
            });
        self.hash = hasher.finish();
        self.hash
    }

    pub fn mbr(&self) -> &Rect<V, D> {
        &self.mbr
    }
}