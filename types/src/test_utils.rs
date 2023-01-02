use std::collections::BTreeSet;
use rand::{Rng, thread_rng};
use crate::hash_value::{ESMTHasher, HashValue};

pub fn num_hash(data: i32) -> HashValue {
    let bytes = data.to_le_bytes();
    let hasher = ESMTHasher::default();
    hasher.update(&bytes).finish()
}

pub fn calc_hash(set: &BTreeSet<HashValue>) -> HashValue {
    let hasher = set.iter()
        .fold(ESMTHasher::default(), |h, hash| {
            h.update(hash.as_ref())
        });
    hasher.finish()
}

pub fn generate_points<V, const D: usize>(min: [V; D], max: [V; D], cnt: usize) -> Vec<[V; D]>
    where
        V: Default + Copy + rand::distributions::uniform::SampleUniform + std::cmp::PartialOrd,
{
    let mut rand = thread_rng();
    let mut points = vec![];
    for _ in 0..cnt {
        let mut p = [V::default(); D];
        for i in 0..D {
            p[i] = rand.gen_range(min[i]..=max[i]);
        }
        points.push(p);
    }
    points
}