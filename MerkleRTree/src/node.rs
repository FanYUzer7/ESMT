use std::collections::{BTreeSet, VecDeque};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};
use types::hash_value::{ESMTHasher, HashValue};
use crate::shape::Rect;

pub trait FromPrimitive: Sized {
    fn from_i32(i: i32) -> Self;
}

pub trait  ToPrimitive: Sized {
    fn to_usize(self) -> usize;
}

pub trait MRTreeDefault: Default + Debug + Copy {}
pub trait MRTreeFunc:
PartialOrd + Sub<Output = Self> + Add<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Sized {
}

impl FromPrimitive for usize {
    #[inline]
    fn from_i32(i: i32) -> Self {
        i as usize
    }
}

impl ToPrimitive for usize {
    #[inline]
    fn to_usize(self) -> usize {
        self
    }
}

impl MRTreeDefault for usize {}
impl MRTreeFunc for usize {}

impl FromPrimitive for f32 {
    #[inline]
    fn from_i32(i: i32) -> Self {
        i as f32
    }
}

impl ToPrimitive for f32 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl MRTreeDefault for f32{}
impl MRTreeFunc for f32{}

impl FromPrimitive for f64 {
    #[inline]
    fn from_i32(i: i32) -> Self {
        i as f64
    }
}

impl ToPrimitive for f64 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl MRTreeDefault for f64{}
impl MRTreeFunc for f64{}

impl FromPrimitive for i32 {
    #[inline]
    fn from_i32(i: i32) -> Self {
        i
    }
}

impl ToPrimitive for i32 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl MRTreeDefault for i32{}
impl MRTreeFunc for i32{}

pub type Float = f32;
pub type UnsignedInteger = usize;
pub type Integer = i32;

/// `ObjectEntry`表示`ESMT`中的一个空间对象，只存在于叶子节点中。
#[derive(Clone)]
pub struct ObjectEntry<V, const D: usize>
    where
        V: MRTreeDefault,
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

pub(crate) struct Node<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    pub height: u32,
    pub mbr: Rect<V, D>,
    pub hash: HashValue,
    pub entry: Vec<ESMTEntry<V, D, C>>,
}

pub(crate) enum ESMTEntry<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    ENode(Node<V, D, C>),
    Object(ObjectEntry<V, D>)
}

impl<V, const D: usize> ObjectEntry<V, D>
    where
        V: MRTreeDefault,
{
    pub fn new(key: String, loc: [V; D], hash: HashValue) -> Self {
        Self {
            key,
            loc: Rect::new_point(loc),
            hash,
            stale: false,
        }
    }

    #[inline]
    pub fn hash(&self) -> HashValue {
        self.hash
    }

    #[inline]
    pub fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        self.hash.as_ref()
    }

    #[inline]
    pub fn loc(&self) -> &Rect<V, D> {
        &self.loc
    }

    #[inline]
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    #[inline]
    pub fn update_loc(&mut self, new_loc: Rect<V, D>) {
        self.loc = new_loc;
    }

    #[inline]
    pub fn delete(&mut self) {
        self.stale = true;
    }

    #[inline]
    pub fn refresh(&mut self) { self.stale = false; }

    #[inline]
    pub fn match_key(&self, key_2_match: &str) -> bool {
        self.key == key_2_match
    }
}

