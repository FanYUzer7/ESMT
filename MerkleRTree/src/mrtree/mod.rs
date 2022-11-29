use std::collections::BTreeSet;
use types::hash_value::{ESMTHasher, HashValue};
use crate::node::{ESMTEntry, FromPrimitive, HilbertSorter, MRTreeDefault, MRTreeFunc, Node, ObjectEntry, ToPrimitive};
use crate::shape::Rect;

struct MerkleRTreeNode<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    node: Node<V, D, C>,
}

impl<V, const D: usize, const C: usize> MerkleRTreeNode<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    pub fn new(node: Node<V, D, C>) -> Self {
        Self {
            node
        }
    }

    pub fn new_with_height(height: u32) -> Self {
        Self {
            node: Node::new_with_height(height),
        }
    }

    /// 插入，重新计算当前层的mbr以及下一层的hash
    fn insert_by_mrt(node: &mut Node<V, D, C>, obj: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
        if height == 0 {
            if node.entry.is_empty() {
                node.entry.push(obj);
                node.recalculate_mbr();
            } else {
                node.mbr.expand(&loc);
                node.entry.push(obj);
            }
        } else {
            let subtree_idx = node.choose_subtree(&loc);
            let node_mut = node.entry[subtree_idx].get_node_mut();
            // node_mut.insert_by_mrt(obj, loc, height - 1);
            Self::insert_by_mrt(node_mut, obj, loc, height - 1);
            // need to split
            if node_mut.entry.len() > Node::<V, D, C>::CAPACITY {
                // 分裂并重新计算mbr
                let new_node = node_mut.split_by_hilbert_sort();
                node.mbr.expand(new_node.mbr());
                node.mbr.expand(node_mut.mbr());
                node.entry.push(ESMTEntry::ENode(new_node));
            } else {
                node_mut.rehash();
            }
            node.mbr.expand(&loc);
        }
    }

    pub fn insert(&mut self, obj: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
        Self::insert_by_mrt(&mut self.node, obj, loc, height);
    }

    /// 删除时会重新计算每一层的mbr以及hash；是否发生下溢由上一层进行判断
    fn delete_by_mrt(node: &mut Node<V, D, C>,
                     rect: &Rect<V, D>,
                     key: &str,
                     reinsert: &mut Vec<ESMTEntry<V, D, C>>,
                     height: u32,
    ) -> (Option<ESMTEntry<V, D, C>>, bool) {
        if height == 0 {
            for i in 0..node.entry.len() {
                if node.entry[i].get_object().match_key(key) {
                    let to_delete = node.entry.swap_remove(i);
                    let recalced = node.mbr.on_edge(to_delete.mbr());
                    if recalced {
                        node.recalculate_mbr();
                    }
                    node.rehash();
                    return (
                        Some(to_delete),
                        recalced
                    );
                }
            }
        } else {
            for i in 0..node.entry.len() {
                if !rect.intersects(node.entry[i].mbr()) {
                    continue;
                }
                let child = node.entry[i].get_node_mut();
                // let (removed, mut recalced) = node.delete_by_mrt(rect, key, reinsert, height - 1);
                let (removed, mut recalced) =
                    Self::delete_by_mrt(child, rect, key, reinsert, height - 1);
                if removed.is_none() {
                    continue;
                }
                if child.entry.len() < Node::<V, D, C>::MIN_FANOUT {
                    reinsert.extend(child.entry.drain(..));
                    let underflow_node = node.entry.swap_remove(i);
                    recalced = node.mbr.on_edge(underflow_node.mbr());
                }
                if recalced {
                    node.recalculate_mbr();
                }
                node.rehash();
                return (removed, recalced);
            }
        }
        (None, false)
    }

    pub fn delete(&mut self,
                  rect: &Rect<V, D>,
                  key: &str,
                  reinsert: &mut Vec<ESMTEntry<V, D, C>>,
                  height: u32,
    ) -> (Option<ESMTEntry<V, D, C>>, bool) {
        Self::delete_by_mrt(&mut self.node, rect, key, reinsert, height)
    }

    #[inline]
    pub fn unpack_node(self) -> Node<V, D, C> {
        self.node
    }
}

