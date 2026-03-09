pub const RESOURCE_COUNT: usize = 11;

pub const STARTING_GOLD: f32 = 200.0;
pub const DEFAULT_PRICE: f32 = 10.0;

// Agent pricing behavior
pub const SELL_PRICE_FACTOR: f32 = 1.0;
pub const BUY_PRICE_FACTOR: f32 = 1.0;
pub const URGENCY_PREMIUM: f32 = 1.15;

pub const URGENCY_THRESHOLD: f32 = 5.0;
pub const SURPLUS_THRESHOLD: f32 = 20.0;
pub const COMFORT_BUFFER_TICKS: f32 = 3.0;

// Merchant behavior
pub const MERCHANT_BUY_DISCOUNT: f32 = 0.92;
pub const MERCHANT_SELL_PREMIUM: f32 = 1.08;

// Consumption per tick (flour is the food everyone eats)
pub const FLOUR_CONSUMPTION: f32 = 1.0;
pub const TOOL_CONSUMPTION: f32 = 0.2;

// Target inventory
pub const TARGET_INVENTORY_TICKS: f32 = 10.0;
pub const INPUT_TARGET_TICKS: f32 = 8.0;

// Price memory
pub const PRICE_EMA_ALPHA: f32 = 0.3;

// Minimum order quantity
pub const MIN_ORDER_QTY: f32 = 0.1;
pub const MIN_PRICE: f32 = 0.5;

// Gold subsidy: agents below this threshold get a small income each tick
// (represents off-screen economic activity)
pub const POVERTY_THRESHOLD: f32 = 20.0;
pub const POVERTY_SUBSIDY: f32 = 5.0;
