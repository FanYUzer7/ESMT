use std::collections::BTreeSet;
use std::fmt::Debug;
use std::mem;
use types::hash_value::{ESMTHasher, HashValue};
use crate::shape::Rect;

pub type ValueSpace = f32;
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
    ENode(Node<V, D, C>),
    Object(ObjectEntry<V, D>)
}

impl<const D: usize> ObjectEntry<ValueSpace, D> {
    pub fn new(key: String, loc: [ValueSpace; D], hash: HashValue) -> Self {
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

    pub fn loc(&self) -> &Rect<ValueSpace, D> {
        &self.loc
    }

    pub fn is_stale(&self) -> bool {
        self.stale
    }

    pub fn update_loc(&mut self, new_loc: Rect<ValueSpace, D>) {
        self.loc = new_loc;
    }

    pub fn delete(&mut self) {
        self.stale = true;
    }
}

// todo: 返回Result，进行错误处理
impl<const D: usize, const C: usize> ESMTEntry<ValueSpace, D, C> {
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

    pub fn mbr(&self) -> &Rect<ValueSpace, D> {
        match self {
            ESMTEntry::ENode(n) => {
                n.mbr()
            },
            ESMTEntry::Object(o) => {
                o.loc()
            },
        }
    }

    pub fn unpack_node(self) -> Node<ValueSpace, D, C> {
        if let Self::ENode(n) = self {
            return n;
        }
        panic!("[ESMTEntry] expect reference of Node, find ObjectEntry");
    }

