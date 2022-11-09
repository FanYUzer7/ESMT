use std::cell::RefCell;
use std::collections::BTreeMap;
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
    Node(Rc<RefCell<Node<V, D, C>>>),
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

    pub fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        self.hash.as_ref()
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
                n.borrow().hash()
            }
            ESMTEntry::Object(o) => {
                o.hash()
            }
        }
    }

    pub fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        match self {
            ESMTEntry::Node(n) => {
                n.borrow().hash_ref()
            }
            ESMTEntry::Object(o) => {
                o.hash_ref()
            }
        }
    }

    pub fn mbr(&self) -> &Rect<V, D> {
        match self {
            ESMTEntry::Node(n) => {
                n.borrow().mbr()
            },
            ESMTEntry::Object(o) => {
                o.loc()
            },
        }
        BTreeMap
    }

    pub fn unpack_node(self) -> Rc<RefCell<Node<V, D, C>>> {
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

    pub fn get_node(&self) -> &Node<V, D, C> {
        if let Self::Node(n) = self {
            return ;
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
    pub const MIN_FANOUT: usize = (Self::CAPACITY + 1) >> 1;
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

    fn hash(&self) -> HashValue {
        self.hash
    }

    fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        self.hash.as_ref()
    }

    fn rehash(&mut self) -> HashValue {
        let hasher = self.entry
            .iter()
            .fold(ESMTHasher::default(), |mut hasher, entry| {
                hasher.update(entry.hash_ref())
            });
        self.hash = hasher.finish();
        self.hash
    }

    fn mbr(&self) -> &Rect<V, D> {
        &self.mbr
    }

    fn choose_least_enlargement(&self, rect: &Rect<V, D>) -> usize {
        if D == 0 {
            return 0_usize;
        }
        let mut candidate_node_idx = 0_usize;
        let mut min_enlargement = rect._min[0];
        let mut min_node_area = rect._min[0];
        for (i, n) in self.entry.iter().enumerate() {
            let union_area = n.mbr().unioned_area(rect);
            let node_area = n.mbr().area();
            let enlargement = union_area - node_area;
            if i == 0 || enlargement < min_enlargement || (enlargement == min_enlargement && node_area < min_node_area){
                // 选择enlarge面积最小的节点，当enlarge面积相同时，选择面积最小的节点
                candidate_node_idx = i;
                min_enlargement = enlargement;
                min_node_area = node_area;
            }
        }
        candidate_node_idx
    }
    
    fn choose_subtree(&self, rect: &Rect<V, D>) -> usize {
        if D == 0 {
            return 0;
        }
        let mut subtree_idx = 0_usize;
        let mut found = false;
        let mut candidate_area = rect._max[0];
        for (i, n) in self.entry.iter().enumerate() {
            if n.mbr().contains(rect) {
                let area = n.mbr().area();
                // 在所有包含的节点中选择面积最小的节点
                if !found || area < candidate_area {
                    candidate_area = area;
                    subtree_idx = i;
                    found = true;
                }
            }
        }
        if !found {
            subtree_idx = self.choose_least_enlargement(rect);
        }
        subtree_idx
    }
}