// impl<V, const D: usize, const C: usize> Node<V, D, C>
//     where
//         V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
// {
//     /// 插入，重新计算当前层的mbr以及下一层的hash
//     pub fn insert_by_mrt(&mut self, obj: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
//         if height == 0 {
//             if self.entry.is_empty() {
//                 self.entry.push(obj);
//                 self.recalculate_mbr();
//             } else {
//                 self.mbr.expand(&loc);
//                 self.entry.push(obj);
//             }
//         } else {
//             let subtree_idx = self.choose_subtree(&loc);
//             let node_mut = self.entry[subtree_idx].get_node_mut();
//             node_mut.insert_by_mrt(obj, loc, height - 1);
//             // need to split
//             if node_mut.entry.len() > Self::CAPACITY {
//                 // 分裂并重新计算mbr
//                 let new_node = node_mut.split_by_hilbert_sort();
//                 self.mbr.expand(new_node.mbr());
//                 self.entry.push(ESMTEntry::ENode(new_node));
//             } else {
//                 node_mut.rehash();
//             }
//             self.mbr.expand(&loc);
//         }
//     }
//
//     /// 删除时会重新计算每一层的mbr以及hash；是否发生下溢由上一层进行判断
//     pub fn delete_by_mrt(&mut self,
//                          rect: &Rect<V, D>,
//                          key: &str,
//                          reinsert: &mut Vec<ESMTEntry<V, D, C>>,
//                          height: u32,
//     ) -> (Option<ESMTEntry<V, D, C>>, bool) {
//         if height == 0 {
//             for i in 0..self.entry.len() {
//                 if self.entry[i].get_object().match_key(key) {
//                     let to_delete = self.entry.swap_remove(i);
//                     let recalced = self.mbr.on_edge(to_delete.mbr());
//                     if recalced {
//                         self.recalculate_mbr();
//                     }
//                     self.rehash();
//                     return (
//                         Some(to_delete),
//                         recalced
//                     );
//                 }
//             }
//         } else {
//             for i in 0..self.entry.len() {
//                 if !rect.intersects(self.entry[i].mbr()) {
//                     continue;
//                 }
//                 let node = self.entry[i].get_node_mut();
//                 let (removed, mut recalced) = node.delete_by_mrt(rect, key, reinsert, height - 1);
//                 if removed.is_none() {
//                     continue;
//                 }
//                 if node.entry.len() < Self::MIN_FANOUT {
//                     reinsert.extend(node.entry.drain(..));
//                     let underflow_node = self.entry.swap_remove(i);
//                     recalced = self.mbr.on_edge(underflow_node.mbr());
//                 }
//                 if recalced {
//                     self.recalculate_mbr();
//                 }
//                 self.rehash();
//                 return (removed, recalced);
//             }
//         }
//         (None, false)
//     }
// }

pub struct MerkleRTree<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    root: Option<MerkleRTreeNode<V, D, C>>,
    height: u32,
    len: usize,
}