    pub fn unpack_object(self) -> ObjectEntry<ValueSpace, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_node(&self) -> &Node<ValueSpace, D, C> {
        if let Self::ENode(n) = self {
            return n;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_node_mut(&mut self) -> &mut Node<ValueSpace, D, C> {
        if let Self::ENode(n) = self {
            return n;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_object(&self) -> &ObjectEntry<ValueSpace, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }

    pub fn get_object_mut(&mut self) -> &mut ObjectEntry<ValueSpace, D> {
        if let Self::Object(obj) = self {
            return obj;
        }
        panic!("[ESMTEntry] expect ObjectEntry, find reference of Node");
    }
}

impl<const D: usize, const C: usize> Node<ValueSpace, D, C> {
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

    pub fn hash(&self) -> HashValue {
        self.hash
    }

    fn hash_ref(&self) -> &[u8; HashValue::LENGTH] {
        self.hash.as_ref()
    }

    fn rehash(&mut self) {
        let hash_set = self.entry.iter()
            .map(|e| e.hash())
            .collect::<BTreeSet<_>>();
        let hasher = hash_set
            .iter()
            .fold(ESMTHasher::default(), |mut hasher, entry| {
                hasher.update(entry.hash_ref())
            });
        self.hash = hasher.finish();
    }

    pub fn mbr(&self) -> &Rect<ValueSpace, D> {
        &self.mbr
    }

    fn choose_least_enlargement(&self, rect: &Rect<ValueSpace, D>) -> usize {
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

    fn choose_subtree(&self, rect: &Rect<ValueSpace, D>) -> usize {
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

    pub fn insert(&mut self, obj: ObjectEntry<ValueSpace, D>, height: u32) {
        if height == 0 {
            self.mbr.expand(obj.loc());
            self.entry.push(ESMTEntry::Object(obj));
        } else {
            let subtree_idx = self.choose_subtree(obj.loc());
            let mut node_mut = self.entry[subtree_idx].get_node_mut();
            node_mut.insert(obj, height - 1);
            // need to split
            if node_mut.entry.len() >= Self::CAPACITY {
                // 分裂并重新计算mbr
                let new_node = node_mut.split_by_hilbert_sort();
                self.entry.push(ESMTEntry::ENode(new_node));
                self.recalculate_mbr();
            } else {
                self.mbr.expand(node_mut.mbr());
            }
        }
        self.rehash();
    }

    fn split_by_hilbert_sort(&mut self) -> Node<ValueSpace, D, C> {
        let mut new_node = Self::new_with_height(self.height);
        let mut areas = self.entry.drain(..).collect::<Vec<_>>();
        let hilbert_sorter = HilbertSorter::new(&self.mbr);
        let mut sorted_entry = hilbert_sorter.sort(areas);
        let cnt_after_split = sorted_entry.len() - Self::MIN_FANOUT;
        self.entry.extend(sorted_entry.drain(..cnt_after_split));
        new_node.entry.extend(sorted_entry.into_iter());

        // recalculate mbr
        self.recalculate_state();
        new_node.recalculate_state();
        new_node
    }

    fn recalculate_mbr(&mut self) {
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
    fn recalculate_state_after_sort(&mut self) {
        if self.entry.is_empty() {
            return;
        }
        let init_mbr = self.entry[0].mbr().clone();
        let (hasher, mbr) = self.entry.iter()
            .fold((ESMTHasher::default(), init_mbr), |(mut hasher, mut mbr), e| {
                mbr.expand(e.mbr());
                (hasher.update(e.hash_ref()), mbr)
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
    19,18,29,28,25,34,45,44,
    20,23,24,27,36,39,40,43,
    21,22,25,26,37,38,41,42u8,
];

struct HilbertSorter<const D: usize, const C: usize> {
    lowbound: [ValueSpace; D],
    range: [ValueSpace; D],
}

impl<const D: usize, const C: usize> HilbertSorter<D, C> {
    pub fn new(area: &Rect<ValueSpace, D>) -> Self {
        let mut range = [ValueSpace::default(); D];
        for i in 0..D {
            range[i] = area._max[i] - area._min[i];
        }
        Self {
            lowbound: area._min.clone(),
            range,
        }
    }

    fn hilbert_idx(&self, obj: &Rect<ValueSpace, D>) -> u8 {
        assert_eq!(D, 2, "only support 2-D now!");
        let obj_c = center(obj);
        let mut idx = (((obj_c[0] - self.lowbound[0]) * 8 as ValueSpace) / self.range[0]) as usize;
        idx = idx * 8 + (((obj_c[1] - self.lowbound[1]) * 8 as ValueSpace) / self.range[1]) as usize;
        _HILBERT3[idx]
    }

    pub fn sort(&self, mut v: Vec<ESMTEntry<ValueSpace, D, C>>) -> Vec<ESMTEntry<ValueSpace, D, C>> {
        // calculate hilebert index
        let mut indexed = v.into_iter()
            .map(|e| (self.hilbert_idx(e.mbr()), e))
            .collect::<Vec<_>>();
        // sort
        indexed.sort_by(|a, b| a.0.cmp(&b.0));
        // discard index
        indexed.into_iter()
            .map(|(_, e)| e)
            .collect()
    }


}

fn center<const D: usize>(rect: &Rect<ValueSpace, D>) -> [ValueSpace; D] {
    let mut c = [ValueSpace::default(); D];
    for i in 0..D {
        c[i] = (rect._max[i] + rect._min[i]) / (2 as ValueSpace);
    }
    c
}

pub struct MerkleRTree<V, const D: usize, const C: usize>
    where
        V: Default + Debug + Copy,
{
    root: Option<Node<V, D, C>>,
    height: u32,
    len: usize,
}

impl<const D: usize, const C: usize> MerkleRTree<ValueSpace, D, C> {
    pub fn new() -> Self {
        Self {
            root: None,
            height: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn area(&self) -> Option<Rect<ValueSpace, D>> {
        match &self.root {
            None => { None }
            Some(root) => {
                Some(root.mbr().clone())
            }
        }
    }

    pub fn insert(&mut self, key: String, loc:[ValueSpace; D], hash: HashValue) {
        if self.root.is_none() {
            self.root = Some(Node::new_with_height(0));
        }
        let obj = ObjectEntry::new(key, loc, hash);
        let root = self.root.as_mut().unwrap();
        root.insert(obj, self.height);
        if root.entry.len() == Node::CAPACITY {
            self.height += 1;
            let mut new_root = Node::new_with_height(self.height);
            let another = root.split_by_hilbert_sort();
            let origin = self.root.take().unwrap();
            new_root.entry.push(ESMTEntry::ENode(origin));
            new_root.entry.push(ESMTEntry::ENode(another));
            self.root = Some(new_root);
        }
        self.len += 1;
    }
}


#[cfg(test)]
mod test {
    use std::time::Instant;
    use crate::shape::Rect;
    use rand::{thread_rng, Rng};
    use types::hash_value::HashValue;
    use crate::node::{ESMTEntry, HilbertSorter, ObjectEntry, ValueSpace};

    #[test]
    fn test_efficient() {
        let mut time = 0;
        for _ in 0..100 {
            let mut v = generate_random_rect();
            let sorter = HilbertSorter::<2, 40>::new(&Rect::new([0f32, 0f32], [100f32, 100f32]));
            let start = Instant::now();
            let sorted_v = {
                v.sort_by(|a, b| {
                    let a_idx = sorter.hilbert_idx(a.mbr());
                    let b_idx = sorter.hilbert_idx(b.mbr());
                    a_idx.cmp(&b_idx)
                });
                v
            };
            time += start.elapsed().as_micros();
        }
        println!("sort by func avg time = {}us", time / 100); //2200us

        time = 0;
        let mut time_p = 0;
        for _ in 0..100 {
            let mut v = generate_random_rect();
            let sorter = HilbertSorter::<2, 40>::new(&Rect::new([0f32, 0f32], [100f32, 100f32]));
            let start = Instant::now();
            let mut sorted_v = {
                let mut m = v.into_iter().map(|e| (sorter.hilbert_idx(e.mbr()), e)).collect::<Vec<_>>();
                m.sort_by(|a,b| a.0.cmp(&b.0));
                m.into_iter().map(|(_,e)| e).collect::<Vec<_>>()
            };
            time += start.elapsed().as_micros();
            sorted_v.pop();
            time_p += start.elapsed().as_micros();
        }
        println!("pack iter & unpack avg time = {}us", time / 100);
        println!("pack iter & unpack avg time = {}us", time_p / 100); //550us
    }

    fn generate_random_rect() -> Vec<ESMTEntry<ValueSpace, 2, 40>> {
        let mut rng = thread_rng();
        let mut v = vec![];
        for _ in 0..1000 {
            let p = rng.gen_range(0f32..100_f32);
            v.push(ESMTEntry::Object(ObjectEntry::new("key".to_string(), [p, p], HashValue::zero())));
        }
        v
    }
}