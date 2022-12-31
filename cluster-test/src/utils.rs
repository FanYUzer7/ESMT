use std::collections::{HashMap};
use MerkleRTree::{mrtree::MerkleRTree as MRTree, shape::Rect, esmtree::PartionManager};
use rand::{thread_rng, seq::SliceRandom, distributions::Uniform, prelude::Distribution};
use types::hash_value::{HashValue, ESMTHasher};

pub enum TreeOpt {
    Insert(String, [f64; 2], HashValue),
    Update(String, [f64; 2]),
    Delete(String),
    Query(Rect<f64, 2>),
}

pub struct MRTreeTestManager {
    pub data: Vec<TreeOpt>,
    pub tree: MRTree<f64, 2, 4>,
    pub keymap: HashMap<String, [f64; 2]>,
}

pub struct MRTreeBuilder {
    base: usize,
    q_size: usize,
    data: Vec<[f64; 2]>,
}

impl MRTreeBuilder {
    pub fn new() -> Self {
        Self {
            base: 1000,
            q_size: 100,
            data: vec![],
        }
    }

    #[inline]
    pub fn base_size(mut self, size: usize) -> Self {
        self.base = size;
        self
    }

    #[inline]
    pub fn opt_size(mut self, size: usize) -> Self {
        self.q_size = size;
        self
    }

    pub fn set_testset(mut self, db: &Vec<[f64; 2]>) -> Self {
        let mut rng = thread_rng();
        self.data = db.choose_multiple(&mut rng, self.base + self.q_size)
            .map(|p| p.clone())
            .collect();
        self
    }