// todo: 返回Result，进行错误处理
impl<V, const D: usize, const C: usize> ESMTEntry<V, D, C>
    where
        V: MRTreeDefault,
{
    pub fn is_node(&self) -> bool {
        if let Self::ENode(_) = self {
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
            ESMTEntry::ENode(n) => {
                n.hash()
            }
            ESMTEntry::Object(o) => {
                o.hash()
            }
        }
    }

    pub fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        match self {
            ESMTEntry::ENode(n) => {
                n.hash_ref()
            }
            ESMTEntry::Object(o) => {
                o.hash_ref()
            }
        }
    }

    pub fn mbr(&self) -> &Rect<V, D> {
        match self {
            ESMTEntry::ENode(n) => {
                n.mbr()
            },
            ESMTEntry::Object(o) => {
                o.loc()
            },
        }
    }

    pub fn unpack_node(self) -> Node<V, D, C> {
        if let Self::ENode(n) = self {
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
        if let Self::ENode(n) = self {
            return n;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_node_mut(&mut self) -> &mut Node<V, D, C> {
        if let Self::ENode(n) = self {
            return n;
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
        V: MRTreeDefault,
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

    pub fn new_with_entry(height: u32, entry: Vec<ESMTEntry<V, D, C>>) -> Self {
        Self {
            height,
            mbr: Rect::default(),
            hash: HashValue::default(),
            entry,
        }
    }

    #[inline]
    pub fn hash(&self) -> HashValue {
        self.hash
    }

    #[inline]
    fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        self.hash.as_ref()
    }

    #[inline]
    pub fn is_overflow(&self) -> bool {
        self.entry.len() > Self::CAPACITY
    }

    #[inline]
    pub fn need_downcast(&self) -> bool {
        self.entry.len() < Self::MIN_FANOUT
    }

    pub fn rehash(&mut self) {
        let hash_set = self.entry.iter()
            .map(|e| e.hash_ref())
            .collect::<BTreeSet<_>>();
        let hasher = hash_set
            .into_iter()
            .fold(ESMTHasher::default(), |hasher, entry| {
                hasher.update(entry)
            });
        self.hash = hasher.finish();
    }

    #[inline]
    pub fn mbr(&self) -> &Rect<V, D> {
        &self.mbr
    }

    pub fn first_stale(&self) -> Option<usize> {
        // assert_eq!(self.height, 0, "first_stale method can only be called on Leaf node");
        for i in 0..self.entry.len() {
            if self.entry[i].is_object() {
                if self.entry[i].get_object().is_stale() {
                    return Some(i);
                }
            } else {
                break;
            }
        }
        None
    }

    #[inline]
    pub fn suitable_for_subtree(&self) -> bool {
        self.height == 0 || self.entry.len() >= Self::MIN_FANOUT
    }

    pub fn display(&self) -> (Vec<(u32, Rect<V, D>)>, Vec<(bool, Rect<V, D>)>) {
        let mut res = vec![];
        let mut objs = vec![];
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while !queue.is_empty() {
            let node = queue.pop_front().unwrap();
            res.push((node.height, node.mbr.clone()));
            if !(node.height == 0) {
                for entry in node.entry.iter() {
                    queue.push_back(entry.get_node());
                }
            } else {
                for entry in node.entry.iter() {
                    objs.push((entry.get_object().is_stale() ,entry.mbr().clone()))
                }
            }
        }
        (res, objs)
    }
}

impl<V, const D: usize, const C: usize> Node<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    pub fn choose_least_enlargement(&self, rect: &Rect<V, D>) -> usize {
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

    pub(crate) fn choose_subtree(&self, rect: &Rect<V, D>) -> usize {
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

    pub fn split_by_hilbert_sort(&mut self) -> Node<V, D, C> {
        let mut new_node = Self::new_with_height(self.height);
        let areas = self.entry.drain(..).collect::<Vec<_>>();
        let hilbert_sorter = HilbertSorter::new(&self.mbr);
        let mut sorted_entry = hilbert_sorter.sort(areas);
        let cnt_after_split = sorted_entry.len() - Self::MIN_FANOUT;
        self.entry.extend(sorted_entry.drain(..cnt_after_split));
        new_node.entry.extend(sorted_entry.into_iter());

        // recalculate mbr
        self.recalculate_state_after_sort();
        new_node.recalculate_state_after_sort();
        new_node
    }

    pub fn recalculate_mbr(&mut self) {
        if self.entry.is_empty() {
            return;
        }
        let mut rect = self.entry[0].mbr().clone();
        for i in 1..self.entry.len() {
            rect.expand(self.entry[i].mbr());
        }
        self.mbr = rect;
    }

    /// 重新计算哈希和mbr
    pub fn recalculate_state_after_sort(&mut self) {
        if self.entry.is_empty() {
            return;
        }
        let init_mbr = self.entry[0].mbr().clone();
        let (hash_set, mbr) = self.entry.iter()
            .fold((BTreeSet::new(), init_mbr), |(mut set, mut mbr), e| {
                mbr.expand(e.mbr());
                set.insert(e.hash_ref());
                (set, mbr)
            });
        let hasher = hash_set.into_iter()
            .fold(ESMTHasher::default(), |hasher, h| {
                hasher.update(h)
            });
        self.hash = hasher.finish();
        self.mbr = mbr;
    }
}

const _HILBERT3: [u8;64] = [
    0,3,4,5,58,59,60,63,
    1,2,7,6,57,56,61,62,
    14,13,8,9,54,55,50,49,
    15,12,11,10,53,52,51,48,
    16,17,30,31,32,33,46,47,
    19,18,29,28,35,34,45,44,
    20,23,24,27,36,39,40,43,
    21,22,25,26,37,38,41,42u8,
];

pub struct HilbertSorter<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    lowbound: [V; D],
    range: [V; D],
}

impl<V, const D: usize, const C: usize> HilbertSorter<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + FromPrimitive + ToPrimitive,
{
    pub fn new(area: &Rect<V, D>) -> Self {
        let mut range = [V::default(); D];
        for i in 0..D {
            range[i] = area._max[i] - area._min[i];
        }
        Self {
            lowbound: area._min.clone(),
            range,
        }
    }

    pub fn hilbert_idx(&self, obj: &Rect<V, D>) -> u8 {
        assert_eq!(D, 2, "only support 2-D now!");
        let obj_c = Self::center(obj);
        let mut x = (((obj_c[0] - self.lowbound[0]) * (V::from_i32(8))) / self.range[0]).to_usize();
        let mut y = (((obj_c[1] - self.lowbound[1]) * (V::from_i32(8))) / self.range[1]).to_usize();
        x = x - (x >> 3);
        y = y - (y >> 3);
        let idx = (y << 3) | x;
        _HILBERT3[idx]
    }

    pub(crate) fn sort(&self, v: Vec<ESMTEntry<V, D, C>>) -> Vec<ESMTEntry<V, D, C>> {
        // calculate hilebert index
        let mut indexed = v.into_iter()
            .map(|e| (self.hilbert_idx(e.mbr()), e))
            .collect::<Vec<_>>();
        // sort
        indexed.sort_by(|a, b| a.0.cmp(&b.0));
        // discard index
        indexed.into_iter()
            .map(|(hi, e)| {
                // println!("{}, loc: {:?}", hi, center(e.mbr()));
                e
            })
            .collect()
    }

    fn center(rect: &Rect<V, D>) -> [V; D] {
        let mut c = [V::default(); D];
        for i in 0..D {
            c[i] = (rect._max[i] + rect._min[i]) / (V::from_i32(2));
        }
        c
    }
}