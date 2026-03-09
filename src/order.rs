use crate::resource::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub agent_id: usize,
    pub resource: Resource,
    pub side: Side,
    pub price: f32,
    pub quantity: f32,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub resource: Resource,
    pub price: f32,
    pub quantity: f32,
    pub buyer_id: usize,
    pub seller_id: usize,
}