impl<V, const D: usize, const C: usize> MerkleRTree<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    pub fn new() -> Self {
        Self {
            root: None,
            height: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn area(&self) -> Option<Rect<V, D>> {
        match &self.root {
            None => { None }
            Some(root) => {
                Some(root.node.mbr().clone())
            }
        }
    }

    pub fn insert(&mut self, key: String, loc:[V; D], hash: HashValue) {
        if self.root.is_none() {
            self.root = Some(MerkleRTreeNode::new_with_height(0));
        }
        let obj = ESMTEntry::Object(ObjectEntry::new(key, loc, hash));
        let obj_loc = obj.mbr().clone();
        self.insert_impl(obj, &obj_loc, self.height);
    }

    fn insert_impl(&mut self, entry: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
        let root = self.root.as_mut().unwrap();
        root.insert(entry, loc, height);
        let need_split = root.node.is_overflow();
        if need_split {
            self.height += 1;
            let mut new_root = Node::new_with_height(self.height);
            let mut origin = self.root.take().unwrap().unpack_node();
            let another = origin.split_by_hilbert_sort();
            new_root.entry.push(ESMTEntry::ENode(origin));
            new_root.entry.push(ESMTEntry::ENode(another));
            new_root.recalculate_state_after_sort();
            self.root = Some(MerkleRTreeNode::new(new_root));
        } else {
            root.node.rehash()
        }
        self.len += 1;
    }

    pub fn delete(&mut self, key: &str, rect: &[V;D]) -> Option<ObjectEntry<V, D>> {
        if let Some(root) = &mut self.root {
            let oloc = Rect::new_point(rect.clone());
            let mut reinsert = Vec::new();
            let (removed, _) = root.delete(&oloc, key, &mut reinsert, self.height);
            if removed.is_none() {
                return None;
            }
            self.len -= 1;
            if self.height == 0 {
                if self.len == 0 {
                    self.root = None;
                }
            } else {
                if root.node.entry.len() == 1 {
                    println!("root downcast. original height: {}", self.height);
                    let new_root = root.node.entry.pop().unwrap().unpack_node();
                    self.height = new_root.height;
                    self.root = Some(MerkleRTreeNode::new(new_root));
                }
            }
            // reinsert
            if !reinsert.is_empty() {
                println!("need re-insert");
                self.reinsert(reinsert);
            }
            removed.map(|entry| entry.unpack_object())
        } else {
            None
        }
    }

    pub fn update_loc(&mut self, key:&str, oloc: &[V; D], nloc: [V; D]) {
        let old = self.delete(key, oloc);
        if old.is_none() {
            println!("obj {} not exist. loc: {:?}", key, oloc);
        } else {
            let mut obj = old.unwrap();
            let new_loc = Rect::new_point(nloc);
            obj.update_loc(new_loc.clone());
            self.insert_impl(ESMTEntry::Object(obj), &new_loc, self.height);
        }
    }

    fn reinsert(&mut self, reinsert_list: Vec<ESMTEntry<V, D, C>>) {
        for entry in reinsert_list.into_iter().rev() {
            // println!("start reinsert. Current height: {}", self.height);
            let entry_loc = entry.mbr().clone();
            let expected_height_to_insert = if entry.is_node() {
                // println!("re-insert node");
                self.height - entry.get_node().height - 1
            } else {
                // println!("re-insert object");
                self.height
            };
            self.insert_impl(entry, &entry_loc, expected_height_to_insert);
        }
    }

    pub fn display(&self) -> (Vec<(u32, Rect<V, D>)>, Vec<(bool, Rect<V, D>)>) {
        match &self.root {
            None => {
                (vec![], vec![])
            }
            Some(root) => {
                root.node.display()
            }
        }
    }

    pub fn root_hash(&self) -> Option<HashValue> {
        match &self.root {
            None => {
                None
            }
            Some(root) => {
                Some(root.node.hash())
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
    use crate::node::{ESMTEntry, HilbertSorter, Integer, Node, ObjectEntry, UnsignedInteger};
    use crate::mrtree::MerkleRTree as Tree;

    #[test]
    fn test_efficient() {
        let mut time = 0;
        for _ in 0..100 {
            let mut v = generate_random_rect();
            let sorter = HilbertSorter::<Integer, 2, 40>::new(&Rect::new([0, 0], [100, 100]));
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
            let sorter = HilbertSorter::<Integer ,2, 40>::new(&Rect::new([0, 0], [100, 100]));
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

    fn generate_random_rect() -> Vec<ESMTEntry<Integer, 2, 40>> {
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
            "0bd13fbae340f13bc8580b2d777c5393652a2e4fce220bb618b156b8cf97b90f".to_string(),
            "7b5b68e400187a7c07f1af2043315dee22517f0919cfd1df1b21a319b0bb04e4".to_string(),
            "902d1aaa9fdedf73a5cb2e289a941d7baed0db1263581e50e09643494c0b917d".to_string(),
            "106175f02bfa4344275457c2da1d9b4cc2d3016a4fd4fc73492a894bbaa2b8aa".to_string(),
            "c9d49706741c3453968f696ff6324e21b7078fcf6171546fa8bad7ef32821593".to_string(),
        ];
        for s in h {
            let bytes = hex::decode(s).unwrap();
            root_hashes.push(HashValue::from_slice(&bytes).unwrap());
        }
        let mut tree = Tree::<UnsignedInteger, 2, 3>::new();
        for (idx, (node_hash, expected_root_hash)) in hashes.into_iter().zip(root_hashes.into_iter()).enumerate() {
            tree.insert(format!("test-{}", idx), points[idx].clone(), node_hash);
            assert_eq!(expected_root_hash, tree.root_hash().unwrap());
            println!("test-{} pass", idx);
        }
        let delete_hash = vec![
            "58296e1fbd0b2e93fde939693fa3d0252003bf97c80a46dabc50f0de1c894e33".to_string(),
            "67a4b78b7ff4ec7ad62ce52e1bf3d8936d689733d3bab11d46fed4476ce94196".to_string(),
            "98accee0abe3a5f21925ee48cd7b416b4fa0e4975770910c3b76080f4faa48d0".to_string(),
            "e2b98de357e138652895953ae972d1ac997bc6524b3c17b3e064b8d048054a1a".to_string(),
            "091f7d99a6262d675fb9e2e0d6a9fe5edfdb5bef5d67fa0f10aa2898f06809f1".to_string(),
            "5c6e11d3d89adb9fc6753c15098fcd4b4818569979e057f51b5a3fd8beabd194".to_string(),
            "2529b265927d4abf94dcc7381d2e436b200f2abba89fa04537164133df51ae16".to_string(),
        ];

        for (i, expect_root_hash_str) in delete_hash.into_iter().enumerate() {
            tree.delete(&format!("test-{}",i), &points[i].clone());
            let expected_hash = HashValue::from_slice(&hex::decode(expect_root_hash_str).unwrap()).unwrap();
            assert_eq!(expected_hash, tree.root_hash().unwrap());
            println!("test-del-{} pass", i);
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