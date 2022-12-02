use std::collections::VecDeque;
use types::hash_value::HashValue;
use crate::node::{ESMTEntry, FromPrimitive, HilbertSorter, MRTreeDefault, MRTreeFunc, Node, ObjectEntry, ToPrimitive};
use crate::shape::Rect;

struct EfficientMRTreeNode<V, const D: usize, const C: usize>
    where
        V: MRTreeDefault,
{
    node: Node<V, D, C>,
}

impl<V, const D: usize, const C: usize> EfficientMRTreeNode<V, D, C>
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    pub fn new(node: Node<V, D, C>) -> Self {
        Self {
            node,
        }
    }

    pub fn new_with_height(height: u32) -> Self {
        Self {
            node: Node::new_with_height(height),
        }
    }

    #[inline]
    pub fn hash(&self) -> HashValue {
        self.node.hash()
    }

    #[inline]
    pub fn unpack_node(self) -> Node<V, D, C> {
        self.node
    }

    /// 插入，重新计算当前层的mbr以及下一层的hash
    fn insert_by_esmt(node: &mut Node<V, D, C>, obj: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
        if height == 0 {
            if let Some(i) = node.first_stale() {
                node.entry[i] = obj;
            } else {
                node.entry.push(obj);
            }
            node.recalculate_mbr();
        } else {
            let subtree_idx = node.choose_subtree(&loc);
            let node_mut = node.entry[subtree_idx].get_node_mut();
            // node_mut.insert_by_mrt(obj, loc, height - 1);
            Self::insert_by_esmt(node_mut, obj, loc, height - 1);
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
            node.recalculate_mbr();
        }
    }

    pub fn insert(&mut self, obj: ESMTEntry<V, D, C>, loc: &Rect<V, D>, height: u32) {
        Self::insert_by_esmt(&mut self.node, obj, loc, height);
    }

    /// 删除时设置stale, 不需要重新计算哈希和mbr
    fn delete_by_esmt(node: &mut Node<V, D, C>,
                      rect: &Rect<V, D>,
                      key: &str,
                      height: u32,
    ) -> Option<ESMTEntry<V, D, C>> {
        if height == 0 {
            for i in 0..node.entry.len() {
                if node.entry[i].get_object().match_key(key) {
                    let to_delete = node.entry[i].get_object().clone();
                    node.entry[i].get_object_mut().delete();
                    return Some(ESMTEntry::Object(to_delete));
                }
            }
        } else {
            for i in 0..node.entry.len() {
                if !rect.intersects(node.entry[i].mbr()) {
                    continue;
                }
                let child = node.entry[i].get_node_mut();
                // let (removed, mut recalced) = node.delete_by_mrt(rect, key, reinsert, height - 1);
                let removed =
                    Self::delete_by_esmt(child, rect, key, height - 1);
                if removed.is_none() {
                    continue;
                }
                return removed;
            }
        }
        None
    }

    pub fn delete(&mut self,
                  rect: &Rect<V, D>,
                  key: &str,
                  height: u32,
    ) -> Option<ESMTEntry<V, D, C>> {
        //Self::delete_by_esmt(&mut self.node, rect, key, height)
        let func =
            |node: &mut Node<V, D, C>, key: &str| -> Option<ESMTEntry<V, D, C>> {
                for i in 0..node.entry.len() {
                    if node.entry[i].get_object().match_key(key) {
                        let to_delete = node.entry[i].get_object().clone();
                        node.entry[i].get_object_mut().delete();
                        return Some(ESMTEntry::Object(to_delete));
                    }
                }
                None
            };
        Self::search_by_esmt(&mut self.node, rect, key, height, &func)
    }

    /// 如果调用了insert方法，返回true
    pub fn update(&mut self,
                  oloc: &Rect<V, D>,
                  nloc: Rect<V, D>,
                  key: &str,
                  height: u32,
    ) -> bool {
        let func =
            |node: &mut Node<V, D, C>, key: &str| -> Option<ESMTEntry<V, D, C>> {
                for i in 0..node.entry.len() {
                    if node.entry[i].get_object().match_key(key) {
                        // 如果更新的位置还在原来的mbr中，则只调整空间对象的位置
                        if node.mbr.contains(&nloc) {
                            node.entry[i].get_object_mut().update_loc(nloc.clone());
                        } else { // 删除原来的节点
                            node.entry[i].get_object_mut().delete();
                        }
                        let to_return = node.entry[i].get_object().clone();
                        return Some(ESMTEntry::Object(to_return));
                    }
                }
                None
            };
        let mut updated_obj = Self::search_by_esmt(&mut self.node, oloc, key, height, &func).unwrap();
        if updated_obj.get_object().is_stale() {
            // 更新位置和stale重新插入
            updated_obj.get_object_mut().update_loc(nloc.clone());
            updated_obj.get_object_mut().refresh();
            self.insert(updated_obj, &nloc, height);
            return true;
        }
        false
    }

    fn search_by_esmt(node: &mut Node<V, D, C>,
                      rect: &Rect<V, D>,
                      key: &str,
                      height: u32,
                      func: &dyn Fn(&mut Node<V, D, C>, &str) -> Option<ESMTEntry<V, D, C>>,
    ) -> Option<ESMTEntry<V, D, C>> {
        if height == 0 {
            return func(node, key);
        } else {
            for i in 0..node.entry.len() {
                if !rect.intersects(node.entry[i].mbr()) {
                    continue;
                }
                let child = node.entry[i].get_node_mut();
                let found =
                    Self::search_by_esmt(child, rect, key, height - 1, func);
                if found.is_none() {
                    continue;
                }
                return found;
            }
        }
        None
    }

    /**
    递归地进行压缩与合并

    递归的步骤：
    1. 如果节点（Node）的高度为0（height=0），则将自身的条目进行压缩并返回
    1. 压缩（[`compact`](fn@compact)）节点的每一个条目（Entry）
    1. 将节点的所有条目进行合并（[`merge`](fn@merge)），重新生成一定数量的条目
        - 如果无法生成一个条目Ei，则将组成条目Ei的条目Ej加入到re-insert队列中
    3. 递归结束，执行re-insert
     */
    // fn compact_and_merge(node: &mut Node<V, D, C>, reinsert: &mut Vec<ESMTEntry<V, D, C>>, height: u32) {
    //     let mut entries = vec![];
    //     node.recalculate_mbr();
    //     let hilbert_sorter = HilbertSorter::<V, D, C>::new(node.mbr());
    //     // 准备处理object
    //     if height == 1 {
    //         // 压缩收集非stale的object
    //         for i in 0..node.entry.len() {
    //             let objs = node.entry[i].get_node_mut().entry.drain(..);
    //             entries.extend(objs.filter(|e| {
    //                 !e.get_object().is_stale()
    //             }));
    //         }
    //         // 排序 & 打包节点
    //         let sorted_entries = hilbert_sorter.sort(entries);
    //         node.entry = PartionTree::pack_node(sorted_entries, reinsert, 0);
    //         // 重新计算节点的mbr
    //         for ety in &mut node.entry {
    //             ety.get_node_mut().recalculate_mbr();
    //         }
    //         // 重新计算自己的mbr
    //         node.recalculate_mbr();
    //     } else {
    //         // 排序，打包
    //         for i in 0..node.entry.len() {
    //             entries.extend(node.entry[i].get_node_mut().entry.drain(..));
    //         }
    //         let sorted_entries = hilbert_sorter.sort(entries);
    //         node.entry = PartionTree::pack_node(sorted_entries, reinsert, height - 1);
    //         for i in 0..node.entry.len() {
    //             let mut child = node.entry[i].get_node_mut();
    //             Self::compact_and_merge(child, reinsert, height - 1);
    //             if child.need_downcast() {
    //                 reinsert.extend(child.entry.drain(..));
    //             }
    //         }
    //     }
    // }

    /// 打包entry形成节点
    fn pack_node(mut entries: Vec<ESMTEntry<V, D, C>>,
                 reinsert: &mut Vec<ESMTEntry<V, D, C>>,
                 height: u32,
    ) -> Vec<ESMTEntry<V, D, C>> {
        // 如果条目数量不足以打包，那么需要重新插入
        if entries.len() < Node::<V, D, C>::MIN_FANOUT {
            reinsert.extend(entries);
            return vec![];
        }
        let full_pack_remain = entries.len() % Node::<V, D, C>::CAPACITY;
        let full_pack_cnt = entries.len() / Node::<V, D, C>::CAPACITY;
        let mut slice = vec![Node::<V, D, C>::CAPACITY; full_pack_cnt];
        if full_pack_remain > 0 && full_pack_remain < Node::<V, D, C>::MIN_FANOUT {
            slice[full_pack_cnt - 1] = Node::<V, D, C>::CAPACITY + full_pack_remain - Node::<V, D, C>::MIN_FANOUT;
            slice.push(Node::<V, D, C>::MIN_FANOUT);
        } else if Node::<V, D, C>::MIN_FANOUT < full_pack_remain {
            slice.push(full_pack_remain);
        }
        let mut nodes = Vec::with_capacity(slice.len());
        for slice_cnt in slice.into_iter() {
            let entry = entries.drain(..slice_cnt).collect::<Vec<_>>();
            let mut node = Node::new_with_entry(height, entry);
            node.recalculate_state_after_sort();
            nodes.push(ESMTEntry::ENode(node));
        }
        nodes
    }

    fn compact(root: Node<V,D,C>) -> Vec<ESMTEntry<V, D, C>> {
        let sorter = HilbertSorter::new(root.mbr());
        let mut queue = VecDeque::new();
        let mut objs = vec![];
        queue.push_back(root);
        while !queue.is_empty() {
            let mut node =queue.pop_front().unwrap();
            if node.height == 0 {
                objs.extend(node.entry
                    .into_iter()
                    .filter(|e| {
                        !e.get_object().is_stale()
                    }));
            } else {
                queue.extend(node.entry
                    .into_iter()
                    .map(|e| {
                        e.unpack_node()
                    }));
            }
        }
        sorter.sort(objs)
    }

    fn build_tree(mut objs: Vec<ESMTEntry<V, D, C>>) -> Node<V, D, C> {
        let cap = Node::<V, D, C>::CAPACITY;
        let min_fanout = Node::<V, D, C>::MIN_FANOUT;
        let mut height = 0u32;
        let mut roo_child = vec![];
        while objs.len() > cap {
            objs = Self::pack_node(objs, &mut roo_child, height);
            height += 1;
        }
        if objs.len() == 0 {
            objs = roo_child;
        }
        let mut root = Node::new_with_entry(height, objs);
        root.recalculate_state_after_sort();
        root
    }
}

