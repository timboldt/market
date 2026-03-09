pub const RESOURCE_COUNT: usize = 12;
pub const RECIPE_COUNT: usize = 9;

pub const STARTING_GOLD: f32 = 200.0;
pub const DEFAULT_PRICE: f32 = 10.0;

// Agent pricing behavior: dynamic sell/buy factors based on inventory
// Centered at 1.0 when inventory is at ~50% capacity
pub const SELL_PRICE_HIGH: f32 = 1.15; // sell factor when inventory is near-empty
pub const SELL_PRICE_LOW: f32 = 0.85; // sell factor when inventory is full
pub const BUY_PRICE_HIGH: f32 = 1.15; // buy factor when desperate (0 ticks supply)
pub const BUY_PRICE_LOW: f32 = 0.85; // buy factor when well-stocked

pub const SURPLUS_THRESHOLD: f32 = 18.0;
pub const COMFORT_BUFFER_TICKS: f32 = 4.0;

pub const MAX_PRICE: f32 = 200.0;

// Consumption per tick (flour is the food everyone eats)
pub const FLOUR_CONSUMPTION: f32 = 1.0;
pub const TOOL_CONSUMPTION: f32 = 0.2;
pub const PLANK_CONSUMPTION: f32 = 0.1;
pub const CLOTH_CONSUMPTION: f32 = 0.2;

// Target inventory
pub const TARGET_INVENTORY_TICKS: f32 = 15.0;
pub const INPUT_TARGET_TICKS: f32 = 12.0;

// Price memory
pub const PRICE_EMA_ALPHA: f32 = 0.3;

// Minimum order quantity
pub const MIN_ORDER_QTY: f32 = 0.1;
pub const MIN_PRICE: f32 = 0.5;

// Poverty threshold for foraging (subsistence)
pub const POVERTY_THRESHOLD: f32 = 25.0;

// Congestion: roles with more than this many agents suffer diminishing returns
pub const ROLE_SATURATION_POINT: f32 = 4.0;

// Economic Friction
pub const ROLE_SWITCH_COST: f32 = 75.0;
pub const MIN_PROFIT_THRESHOLD: f32 = 5.0;

// Anti-herding: agents pick randomly from the top N most profitable roles
pub const TOP_N_ROLES: usize = 4;
