use std::{collections::{HashMap}, str::FromStr, time::{Instant, Duration}, task::Poll};
use authentic_rtree::{mrtree::MerkleRTree as MRTree, shape::Rect, esmtree::PartionManager};
use rand::{thread_rng, seq::SliceRandom, distributions::Uniform, prelude::Distribution};
use types::hash_value::{HashValue, ESMTHasher};
use authentic_rtree::mrtree::{NODE_SPLIT, NODE_TRAVERSE};
use crate::read_dataset;
use std::path::PathBuf;
use threadpool::ThreadPool;
use std::sync::{Arc, Barrier};

pub enum TreeOpt {
    Insert(String, [f64; 2], HashValue),
    Update(String, [f64; 2]),
    Delete(String),
    Query(Rect<f64, 2>),
    Edge(Rect<f64, 2>),
}

impl TreeOpt {
    pub fn to_insert(self) -> Option<(String, [f64; 2], HashValue)> {
        if let TreeOpt::Insert(k, l, h) = self {
            Some((k, l, h))
        } else {
            None
        }
    }
}

pub struct MRTreeTestManager {
    pub data: Vec<TreeOpt>,
    pub tree: MRTree<f64, 2, 51>,
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
        self.data.shuffle(&mut rng);
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
    pub fn exec(self) -> (MRTree<f64, 2, 51>, f64) {
        let mut keymap = self.keymap;
        let mut tree = self.tree;
        // let mut node_travese_cnt = 0u64;
        // let mut node_split_cnt = 0u64;
        let data_size = self.data.len() as f64;
        let mut value = Duration::new(0,0);
        for opt in self.data {
            // (*NODE_TRAVERSE).lock().unwrap().clear();
            // (*NODE_SPLIT).lock().unwrap().clear();
            match opt {
                TreeOpt::Insert(key, loc, hash) => {
                    let timer_start = Instant::now();
                    tree.insert(key.clone(), loc.clone(), hash);
                    value += Instant::elapsed(&timer_start);
                    keymap.insert(key, loc);
                },
                TreeOpt::Update(key, nloc) => {
                    let timer_start = Instant::now();
                    let oloc = keymap.get(&key).unwrap();
                    tree.update_loc(&key, oloc, nloc.clone());
                    value += Instant::elapsed(&timer_start);
                    let _ = keymap.insert(key, nloc);
                },
                TreeOpt::Delete(key) => {
                    let timer_start = Instant::now();
                    let oloc = keymap.get(&key).unwrap();
                    let _ = tree.delete(&key, oloc);
                    value += Instant::elapsed(&timer_start);
                    let _ = keymap.remove(&key);
                },
                TreeOpt::Query(query) => {
                    let timer_start = Instant::now();
                    let _ = tree.range_query(&query);
                    value += Instant::elapsed(&timer_start);
                },
                _ => {}
            }
            // node_travese_cnt += (*NODE_TRAVERSE).lock().unwrap().value();
            // node_split_cnt += (*NODE_SPLIT).lock().unwrap().value();
        }
        // println!("average node traverse count = {}", (node_travese_cnt as f64) / data_size);
        // println!("average node split count = {}", (node_split_cnt as f64) / data_size);
        (tree, value.as_secs_f64())
        // (tree, keymap)
    }
}

pub struct ESMTreeTestManager {
    pub prepare: Vec<Vec<(String, [f64; 2], HashValue)>>,
    pub data: Vec<TreeOpt>,
    pub tree: PartionManager<f64, 2, 51>,
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
            p_height: 4,
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
    pub fn range(mut self, min: [f64; 2], max: [f64; 2]) -> Self {
        self.range = Rect::new(min, max);
        self
    }