    pub fn build_insert_test(self) -> MRTreeTestManager {
        let tree = MRTree::new();
        let data = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx,l)| {
                let k = format!("test/mbr/{}", idx);
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                TreeOpt::Insert(k, l, hash) 
            })
            .collect();
        MRTreeTestManager {
            data,
            tree,
            keymap: HashMap::new()
        }
    }

    pub fn build_update_test(self, percent: f64) -> MRTreeTestManager {
        assert!(percent >= 0.0 && percent <= 1.0, "require percent in [0,1], input: {}", percent);
        let insert_cnt = (self.q_size as f64 * percent).floor() as usize;
        let insert_vec = self.data[self.base..(self.base + insert_cnt)].to_vec();
        let update_vec = self.data[(self.base + insert_cnt)..].to_vec();
        // init
        let keymap = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, p)| {
                let key = format!("test/mbr/{}", idx);
                (key, p)
            })
            .collect::<HashMap<_, _>>();
        let mut tree = MRTree::new();
        let iter = keymap.iter()
            .take(self.base)
            .map(|(k,l)| {
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                (k.clone(), l.clone(), hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        // 构造数据
        let mut data = insert_vec.into_iter()
            .enumerate()
            .map(|(idx, loc)| {
                let key = format!("test/mbr/{}", idx + self.base);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                TreeOpt::Insert(key, loc, hash)
            })
            .collect::<Vec<_>>();
        let mut rng = thread_rng();
        let update_cand = (0..self.base).collect::<Vec<_>>()
            .choose_multiple(&mut rng, update_vec.len())
            .map(|key| *key)
            .collect::<Vec<_>>();
        data.extend(update_cand
            .into_iter()
            .zip(update_vec)
            .map(|(upd_idx, nloc)| {
                let key = format!("test/mbr/{}", upd_idx);
                TreeOpt::Update(key, nloc)
            })
        );
        // 打乱顺序
        data.shuffle(&mut rng);

        MRTreeTestManager {
            data,
            tree,
            keymap,
        }
    }

    pub fn build_delete_test(self) -> MRTreeTestManager {
        let mut rng = thread_rng();
        let del_cand = (0..self.base).collect::<Vec<_>>()
            .choose_multiple(&mut rng, self.q_size)
            .map(|key| *key)
            .collect::<Vec<_>>();
        // init
        let keymap = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, p)| {
                let key = format!("test/mbr/{}", idx);
                (key, p)
            })
            .collect::<HashMap<_, _>>();
        let mut tree = MRTree::new();
        let iter = keymap.iter()
            .take(self.base)
            .map(|(k,l)| {
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                (k.clone(), l.clone(), hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        let data = del_cand.into_iter()
            .map(|idx| {
                let key = format!("test/mbr/{}", idx);
                TreeOpt::Delete(key)
            })
            .collect();
        
        MRTreeTestManager {
            data,
            tree,
            keymap,
        }
    }

    pub fn build_query_test(self) -> MRTreeTestManager {
        // init
        let keymap = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, p)| {
                let key = format!("test/mbr/{}", idx);
                (key, p)
            })
            .collect::<HashMap<_, _>>();
        let mut tree = MRTree::new();
        let iter = keymap.iter()
            .take(self.base)
            .map(|(k,l)| {
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                (k.clone(), l.clone(), hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        // 构造查询数据
        let mut rng = thread_rng();
        let area = tree.area().unwrap();
        let x_grid = (area._max[0] - area._min[0]) / 20.0;
        let y_grid = (area._max[1] - area._min[1]) / 20.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Query(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        MRTreeTestManager {
            data,
            tree,
            keymap,
        }
    }
}

impl MRTreeTestManager {
    pub fn exec(self) {
        let mut keymap = self.keymap;
        let mut tree = self.tree;
        for opt in self.data {
            match opt {
                TreeOpt::Insert(key, loc, hash) => {
                    tree.insert(key.clone(), loc.clone(), hash);
                    keymap.insert(key, loc);
                },
                TreeOpt::Update(key, nloc) => {
                    let oloc = keymap.get(&key).unwrap();
                    tree.update_loc(&key, oloc, nloc.clone());
                    let _ = keymap.insert(key, nloc);
                },
                TreeOpt::Delete(key) => {
                    let oloc = keymap.get(&key).unwrap();
                    let _ = tree.delete(&key, oloc);
                    let _ = keymap.remove(&key);
                },
                TreeOpt::Query(query) => {
                    let _ = tree.range_query(&query);
                },
            }
        }
    }
}

pub struct ESMTreeTestManager {
    pub data: Vec<TreeOpt>,
    pub tree: PartionManager<f64, 2, 4>,
}

pub struct ESMTreeBuilder {
    base: usize,
    q_size: usize,
    p_height: u32,
    range: Rect<f64, 2>,
    data: Vec<[f64; 2]>,
}

impl ESMTreeBuilder {
    pub fn new() -> Self {
        Self {
            base: 1000,
            q_size: 100,
            p_height: 3,
            range: Rect { _max: [0.0, 0.0], _min: [100.0, 100.0] },
            data: vec![],
        }
    }

    #[inline]
    pub fn base_size(mut self, size: usize) -> Self {
        self.base = size;
        self
    }

    #[inline]
    pub fn opt_size(mut self, size: usize) -> Self {
        self.q_size = size;
        self
    }

    #[inline]
    pub fn partion_height(mut self, height: u32) -> Self {
        self.p_height = height;
        self
    }

    #[inline]
    pub fn range(mut self, r: Rect<f64, 2>) -> Self {
        self.range = r;
        self
    }

    pub fn set_testset(mut self, db: &Vec<[f64; 2]>) -> Self {
        let mut rng = thread_rng();
        self.data = db.choose_multiple(&mut rng, self.base + self.q_size)
            .map(|p| p.clone())
            .collect();
        self
    }

    pub fn build_insert_test(self) -> ESMTreeTestManager {
        let tree = PartionManager::new(self.range, self.p_height);
        let data = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx,l)| {
                let k = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                TreeOpt::Insert(k, l, hash) 
            })
            .collect();
        
        ESMTreeTestManager {
            data,
            tree,
        }
    }

    pub fn build_update_test(self, percent: f64) -> ESMTreeTestManager {
        assert!(percent >= 0.0 && percent <= 1.0, "require percent in [0,1], input: {}", percent);
        let insert_cnt = (self.q_size as f64 * percent).floor() as usize;
        let insert_vec = self.data[self.base..(self.base + insert_cnt)].to_vec();
        let update_vec = self.data[(self.base + insert_cnt)..].to_vec();
        // init
        let mut tree = PartionManager::new(self.range, self.p_height);
        let iter = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, point)| {
                let key = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                (key, point, hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        // 构造数据
        let mut data = insert_vec.into_iter()
            .enumerate()
            .map(|(idx, loc)| {
                let key = format!("test/esmt/{}", idx + self.base);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                TreeOpt::Insert(key, loc, hash)
            })
            .collect::<Vec<_>>();
        let mut rng = thread_rng();
        let update_cand = (0..self.base).collect::<Vec<_>>()
            .choose_multiple(&mut rng, update_vec.len())
            .map(|key| *key)
            .collect::<Vec<_>>();
        data.extend(update_cand
            .into_iter()
            .zip(update_vec)
            .map(|(upd_idx, nloc)| {
                let key = format!("test/esmt/{}", upd_idx);
                TreeOpt::Update(key, nloc)
            })
        );
        // 打乱顺序
        data.shuffle(&mut rng);

        ESMTreeTestManager {
            data,
            tree,
        }
    }

    pub fn build_delete_test(self) -> ESMTreeTestManager {
        let mut rng = thread_rng();
        let del_cand = (0..self.base).collect::<Vec<_>>()
            .choose_multiple(&mut rng, self.q_size)
            .map(|key| *key)
            .collect::<Vec<_>>();
        // init
        let mut tree = PartionManager::new(self.range, self.p_height);
        let iter = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, point)| {
                let key = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                (key, point, hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        // 构造
        let data = del_cand.into_iter()
            .map(|idx| {
                let key = format!("test/esmt/{}", idx);
                TreeOpt::Delete(key)
            })
            .collect();
        
        ESMTreeTestManager {
            data,
            tree,
        }
    }

    pub fn build_query_test(self) -> ESMTreeTestManager {
        // init
        let mut tree = PartionManager::new(self.range.clone(), self.p_height);
        let iter = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, point)| {
                let key = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                (key, point, hash)
            });
        for (key, loc, hash) in iter {
            tree.insert(key, loc, hash);
        }
        // 构造查询数据
        let mut rng = thread_rng();
        let area = self.range;
        let x_grid = (area._max[0] - area._min[0]) / 20.0;
        let y_grid = (area._max[1] - area._min[1]) / 20.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Query(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        ESMTreeTestManager {
            data,
            tree,
        }
    }
}

impl ESMTreeTestManager {
    pub fn exec(self) {
        let mut tree = self.tree;
        for opt in self.data {
            match opt {
                TreeOpt::Insert(key, loc, hash) => {
                    tree.insert(key, loc, hash);
                },
                TreeOpt::Update(key, nloc) => {
                    tree.update(&key, nloc);
                },
                TreeOpt::Delete(key) => {
                    let _ = tree.delete(&key);
                },
                TreeOpt::Query(query) => {
                    let _ = tree.range_query(&query);
                },
            }
        }
    }
}