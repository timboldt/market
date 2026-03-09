use crate::config::RESOURCE_COUNT;
use crate::resource::Resource;

pub type ResourceVec = [f32; RESOURCE_COUNT];

pub fn empty_vec() -> ResourceVec {
    [0.0; RESOURCE_COUNT]
}

pub fn ones_vec() -> ResourceVec {
    [1.0; RESOURCE_COUNT]
}

pub fn get(v: &ResourceVec, r: Resource) -> f32 {
    v[r as usize]
}

pub fn set(v: &mut ResourceVec, r: Resource, val: f32) {
    v[r as usize] = val;
}

pub fn add(v: &mut ResourceVec, r: Resource, amount: f32) {
    v[r as usize] += amount;
}

pub fn sub(v: &mut ResourceVec, r: Resource, amount: f32) {
    v[r as usize] -= amount;
}

pub fn _has(v: &ResourceVec, r: Resource, amount: f32) -> bool {
    v[r as usize] >= amount
}
