use types::hash_value::HashValue;
use crate::node::{ESMTEntry, FromPrimitive, MRTreeDefault, MRTreeFunc, Node, ObjectEntry, ToPrimitive};
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

