pub const RESOURCE_COUNT: usize = 11;

pub const STARTING_GOLD: f32 = 200.0;
pub const DEFAULT_PRICE: f32 = 10.0;

// Agent pricing behavior: dynamic sell/buy factors based on inventory
// Centered at 1.0 when inventory is at ~50% capacity
pub const SELL_PRICE_HIGH: f32 = 1.15; // sell factor when inventory is near-empty
pub const SELL_PRICE_LOW: f32 = 0.85; // sell factor when inventory is full
pub const BUY_PRICE_HIGH: f32 = 1.15; // buy factor when desperate (0 ticks supply)
pub const BUY_PRICE_LOW: f32 = 0.85; // buy factor when well-stocked

pub const SURPLUS_THRESHOLD: f32 = 12.0;
pub const COMFORT_BUFFER_TICKS: f32 = 3.0;

pub const MAX_PRICE: f32 = 100.0;

// Merchant behavior
pub const MERCHANT_BUY_DISCOUNT: f32 = 0.80;
pub const MERCHANT_SELL_PREMIUM: f32 = 1.05;

// Consumption per tick (flour is the food everyone eats)
pub const FLOUR_CONSUMPTION: f32 = 1.0;
pub const TOOL_CONSUMPTION: f32 = 0.4;
pub const PLANK_CONSUMPTION: f32 = 0.3;

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
pub const POVERTY_SUBSIDY: f32 = 3.0;
