use std::collections::{VecDeque, HashMap};
use types::hash_value::HashValue;
use crate::node::{ESMTEntry, FromPrimitive, HilbertSorter, MRTreeDefault, MRTreeFunc, Node, ObjectEntry, ToPrimitive};
use crate::shape::Rect;
use crate::verify::{VerifyObject, VerifyObjectEntry, SiblingObject};

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

    /// 删除时会重新计算每一层的mbr以及hash；是否发生下溢由上一层进行判断
    fn delete_downcast(node: &mut Node<V, D, C>,
                     rect: &Rect<V, D>,
                     reinsert: &mut VecDeque<ESMTEntry<V, D, C>>,
                     height: u32,
    ) -> (Option<ESMTEntry<V, D, C>>, bool) {
        let subtree_idx = node.choose_subtree(rect);
        if height == 0 {
            let to_delete = node.entry.swap_remove(subtree_idx);
            let recalced = node.mbr.on_edge(to_delete.mbr());
            if recalced {
                node.recalculate_mbr();
            }
            node.rehash();
            (Some(to_delete),
             recalced)
        } else {
            let child = node.entry[subtree_idx].get_node_mut();
            let (something_delete, mut recalced) =
                Self::delete_downcast(child, rect, reinsert, height - 1);
            if child.need_downcast() {
                reinsert.extend(child.entry.drain(..));
                let underflow_node = node.entry.swap_remove(subtree_idx);
                recalced = node.mbr.on_edge(underflow_node.mbr());
            }
            if recalced {
                node.recalculate_mbr();
            }
            node.rehash();
            (something_delete, recalced)
        }
    }

    /// 打包entry形成节点
    fn pack_node(mut entries: Vec<ESMTEntry<V, D, C>>,
                 height: u32,
    ) -> Vec<ESMTEntry<V, D, C>> {
        // 如果条目数量不足以打包，那么需要重新插入
        if entries.len() < Node::<V, D, C>::CAPACITY {
            return entries;
        }
        let full_pack_remain = entries.len() % Node::<V, D, C>::CAPACITY;
        let full_pack_cnt = entries.len() / Node::<V, D, C>::CAPACITY;
        let mut slice = vec![Node::<V, D, C>::CAPACITY; full_pack_cnt];
        if full_pack_remain > 0 && full_pack_remain < Node::<V, D, C>::MIN_FANOUT {
            slice[full_pack_cnt - 1] = Node::<V, D, C>::CAPACITY + full_pack_remain - Node::<V, D, C>::MIN_FANOUT;
            slice.push(Node::<V, D, C>::MIN_FANOUT);
        } else if Node::<V, D, C>::MIN_FANOUT <= full_pack_remain {
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
        while objs.len() > cap {
            objs = Self::pack_node(objs, height);
            height += 1;
        }
        let mut root = Node::new_with_entry(height, objs);
        root.recalculate_state_after_sort();
        root
    }

    pub fn range_query(&self, query: &Rect<V, D>, height: u32) -> VerifyObject<V, D> {
        Self::range_query_impl(&self.node, query, height)
    }

    fn range_query_impl(node: &Node<V, D, C>, query: &Rect<V, D>, height: u32) -> VerifyObject<V, D> {
        let mut vo = VerifyObject::new();
        if height == 0 {
            let exist_vec = node.entry.iter()
                .map(|ety| query.contains(ety.mbr()))
                .collect::<Vec<_>>();
            let exist_flag = exist_vec.iter().fold(false, |acc, e| acc || *e);
            if exist_flag {
                vo.push(VerifyObjectEntry::LevelBegin);
                for i in 0..exist_vec.len() {
                    if exist_vec[i] {
                        vo.push(VerifyObjectEntry::Target(node.entry[i].get_object().clone()));
                    } else {
                        vo.push(VerifyObjectEntry::Sibling(SiblingObject::from(node.entry[i].get_object())));
                    }
                }
                vo.push(VerifyObjectEntry::LevlEnd);
            }
        } else {
            let mut exist_vec = vec![];
            let mut temp_vo = VerifyObject::new();
            for ety in node.entry.iter() {
                if query.intersects(ety.mbr()) {
                    let sub_vo = Self::range_query_impl(ety.get_node(), query, height - 1);
                    if sub_vo.is_empty() {
                        exist_vec.push(false);
                    } else {
                        exist_vec.push(true);
                        temp_vo.extend(sub_vo);
                    }
                } else {
                    exist_vec.push(false);
                }
            }
            if !temp_vo.is_empty() {
                vo.push(VerifyObjectEntry::LevelBegin);
                vo.extend(temp_vo);
                for i in 0..exist_vec.len() {
                    if !exist_vec[i] {
                        vo.push(VerifyObjectEntry::Sibling(SiblingObject::from(node.entry[i].get_node())));
                    }
                }
                vo.push(VerifyObjectEntry::LevlEnd);
            }
        }
        vo
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

    pub fn range(&self) -> Option<Rect<V, D>> {
        match &self.root {
            None => { None }
            Some(r) => {
                Some(r.node.mbr().clone())
            }
        }
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
        self.len += 1;
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

    // !TODO: test correctness
    pub fn merge_with_subtree(&mut self, mut another: PartionTree<V, D, C>) {
        if another.root.is_none() {
            return;
        } else if self.root.is_none() {
            let compacted_root =
                EfficientMRTreeNode::build_tree(
                    EfficientMRTreeNode::compact(
                        another.root.take().unwrap().unpack_node()));
            self.height = compacted_root.height;
            self.len = another.len;
            self.root = Some(EfficientMRTreeNode::new(compacted_root));
            return;
        }

        let (mut large_tree, small_tree) = if self.height < another.height {
            (another.root.take().unwrap().unpack_node(), self.root.take().unwrap().unpack_node())
        } else {
            (self.root.take().unwrap().unpack_node(), another.root.take().unwrap().unpack_node())
        };

        let expected_repack_height = (large_tree.height as i32) - (small_tree.height as i32) - 1;
        if expected_repack_height >= 0 { // merge进来的子树的高度比自己低
            let mut reinsert = VecDeque::new();
            let (to_repack,_) =
                EfficientMRTreeNode::delete_downcast(&mut large_tree, small_tree.mbr(), &mut reinsert, expected_repack_height as u32);

            if large_tree.height != 0 && large_tree.entry.len() == 1 {
                large_tree = large_tree.entry.pop().unwrap().unpack_node();
            }

            let to_repack = to_repack.unwrap();
            assert_eq!(to_repack.get_node().height, small_tree.height, "get different height subtrees");
            let to_compact = Node::new_with_entry(
                small_tree.height + 1,
                vec![
                    ESMTEntry::ENode(to_repack.unpack_node()),
                    ESMTEntry::ENode(small_tree),
                ]
            );
            let mut new_subtree = EfficientMRTreeNode::build_tree(EfficientMRTreeNode::compact(to_compact));
            // 根据new_subtree的高度和large_tree的高度要分类讨论
            if new_subtree.height > large_tree.height {
                std::mem::swap(&mut large_tree, &mut new_subtree);
            }
            if new_subtree.height < large_tree.height && new_subtree.suitable_for_subtree() {
                reinsert.push_front(ESMTEntry::ENode(new_subtree));
            } else {
                for ety in new_subtree.entry {
                    reinsert.push_front(ety);
                }
            }

            // if new_subtree.height < large_tree.height {
            //     if new_subtree.suitable_for_subtree() {
            //         reinsert.push_front(ESMTEntry::ENode(new_subtree));
            //     } else {
            //         for ety in new_subtree.entry {
            //             reinsert.push_front(ety);
            //         }
            //     }
            // } else if new_subtree.height == large_tree.height {
            //     for ety in new_subtree.entry {
            //         reinsert.push_front(ety);
            //     }
            // } else {
            //     std::mem::swap(&mut large_tree, &mut new_subtree);
            // }
            // reinsert
            self.height = large_tree.height;
            self.root = Some(EfficientMRTreeNode::new(large_tree));
            while let Some(entry) = reinsert.pop_back() {
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
            // update metadate
            self.len += another.len;
        } else { // 高度相同
            let to_compact = Node::new_with_entry(
                small_tree.height + 1,
                vec![
                    ESMTEntry::ENode(large_tree),
                    ESMTEntry::ENode(small_tree),
                ]
            );
            let new_root = EfficientMRTreeNode::build_tree(EfficientMRTreeNode::compact(to_compact));
            self.height = new_root.height;
            self.root = Some(EfficientMRTreeNode::new(new_root));
            self.len += another.len;
        }
    }

    pub fn clear(&mut self) -> PartionTree<V, D, C> {
        let root = self.root.take();
        let partion = Self {
            root,
            area: self.area.clone(),
            height: self.height,
            len: self.len,
        };
        self.height = 0;
        self.len = 0;
        partion
    }

    pub fn range_query(&self, query: &Rect<V, D>) -> Option<VerifyObject<V, D>> {
        if self.root.is_none() {
            return None;
        }
        let root = self.root.as_ref().unwrap();
        let vo = root.range_query(query, self.height);
        if vo.is_empty() {
            None
        } else {
            Some(vo)
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
}

pub struct PartionManager<V, const D: usize, const C: usize> 
    where
        V: MRTreeDefault,  
{
    // 四叉树的高度，根节点的高度为0
    height: u32,
    areas: Vec<Rect<V, D>>,
    centers: Vec<Rect<V, D>>,
    partions: Vec<PartionTree<V, D, C>>,
    key_2_loc: HashMap<String, (usize, [V; D])>,
}

impl<V, const D: usize, const C: usize> PartionManager<V, D, C> 
    where
        V: MRTreeDefault + MRTreeFunc + ToPrimitive + FromPrimitive,
{
    const BASIC_THRESHOLD: usize = 1024;
    const DEGREE: usize = 2usize.pow(D as u32);
    /// 二维数据下的区域划分
    /// y
    /// | 2   4
    /// | 1   3
    /// |_______ x
    pub fn new(area: Rect<V, D>, height: u32) -> Self {
        // assert_eq!(D, 2, "only support 2-d space now");
        let partion_cnt = ((Self::DEGREE.pow(height + 1) - 1) / (Self::DEGREE - 1)) as usize;
        let mut areas = Vec::with_capacity(partion_cnt);
        let mut centers = Vec::with_capacity(partion_cnt);
        let mut partions = Vec::with_capacity(partion_cnt);

        // 将2^D叉树根节点（最上层的partion）插入到数组中
        centers.push(Self::center(&area));
        areas.push(area.clone());
        partions.push(PartionTree::new_with_area(area));

        // 处理其他的层
        for idx in 1..partion_cnt {
            let parent = (idx - 1) >> D;
            let par_center = centers[parent]._min.clone();
            let bits = Self::num_bits((idx - 1) % Self::DEGREE);
            let mut min = [V::default(); D];
            let mut max = [V::default(); D];
            for i in 0..D {
                if bits[i] == 0 {
                    min[i] = areas[parent]._min[i];
                    max[i] = par_center[i];
                } else {
                    min[i] = par_center[i];
                    max[i] = areas[parent]._max[i];
                }
            }
            let cur_area = Rect::new(min, max);
            let cur_center = Self::center(&cur_area);
            let partion = PartionTree::new_with_area(cur_area.clone());
            centers.push(cur_center);
            areas.push(cur_area);
            partions.push(partion);
        }

        Self {
            height,
            areas,
            centers,
            partions,
            key_2_loc: HashMap::new(),
        }
    }

    fn num_bits(mut num: usize) -> Vec<usize> {
        let mut bits = Vec::with_capacity(D);
        for _ in 0..D {
            bits.push(num % 2);
            num >>= 1;
        }
        bits.reverse();
        bits
    }

    fn center(rect: &Rect<V, D>) -> Rect<V, D> {
        let mut c = [V::default(); D];
        for i in 0..D {
            c[i] = (rect._max[i] + rect._min[i]) / (V::from_i32(2));
        }
        Rect::new_point(c)
    }

    pub fn print_level_info(&self) {
        for (idx, rect) in self.areas.iter().enumerate() {
            println!("{}: {:?}", idx, rect)
        }
    }

    pub fn point_index(&self, point: &[V; D]) -> usize {
        let mut parent = 0usize;
        for _ in 0..self.height {
            let mut level_idx = 0usize;
            for i in 0..D {
                level_idx = (level_idx << 1) | ((point[i] > self.centers[parent]._max[i]) as usize);
            }
            parent = ((parent << D) | level_idx) + 1;
        }
        parent
    }

    pub fn insert(&mut self, key: String, loc:[V; D], hash: HashValue) {
        let partion_to_insert = self.point_index(&loc);
        // 将新插入的数据对象添加到表中
        self.key_2_loc.insert(key.clone(), (partion_to_insert, loc.clone()));

        let cur_partion = partion_to_insert;
        self.insert_impl(key, loc, hash, cur_partion);
    }

    fn insert_impl(&mut self, key: String, loc:[V; D], hash: HashValue, index: usize) {
        // 先处理需要merge的情况
        self.merge(index, 1);
        self.partions[index].insert(key, loc, hash);
    }

    pub fn delete(&mut self, key: &String) -> Option<ObjectEntry<V, D>> {
        if let Some((idx, oloc)) = self.key_2_loc.remove(key) {
            self.partions[idx].delete(key, &oloc)
        } else {
            None
        }
    }

    pub fn update(&mut self, key: &String, nloc: [V; D]) {
        let nidx = self.point_index(&nloc);
        let some_data = self.key_2_loc.get(key).map(|(idx, loc)| (*idx, loc.clone()));
        if let Some((oidx, oloc)) = some_data {
            // 更新在同一分区中
            if oidx == nidx {
                self.partions[nidx].update(key, &oloc, nloc);
            } else {
                let obj = self.partions[oidx].delete(key, &oloc).unwrap();
                // self.insert_impl(key.clone(), nloc.clone(), obj.hash(), nidx);
                self.partions[nidx].insert(key.clone(), nloc.clone(), obj.hash());
            }
            // 更新表中的信息
            let (idx, loc) = self.key_2_loc.get_mut(key).unwrap();
            *idx = nidx;
            *loc = nloc;
        }
    }

    fn merge(&mut self, cur_partion: usize, threshold_mul: usize) {
        // 该partion不需要merge || 该partion是根partion
        if cur_partion == 0 || self.partions[cur_partion].len() < Self::BASIC_THRESHOLD * threshold_mul {
            return;
        }
        let parent = (cur_partion - 1) >> D;
        // 先merge上层的partion
        self.merge(parent, threshold_mul << (D + D));
        // 把自己merge上去
        let need_to_merge = self.partions[cur_partion].clear();
        self.partions[parent].merge_with_subtree(need_to_merge);
    }

    pub fn range_query(&self, query: &Rect<V, D>) -> Vec<VerifyObject<V, D>> {
        let mut res = vec![];
        for p in self.partions.iter() {
            if p.area.intersects(query) {
                if let Some(vo) = p.range_query(query) {
                    res.push(vo);
                }
            }
        }
        res
    }
}
#[cfg(test)]
mod test {
    use types::hash_value::HashValue;
    use types::test_utils::{num_hash};
    use crate::esmtree::PartionTree;
    use crate::esmtree::PartionManager;
    use crate::shape::Rect;

    #[derive(Debug)]
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
            [2usize, 7], [2, 5], [2, 3], [6, 1], [8, 3], [0, 0], [4, 5], [4, 3], [7, 5], [0, 3]
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
            Operator::Insert(9),
            Operator::Update(5),
            Operator::Update(4),
            Operator::Update(1),
            Operator::Update(0),
            Operator::Merge,
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
            "9fbca63c043666257f91d43f329d3234f14582147d5b95250b2f095bed549712".to_string(), // u 5
            "9fbca63c043666257f91d43f329d3234f14582147d5b95250b2f095bed549712".to_string(), // u 4
            "8865cad780eedf83b2db2f6425cb5f6592729dee25da1dce2c8820823e32db7a".to_string(), // u 1
            "200394f45e6ff302874ec2d8ee59c2272e701d6957172eba5b08d999ecfb6d08".to_string(), // u 0
            "96112008a00abf1ef6c7ea6f0409eb477ff251dc09bb2ade7409aa85080690dc".to_string(), // m
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
                    tree.update(&format!("testkey-{}",i),&points[i], new_points[i].clone());
                }
                Operator::Merge => {
                    tree.merge_empty();
                }
            }
            assert_eq!(tree.root_hash().unwrap(), hash);
            println!("{:?} passed", op);
        }
    }

    #[test]
    fn test_merge() {

    }

    #[test]
    fn test_level_info() {
        let pm: PartionManager<f32, 2, 3> = PartionManager::new(Rect::new([1.0f32, 3.0f32], [14.0f32, 8.0f32]), 1);
        pm.print_level_info();
        assert_eq!(pm.point_index(&[4.0f32, 3.7]), 1);
        assert_eq!(pm.point_index(&[3.7f32, 6.9]), 2);
    }
}