    pub fn set_testset(mut self, db: &Vec<[f64; 2]>) -> Self {
        let mut rng = thread_rng();
        self.data = db.choose_multiple(&mut rng, self.base + self.q_size)
            .map(|p| p.clone())
            .collect();
        self.data.shuffle(&mut rng);
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
            prepare: vec![],
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
            prepare: vec![],
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
            prepare: vec![],
            data,
            tree,
        }
    }

    pub fn build_query_test(self, size: f64) -> ESMTreeTestManager {
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
        let x_grid = (area._max[0] - area._min[0])*size/2.0;
        let y_grid = (area._max[1] - area._min[1])*size/2.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Query(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        ESMTreeTestManager {
            prepare: vec![],
            data,
            tree,
        }
    }

    pub fn build_edge_test(self, size: f64) -> ESMTreeTestManager {
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
        let x_grid = (area._max[0] - area._min[0])*size/2.0;
        let y_grid = (area._max[1] - area._min[1])*size/2.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Edge(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        ESMTreeTestManager {
            prepare: vec![],
            data,
            tree,
        }
    }

    pub fn build_batch_construct(self, batch_size: usize) -> ESMTreeTestManager {
        let tree = PartionManager::new(self.range, self.p_height);
        let mut data = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx,l)| {
                let k = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                (k, l, hash) 
            })
            .collect::<Vec<_>>();
    
        let mut prepare = vec![];
        while data.len() >= batch_size {
            let fetched = data.drain(..batch_size).collect();
            prepare.push(fetched);
        }
        let remain = data.drain(..).collect();
        prepare.push(remain);

        ESMTreeTestManager {
            prepare,
            data: vec![],
            tree,
        }
    }

    pub fn build_batch_insert(self, batch_size: usize) -> ESMTreeTestManager {
        let insert_vec = self.data[self.base..].to_vec();
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
                (key, loc, hash)
            })
            .collect::<Vec<_>>();
        let mut prepare = vec![];
        while data.len() >= batch_size {
            let fetched = data.drain(..batch_size).collect();
            prepare.push(fetched);
        }
        let remain = data.drain(..).collect();
        prepare.push(remain);

        ESMTreeTestManager {
            prepare,
            data: vec![],
            tree,
        }
    }

    pub fn batch_build_query(self, batch_size: usize, size: f64) -> ESMTreeTestManager {
        // 初始化esmt
        let mut tree = PartionManager::new(self.range.clone(), self.p_height);
        let mut iter = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, point)| {
                let key = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                (key, point, hash)
            }).collect::<Vec<_>>();
        while iter.len() >= batch_size {
            let batch = iter.drain(..batch_size).collect::<Vec<_>>();
            tree.batch_insert(batch);
        }
        // 构造测试数据
        // 构造查询数据
        let mut rng = thread_rng();
        let area = self.range;
        let x_grid = (area._max[0] - area._min[0])*size/2.0;
        let y_grid = (area._max[1] - area._min[1])*size/2.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Edge(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        ESMTreeTestManager {
            prepare: vec![],
            data,
            tree,
        }
    }

    pub fn batch_build_iter_query(self, batch_size: usize, size: f64) -> ESMTreeTestManager {
        // 初始化esmt
        let mut tree = PartionManager::new(self.range.clone(), self.p_height);
        let mut iter = self.data.into_iter()
            .enumerate()
            .take(self.base)
            .map(|(idx, point)| {
                let key = format!("test/esmt/{}", idx);
                let hash = ESMTHasher::default().update(key.as_bytes()).finish();
                (key, point, hash)
            }).collect::<Vec<_>>();
        while iter.len() >= batch_size {
            let batch = iter.drain(..batch_size).collect::<Vec<_>>();
            tree.batch_iter_insert(batch);
        }
        // 构造测试数据
        // 构造查询数据
        let mut rng = thread_rng();
        let area = self.range;
        let x_grid = (area._max[0] - area._min[0])*size/2.0;
        let y_grid = (area._max[1] - area._min[1])*size/2.0;
        let x_sample = Uniform::new(area._min[0] + x_grid, area._max[0] - x_grid);
        let y_sample = Uniform::new(area._min[1] + y_grid, area._max[1] - y_grid);
        let data = (0..self.q_size).map(|_| {
            let x = x_sample.sample(&mut rng);
            let y = y_sample.sample(&mut rng);
            TreeOpt::Edge(Rect::new([x - x_grid, y - y_grid], [x + x_grid, y + y_grid]))
        }).collect();

        ESMTreeTestManager {
            prepare: vec![],
            data,
            tree,
        }
    }
}

impl ESMTreeTestManager {
    pub fn exec(self) -> (PartionManager<f64, 2, 51>, f64) {
        let mut tree = self.tree;
        let data_size = self.data.len() as f64;
        let mut value = Duration::new(0,0);
        for opt in self.data {
            match opt {
                TreeOpt::Insert(key, loc, hash) => {
                    let timer_start = Instant::now();
                    tree.insert(key, loc, hash);
                    value += Instant::elapsed(&timer_start);
                },
                TreeOpt::Update(key, nloc) => {
                    let timer_start = Instant::now();
                    tree.update(&key, nloc);
                    value += Instant::elapsed(&timer_start);
                },
                TreeOpt::Delete(key) => {
                    let timer_start = Instant::now();
                    let _ = tree.delete(&key);
                    value += Instant::elapsed(&timer_start);
                },
                TreeOpt::Query(query) => {
                    let timer_start = Instant::now();
                    let _ = tree.range_query(&query);
                    value += Instant::elapsed(&timer_start);
                },
                TreeOpt::Edge(query) => {
                    let timer_start = Instant::now();
                    let _ = tree.edge_query(&query);
                    value += Instant::elapsed(&timer_start);
                }
            }
        }
        (tree, value.as_secs_f64())
    }

    pub fn exec_batch(self) -> (PartionManager<f64, 2, 51>, f64) {
        let mut tree = self.tree;
        let mut value = Duration::new(0,0);
        for batch in self.prepare {
            let timer_start = Instant::now();
            tree.batch_insert(batch);
            value += Instant::elapsed(&timer_start);
        }
        (tree, value.as_secs_f64())
    }

    pub fn exec_parallel(self) -> (PartionManager<f64, 2, 51>, f64, Vec<f64>) {
        let mut tree = self.tree;
        let mut value = Duration::new(0,0);
        let (jobs, workers) = (16usize, 24usize);
        let pool = ThreadPool::new(workers);
        let mut thread_time = vec![0.0f64;16];
        for opt in self.data {
            if let TreeOpt::Query(query) = opt {
                // let timer_start = Instant::now();
                let (e, _, times) = tree.parallel_query(&query, &pool);
                // value += Instant::elapsed(&timer_start);
                value += e;
                for i in 0..16usize {
                    thread_time[i] += times[i];
                }
            }
        }
        (tree, value.as_secs_f64(), thread_time)
    }
}

pub fn manu_test() {
    // let data = read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap();
    // let mut value = 0f64;
    // let mut times = Vec::new();
    // for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000].iter() {
    // // for i in [512000usize, 1024000].iter() {
    //     for _  in 0..20 {
    //         let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&data).build_update_test(1.0);
    //         let output = mrt.exec();
    //         value += output.1;
    //     }
    //     times.push((*i, value));
    //     value = 0f64;
    // }
    // for (data_size, avg_time) in times {
    //     println!("with data size = {}, get average exec time = {}", data_size, avg_time);
    // }
}