pub struct PartionTree<V, const D: usize, const C: usize> 
    where
        V: MRTreeDefault,
{
    root: Option<EfficientMRTreeNode<V, D, C>>,
    area: Rect<V, D>,
    height: u32,
    len: usize,
}

impl<V, const D: usize, const C: usize> PartionTree<V, D, C> 
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    pub fn new() -> Self {
        Self {
            root: None,
            area: Rect::default(),
            height: 0,
            len: 0
        }
    }

    pub fn new_with_area(area: Rect<V, D>) -> Self {
        Self {
            root: None,
            area,
            height: 0,
            len: 0
        }
    }

    #[inline]
    pub fn area(&self) -> Rect<V, D> {
        self.area.clone()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn root_hash(&self) -> Option<HashValue> {
        match &self.root {
            None => { None }
            Some(r) => { Some(r.hash()) }
        }
    }

    pub fn insert(&mut self, key: String, loc:[V; D], hash: HashValue) {
        if self.root.is_none() {
            self.root = Some(EfficientMRTreeNode::new_with_height(0));
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
            self.root = Some(EfficientMRTreeNode::new(new_root));
        } else {
            root.node.rehash()
        }
        self.len += 1;
    }

    pub fn delete(&mut self, key: &str, rect: &[V;D]) -> Option<ObjectEntry<V, D>> {
        if let Some(root) = &mut self.root {
            let loc = Rect::new_point(rect.clone());
            let entry = root.delete(&loc, key, self.height);
            if entry.is_none() {
                return None;
            }
            self.len -= 1;
            entry.map(|e| e.unpack_object())
        } else {
            None
        }
    }

    pub fn update(&mut self, key: &str, oloc: &[V; D], nloc: [V; D]) {
        if let Some(root) = &mut self.root {
            let orect = Rect::new_point(oloc.clone());
            let nrect = Rect::new_point(nloc);
            let call_insert = root.update(&orect, nrect, key, self.height);
            if call_insert {
                let need_split = root.node.is_overflow();
                if need_split {
                    self.height += 1;
                    let mut new_root = Node::new_with_height(self.height);
                    let mut origin = self.root.take().unwrap().unpack_node();
                    let another = origin.split_by_hilbert_sort();
                    new_root.entry.push(ESMTEntry::ENode(origin));
                    new_root.entry.push(ESMTEntry::ENode(another));
                    new_root.recalculate_state_after_sort();
                    self.root = Some(EfficientMRTreeNode::new(new_root));
                } else {
                    root.node.rehash()
                }
            }
        }
    }

    pub fn merge_empty(&mut self) {
        let root = self.root.take().unwrap().unpack_node();
        let new_root = EfficientMRTreeNode::build_tree(EfficientMRTreeNode::compact(root));
        self.root = Some(EfficientMRTreeNode::new(new_root));
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
}

#[cfg(test)]
mod test {
    use types::hash_value::HashValue;
    use types::test_utils::{generate_points, num_hash};
    use crate::esmtree::PartionTree;

    enum Operator {
        Insert(usize),
        Delete(usize),
        Update(usize),
        Merge,
    }
    #[test]
    fn test_root_hash() {
        // let points = generate_points([0usize, 0], [8usize, 8], 10);
        let points = vec![
            [1usize, 6],
            [0, 5],
            [3, 2],
            [4, 5],
            [8, 5],
            [2, 8],
            [2, 3],
            [6, 7],
            [8, 0],
            [1, 1]
        ];
        let new_points = vec![
            [4usize, 1], [2, 5], [2, 3], [6, 1], [8, 3], [0, 0], [4, 5], [4, 3], [7, 5], [0, 3]
        ];
        println!("{:?}", points);
        let ops = vec![
            Operator::Insert(0),
            Operator::Insert(1),
            Operator::Insert(2),
            Operator::Insert(3),
            Operator::Insert(4),
            Operator::Insert(5),
            Operator::Insert(6),
            Operator::Insert(7),
            Operator::Insert(8),
        ];
        let hash_str = vec![
            "5b4d6fe0dd8fd7bc6de264d7c3db3ed25ae1306dbdf20843e91acaaf8b6728f5".to_string(), // i 0
            "8186e82dd80cce7b15828191c85bf1f128bd6e1168f670361a65c9b14cd7b06d".to_string(), // i 1
            "8d015b832a692a90b69409c6bdabcd122c05f198306dd481bf380c0a1d817e66".to_string(), // i 2
            "0bd13fbae340f13bc8580b2d777c5393652a2e4fce220bb618b156b8cf97b90f".to_string(), // i 3
            "7b5b68e400187a7c07f1af2043315dee22517f0919cfd1df1b21a319b0bb04e4".to_string(), // i 4
            "902d1aaa9fdedf73a5cb2e289a941d7baed0db1263581e50e09643494c0b917d".to_string(), // i 5
            "0af2fb57bcc0d167b87d4ac94cb22ae1e977d788225f6dc6cff8d2337a4fa572".to_string(), // i 6
            "6fb829122401c6d92d7d860aa6f6329e2d8202d63259c27a6cd819f753ebad7d".to_string(), // i 7
            "48e1e65014f7ea6e53398dc8718e7d10c98cc1f93e87f941d64336a548b9f65a".to_string(), // i 8
            "2b3e36e150217da8d4fa8466dbbcbc8b4c2fc9822120d2d992639fada09dcc43".to_string(), // i 9
        ];
        let root_hashes = hash_str.into_iter()
            .map(|s| HashValue::from_slice(&hex::decode(s).unwrap()).unwrap())
            .collect::<Vec<_>>();
        let mut tree = PartionTree::<usize, 2, 3>::new();
        for (idx,(op,hash)) in ops.into_iter().zip(root_hashes.into_iter()).enumerate() {
            match op {
                Operator::Insert(i) => {
                    tree.insert(format!("testkey-{}",i), points[i].clone(), num_hash(i as i32));
                }
                Operator::Delete(i) => {
                    tree.delete(&format!("testkey-{}",i), &points[i]);
                }
                Operator::Update(i) => {
                    unimplemented!()
                }
                Operator::Merge => {
                    tree.merge_empty();
                }
            }
            assert_eq!(tree.root_hash().unwrap(), hash);
            println!("hash test-{} passed", idx);
        }
    }
}