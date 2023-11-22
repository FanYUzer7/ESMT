use std::{thread, clone, collections::{HashMap, HashSet}};
use node::{MockChain, Request, Response, generate_channel, ServerEnd, NodeArg};
use structopt::StructOpt;
use types::hash_value::{HashValue, ESMTHasher};
use rand::{thread_rng, distributions::Uniform, prelude::Distribution};

fn generate_testset(test: &str, size: usize) -> Vec<Request> {
    let actual_size = if test == "batch" {size / 50} else {size};
    let mut res = Vec::with_capacity(actual_size);
    let mut rng = thread_rng();
    let xsampler = Uniform::new(0.0f64, 100.0f64);
    let ysampler = Uniform::new(0.0f64, 100.0f64);
    let key_sampler = Uniform::new(0usize, 10000);
    let type_sampler = Uniform::new(0usize, 3usize);
    match test {
        "insert" => {
            for i in 0..actual_size {
                let k = format!("test/mbr/{}", i);
                let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                let x = xsampler.sample(&mut rng);
                let y = ysampler.sample(&mut rng);
                res.push(Request::INSERT(k, [x,y], hash));
            }
        }
        "update" => {
            for _ in 0..actual_size {
                let k = format!("test/mbr/{}", key_sampler.sample(&mut rng));
                let x = xsampler.sample(&mut rng);
                let y = ysampler.sample(&mut rng);
                res.push(Request::UPDATE(k, [x, y]));
            }
        }
        "delete" => {
            for _ in 0..actual_size {
                let k = format!("test/mbr/{}", key_sampler.sample(&mut rng));
                res.push(Request::DELETE(k));
            }
        }
        "batch" => {
            for i in 0..actual_size {
                let mut batch = Vec::with_capacity(50);
                for j in 0..50usize {
                    let k = format!("test/mbr/{}", i*50+j);
                    let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                    let x = xsampler.sample(&mut rng);
                    let y = ysampler.sample(&mut rng);
                    batch.push((k, [x, y], hash));
                }
                res.push(Request::BATCHINSERT(batch));
            }
        }
        "mix" => {
            let mut map = HashSet::new();
            for _ in 0..actual_size {
                let t = type_sampler.sample(&mut rng);
                let idx = key_sampler.sample(&mut rng);
                let k = format!("test/mbr/{}", idx);
                match t {
                    0 => {
                        map.insert(idx);
                        let x = xsampler.sample(&mut rng);
                        let y = ysampler.sample(&mut rng);
                        let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                        res.push(Request::INSERT(k, [x,y], hash));
                    }
                    1 => {
                        let x = xsampler.sample(&mut rng);
                        let y = ysampler.sample(&mut rng);
                        if map.contains(&idx) {
                            res.push(Request::UPDATE(k, [x, y]));
                        } else {
                            let hash = ESMTHasher::default().update(k.as_bytes()).finish();
                            res.push(Request::INSERT(k, [x,y], hash));
                        }
                    }
                    2 => {
                        map.remove(&idx);
                        res.push(Request::DELETE(k));
                    }
                    _ => {}
                }
            }
        }
        _ => panic!("unknown test")
    }
    res
}

fn work(mut node: MockChain, chan: ServerEnd) {
    loop {
        let req: Request = chan.recv().unwrap();
        match req {
            Request::INSERT(key, loc, hash) => {
                node.insert(key, loc, hash);
            },
            Request::DELETE(key) => {
                node.delete(key);
            },
            Request::UPDATE(key, nloc) => {
                node.update(key, nloc);
            },
            Request::BATCHINSERT(data) => {
                node.batch_insert(data);
            },
            Request::QUIT => {
                println!("ready to quit");
                break;
            },
        }
        let _ = chan.send(Response{ hashes: node.hashes()}).unwrap();
    }
}

fn check(r1: Vec<Option<HashValue>>, r2: Vec<Option<HashValue>>, r3: Vec<Option<HashValue>>, r4: Vec<Option<HashValue>>) -> bool {
    let mut res = true;
    if !(r1.len() == r2.len() && r3.len() == r4.len() && r1.len() == r3.len()) {return false};
    for (((h1, h2), h3), h4) in r1.into_iter().zip(r2.into_iter()).zip(r3.into_iter()).zip(r4.into_iter()) {
        if !(h1 == h2 && h3 == h4 && h1 == h3) {return false;}
    }
    res
}

fn main() {
    let args = NodeArg::from_args();
    let test_size = args.size;
    let test = args.test.as_str();

    let pre_data = {
        let mut res = vec![];
        let mut rng = thread_rng();
        let xsampler = Uniform::new(0.0f64, 100.0f64);
        let ysampler = Uniform::new(0.0f64, 100.0f64);
        for i in 0..test_size {
            let k = format!("test/mbr/{}", i);
            let hash = ESMTHasher::default().update(k.as_bytes()).finish();
            let x = xsampler.sample(&mut rng);
            let y = ysampler.sample(&mut rng);
            res.push(Request::INSERT(k, [x,y], hash));
        }
        res
    };

    let test_data = generate_testset(test, test_size);
    let (c1, s1) = generate_channel();
    let (c2, s2) = generate_channel();
    let (c3, s3) = generate_channel();
    let (c4, s4) = generate_channel();

    let mut node1 = MockChain::new();
    let mut node2 = MockChain::new();
    let mut node3 = MockChain::new();
    let mut node4 = MockChain::new();

    if test == "update" || test == "delete" {
        for req in pre_data {
            if let Request::INSERT(k, loc, hash) = req {
                node1.insert(k.clone(), loc.clone(), hash.clone());
                node2.insert(k.clone(), loc.clone(), hash.clone());
                node3.insert(k.clone(), loc.clone(), hash.clone());
                node4.insert(k.clone(), loc.clone(), hash.clone());
            }
        }
        let r1 = node1.hashes();
        let r2 = node2.hashes();
        let r3 = node3.hashes();
        let r4 = node4.hashes();
        if !check(r1, r2, r3, r4) {
            println!("test failed!");
            return;
        }
    }

    let handle1 = thread::spawn(move || {
        work(node1, s1);
    });
    let handle2 = thread::spawn(move || {
        work(node2, s2);
    });
    let handle3 = thread::spawn(move || {
        work(node3, s3);
    });
    let handle4 = thread::spawn(move || {
        work(node4, s4);
    });

    let mut pass_cnt = 0;
    for req in test_data {
        let _ = c1.send(req.clone()).unwrap();
        let _ = c2.send(req.clone()).unwrap();
        let _ = c3.send(req.clone()).unwrap();
        let _ = c4.send(req.clone()).unwrap();
        let r1 = c1.recv().unwrap();
        let r2 = c2.recv().unwrap();
        let r3 = c3.recv().unwrap();
        let r4 = c4.recv().unwrap();
        if check(r1.hashes, r2.hashes, r3.hashes, r4.hashes) { pass_cnt += 1};
    }
    println!("TEST--{}: passed: {}, total: {}", test, pass_cnt, test_size);
    let _ = c1.send(Request::QUIT).unwrap();
    let _ = c2.send(Request::QUIT).unwrap();
    let _ = c3.send(Request::QUIT).unwrap();
    let _ = c4.send(Request::QUIT).unwrap();
    let _ = handle1.join().unwrap();
    let _ = handle2.join().unwrap();
    let _ = handle3.join().unwrap();
    let _ = handle4.join().unwrap();
    println!("thread quit");
}
