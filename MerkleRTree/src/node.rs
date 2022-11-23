use std::collections::{BTreeSet, VecDeque};
use std::fmt::Debug;
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

    pub fn match_key(&self, key_2_match: &str) -> bool {
        self.key == key_2_match
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

    pub fn is_overflow(&self) -> bool {
        self.entry.len() > Self::CAPACITY
    }

    pub fn need_downcast(&self) -> bool {
        self.entry.len() < Self::MIN_FANOUT
    }

    fn rehash(&mut self) {
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

    /// 插入，重新计算当前层的mbr以及下一层的hash
    pub fn insert(&mut self, obj: ESMTEntry<ValueSpace, D, C>, loc: &Rect<ValueSpace, D>, height: u32) {
        if height == 0 {
            if self.entry.is_empty() {
                self.entry.push(obj);
                self.recalculate_mbr();
            } else {
                self.mbr.expand(&loc);
                self.entry.push(obj);
            }
        } else {
            let subtree_idx = self.choose_subtree(&loc);
            let node_mut = self.entry[subtree_idx].get_node_mut();
            node_mut.insert(obj, loc, height - 1);
            // need to split
            if node_mut.entry.len() > Self::CAPACITY {
                // 分裂并重新计算mbr
                let new_node = node_mut.split_by_hilbert_sort();
                self.mbr.expand(new_node.mbr());
                self.entry.push(ESMTEntry::ENode(new_node));
            } else {
                node_mut.rehash();
            }
            self.mbr.expand(&loc);
        }
    }

    fn split_by_hilbert_sort(&mut self) -> Node<ValueSpace, D, C> {
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

    /// 删除时会重新计算每一层的mbr以及hash；是否发生下溢由上一层进行判断
    pub fn delete(&mut self, 
        rect: &Rect<ValueSpace, D>, 
        key: &str, 
        reinsert: &mut Vec<ESMTEntry<ValueSpace, D, C>>,
        height: u32,
    ) -> (Option<ESMTEntry<ValueSpace, D, C>>, bool) {
        if height == 0 {
            for i in 0..self.entry.len() {
                if self.entry[i].get_object().match_key(key) {
                    let to_delete = self.entry.swap_remove(i);
                    let recalced = self.mbr.on_edge(to_delete.mbr());
                    if recalced {
                        self.recalculate_mbr();
                    }
                    self.rehash();
                    return (
                        Some(to_delete),
                        recalced
                    );
                }
            }
        } else {
            for i in 0..self.entry.len() {
                if !rect.intersects(self.entry[i].mbr()) {
                    continue;
                }
                let node = self.entry[i].get_node_mut();
                let (removed, mut recalced) = node.delete(rect, key, reinsert, height - 1);
                if removed.is_none() {
                    continue;
                }
                if node.entry.len() < Self::MIN_FANOUT {
                    reinsert.extend(node.entry.drain(..));
                    let underflow_node = self.entry.swap_remove(i);
                    recalced = self.mbr.on_edge(underflow_node.mbr());
                }
                if recalced {
                    self.recalculate_mbr();
                }
                self.rehash();
                return (removed, recalced);
            }
        }
        (None, false)
    }

    pub fn display(&self) -> (Vec<(u32, Rect<ValueSpace, D>)>, Vec<Rect<ValueSpace, D>>) {
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
                    objs.push(entry.mbr().clone())
                }
            }
        }
        (res, objs)
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

pub struct HilbertSorter<const D: usize, const C: usize> {
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

    pub fn hilbert_idx(&self, obj: &Rect<ValueSpace, D>) -> u8 {
        assert_eq!(D, 2, "only support 2-D now!");
        let obj_c = center(obj);
        let mut x = (((obj_c[0] - self.lowbound[0]) * 8 as ValueSpace) / self.range[0]) as usize;
        let mut y = (((obj_c[1] - self.lowbound[1]) * 8 as ValueSpace) / self.range[1]) as usize;
        x = x - (x >> 3);
        y = y - (y >> 3);
        let idx = (x << 3) | y;
        _HILBERT3[idx]
    }

    pub fn sort(&self, v: Vec<ESMTEntry<ValueSpace, D, C>>) -> Vec<ESMTEntry<ValueSpace, D, C>> {
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
        let obj = ESMTEntry::Object(ObjectEntry::new(key, loc, hash));
        let obj_loc = obj.mbr().clone();
        self.insert_impl(obj, &obj_loc, self.height);
    }

    fn insert_impl(&mut self, entry: ESMTEntry<ValueSpace, D, C>, loc: &Rect<ValueSpace, D>, height: u32) {
        let root = self.root.as_mut().unwrap();
        root.insert(entry, loc, self.height);
        if root.is_overflow() {
            self.height += 1;
            let mut new_root = Node::new_with_height(self.height);
            let another = root.split_by_hilbert_sort();
            let origin = self.root.take().unwrap();
            new_root.entry.push(ESMTEntry::ENode(origin));
            new_root.entry.push(ESMTEntry::ENode(another));
            new_root.recalculate_state_after_sort();
            self.root = Some(new_root);
        } else {
            root.rehash();
        }
        self.len += 1;
    }

    pub fn delete(&mut self, key: &str, rect: &Rect<ValueSpace, D>) -> Option<ObjectEntry<ValueSpace, D>> {
        if let Some(root) = &mut self.root {
            let mut reinsert = Vec::new();
            let (removed, recalced) = root.delete(rect, key, &mut reinsert, self.height);
            if removed.is_none() {
                return None;
            }
            self.len -= 1;
            if self.height == 0 {
                if self.len == 0 {
                    self.root = None;
                }
            } else {
                if root.entry.len() == 1 {
                    let new_root = root.entry.pop().unwrap().unpack_node();
                    self.root = Some(new_root);
                    self.height -= 1;
                }              
            }
            // reinsert
            if !reinsert.is_empty() {
                self.reinsert(reinsert);
            }
            removed.map(|entry| entry.unpack_object())
        } else {
            None
        }
    }

    fn reinsert(&mut self, reinsert_list: Vec<ESMTEntry<ValueSpace, D, C>>) {
        for entry in reinsert_list.into_iter().rev() {
            let entry_loc = entry.mbr().clone();
            let expected_height_to_insert = if entry.is_node() {
                self.height - entry.get_node().height - 1
            } else {
                self.height
            };
            self.insert_impl(entry, &entry_loc, expected_height_to_insert);
        }
    }

    pub fn display(&self) -> (Vec<(u32, Rect<ValueSpace, D>)>, Vec<Rect<ValueSpace, D>>) {
        match &self.root {
            None => {
                (vec![], vec![])
            }
            Some(root) => {
                root.display()
            }
        }
    }

    pub fn root_hash(&self) -> Option<HashValue> {
        match &self.root {
            None => {
                None
            }
            Some(root) => {
                Some(root.hash())
            }
        }
    }
}


#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use std::time::Instant;
    use crate::shape::Rect;
    use rand::{thread_rng, Rng};
    use types::hash_value::{ESMTHasher, HashValue};
    use crate::node::{ESMTEntry, HilbertSorter, ObjectEntry, ValueSpace};
    use crate::node::MerkleRTree as Tree;

    #[test]
    fn test_efficient() {
        let mut time = 0;
        for _ in 0..100 {
            let mut v = generate_random_rect();
            let sorter = HilbertSorter::<2, 40>::new(&Rect::new([0, 0], [100, 100]));
            let start = Instant::now();
            let _sorted_v = {
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
            let v = generate_random_rect();
            let sorter = HilbertSorter::<2, 40>::new(&Rect::new([0, 0], [100, 100]));
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
            let p = rng.gen_range(0..100);
            v.push(ESMTEntry::Object(ObjectEntry::new("key".to_string(), [p, p], HashValue::zero())));
        }
        v
    }

    #[test]
    fn test_root_hash() {
        // let mut rng = thread_rng();
        let points = vec![
            [1usize, 8],
            [3, 9],
            [3, 6],
            [9, 2],
            [2, 7],
            [7, 1],
            [3, 1],
            [5, 8],
        ];
        let mut hashes = vec![];
        for i in 0..8 {
            hashes.push(hash(i));
        }
        let mut root_hashes = vec![];
        let mut node_set = BTreeSet::new();
        // 插入0，1，2
        for i in 0..3usize {
            node_set.insert(hashes[i]);
            root_hashes.push(calc_hash(&node_set));
        }
        let h = vec![
            "762a02e8898f0a78ab0b08fcbc5a1a7f6af94f3d8bcc6255f000972c7fb0b835".to_string(),
            "3bc18bc99703ddb4806a4c9b3d77622f868485794555f2a82755b9b058a5853c".to_string(),
            "f1aeb9ad07cf28af64c862e7b5f6dc9b5bd900f81f88812caf651d79720516bc".to_string(),
            "7e061d9ea5d03d4fa8f0bcab2e63e575e978c1833e6e2209aa484ffc7daec65f".to_string(),
            "2dc9ac5321743fd711eba2e6d1bd43d682404f26a0b1f85bd6ea89b3187f180b".to_string(),
        ];
        for s in h {
            let bytes = hex::decode(s).unwrap();
            root_hashes.push(HashValue::from_slice(&bytes).unwrap());
        }
        let mut tree = Tree::<usize, 2, 3>::new();
        for (idx, (node_hash, expected_root_hash)) in hashes.into_iter().zip(root_hashes.into_iter()).enumerate() {
            tree.insert("test".to_string(), points[idx].clone(), node_hash);
            assert_eq!(expected_root_hash, tree.root_hash().unwrap());
            println!("test-{} pass", idx);
        }
    }

    fn hash(data: i32) -> HashValue {
        let bytes = data.to_le_bytes();
        let hasher = ESMTHasher::default();
        hasher.update(&bytes).finish()
    }

    fn calc_hash(set: &BTreeSet<HashValue>) -> HashValue {
        let hasher = set.iter()
            .fold(ESMTHasher::default(), |h, hash| {
                h.update(hash.as_ref())
            });
        hasher.finish()
    }
}