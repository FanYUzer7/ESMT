use std::{slice::Iter, vec::IntoIter, collections::BTreeSet};

use types::hash_value::{HashValue, ESMTHasher};

use crate::{node::{ObjectEntry, MRTreeDefault, Node, MRTreeFunc}, shape::Rect};

#[derive(Clone)]
pub enum VerifyObjectEntry<V, const D: usize> 
    where
        V: MRTreeDefault,
{
    LevelBegin,
    LevlEnd,
    Target(ObjectEntry<V, D>),
    Sibling(SiblingObject<V, D>),
}

impl<V, const D: usize> VerifyObjectEntry<V, D> 
    where
        V: MRTreeDefault,
{
    pub fn hash(&self) -> Option<HashValue> {
        match self {
            VerifyObjectEntry::LevelBegin | VerifyObjectEntry::LevlEnd => None,
            VerifyObjectEntry::Target(t) => Some(t.hash()),
            VerifyObjectEntry::Sibling(s) => Some(s.hash()),
        }
    }
}

#[derive(Clone)]
pub struct SiblingObject<V, const D: usize>
    where
        V: MRTreeDefault,
{
    range: Rect<V, D>,
    hash: HashValue,
}

impl<V, const D: usize> SiblingObject<V, D> 
    where
        V: MRTreeDefault,
{
    #[inline]
    pub fn hash_ref(&self) -> &[u8] {
        self.hash.as_ref()
    }

    #[inline]
    pub fn hash(&self) -> HashValue {
        self.hash
    }

    #[inline]
    pub fn range(&self) -> &Rect<V, D> {
        &self.range
    }
}

impl<V, const D: usize, const C: usize> From<&Node<V, D, C>> for SiblingObject<V, D> 
    where
        V: MRTreeDefault,
{
    fn from(node: &Node<V, D, C>) -> Self {
        Self {
            range: node.mbr.clone(),
            hash: node.hash.clone(),
        }
    }
}

impl<V, const D: usize> From<&ObjectEntry<V, D>> for SiblingObject<V, D> 
    where
        V: MRTreeDefault,
{
    fn from(obj: &ObjectEntry<V, D>) -> Self {
        Self {
            range: obj.loc().clone(),
            hash: obj.hash()
        }
    }
}

pub struct VerifyObject<V, const D: usize> 
    where
        V: MRTreeDefault,
{
    verify_path: Vec<VerifyObjectEntry<V, D>>,
}

impl<V, const D: usize> VerifyObject<V, D> 
    where
        V: MRTreeDefault,
{
    pub fn new() -> Self {
        Self {
            verify_path: vec![],
        }
    }

    #[inline]
    pub fn push(&mut self, entry: VerifyObjectEntry<V, D>) {
        self.verify_path.push(entry);
    }

    #[inline]
    pub fn extend(&mut self, ano: VerifyObject<V, D>) {
        self.verify_path.extend(ano.verify_path);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.verify_path.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, VerifyObjectEntry<V, D>> {
        self.verify_path.iter()
    }

    #[inline]
    pub fn into_iter(self) -> IntoIter<VerifyObjectEntry<V, D>> {
        self.verify_path.into_iter()
    }

    pub fn display(&self) {
        for ety in self.verify_path.iter() {
            match ety {
                VerifyObjectEntry::LevelBegin => {
                    print!("[");
                },
                VerifyObjectEntry::LevlEnd => {
                    print!("], ");
                },
                VerifyObjectEntry::Target(t) => {
                    print!("{}, ", t.key());
                },
                VerifyObjectEntry::Sibling(s) => {
                    print!("<{:?}>, ", s.range())
                },
            }
        }
        println!();
    }
}

impl<V, const D: usize> VerifyObject<V, D> 
    where
        V: MRTreeDefault + MRTreeFunc,
{
    pub fn verify(&self, query: &Rect<V, D>, root_hash: HashValue) -> Result<(), VerifyError> {
        let mut res_mbr = None;
        let mut parse_stack = vec![];
        let mut hash_stack: Vec<HashValue> = vec![];
        for ety in self.verify_path.iter() {
            match ety {
                VerifyObjectEntry::LevelBegin => {
                    parse_stack.push(ety.clone());
                },
                VerifyObjectEntry::LevlEnd => {
                    let mut hash_set = BTreeSet::new();
                    loop {
                        let op = parse_stack.pop().unwrap();
                        if let VerifyObjectEntry::LevelBegin = op {
                            break;
                        }
                        hash_set.insert(hash_stack.pop().unwrap());
                    }
                    let hasher = hash_set
                        .into_iter()
                        .fold(ESMTHasher::default(), |hasher, entry| {
                            hasher.update(entry.as_ref())
                        });
                    hash_stack.push(hasher.finish());
                },
                VerifyObjectEntry::Target(target) => {
                    parse_stack.push(ety.clone());
                    hash_stack.push(target.hash());
                    if res_mbr.is_none() {
                        res_mbr = Some(target.loc().clone());
                    } else {
                        let mut r = res_mbr.as_mut().unwrap();
                        r.expand(target.loc());
                    }
                },
                VerifyObjectEntry::Sibling(sibling) => {
                    if sibling.range().intersects(query) {
                        return Err(VerifyError::CompletenessError);
                    }
                    parse_stack.push(ety.clone());
                    hash_stack.push(sibling.hash());
                },
            }
        }
        if res_mbr.is_none() {
            return Ok(());
        } else {
            let res_mbr = res_mbr.unwrap();
            if !query.contains(&res_mbr) {
                return Err(VerifyError::ResultError);
            }
        }
        let verify_root_hash = hash_stack.pop().unwrap();
        if verify_root_hash == root_hash {
            Ok(())
        } else {
            Err(VerifyError::SoundnessError)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum VerifyError {
    SoundnessError,
    CompletenessError,
    ResultError,
}