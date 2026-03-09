pub const RESOURCE_COUNT: usize = 12;
pub const RECIPE_COUNT: usize = 9;

pub const STARTING_GOLD: f32 = 100.0;
pub const DEFAULT_PRICE: f32 = 10.0;

// Agent pricing behavior: dynamic sell/buy factors based on inventory
// Centered at 1.0 when inventory is at ~50% capacity
pub const SELL_PRICE_HIGH: f32 = 1.15; // sell factor when inventory is near-empty
pub const SELL_PRICE_LOW: f32 = 0.85; // sell factor when inventory is full
pub const BUY_PRICE_HIGH: f32 = 1.15; // buy factor when desperate (0 ticks supply)
pub const BUY_PRICE_LOW: f32 = 0.85; // buy factor when well-stocked

pub const SURPLUS_THRESHOLD: f32 = 10.0;
pub const COMFORT_BUFFER_TICKS: f32 = 2.0;

pub const MAX_PRICE: f32 = 500.0;

// Consumption per tick (flour is the food everyone eats)
pub const FLOUR_CONSUMPTION: f32 = 1.0;
pub const TOOL_CONSUMPTION: f32 = 0.3;
pub const PLANK_CONSUMPTION: f32 = 0.2;
pub const CLOTH_CONSUMPTION: f32 = 0.2;

// Target inventory
pub const TARGET_INVENTORY_TICKS: f32 = 4.0;
pub const INPUT_TARGET_TICKS: f32 = 3.0;

// Price memory
pub const PRICE_EMA_ALPHA: f32 = 0.15;

// Spoilage rates (fraction of inventory lost per tick)
pub const SPOILAGE_PERISHABLE: f32 = 0.05; // Grain, Flour, Herbs
pub const SPOILAGE_RAW: f32 = 0.02; // Timber, Wool, Clay, IronOre, Stone
pub const SPOILAGE_PROCESSED: f32 = 0.01; // Planks, Ingots, Tools, Cloth

// Minimum order quantity
pub const MIN_ORDER_QTY: f32 = 0.1;
pub const MIN_PRICE: f32 = 0.5;

// Royal tax rate (fraction of gold collected per tick)
pub const TAX_RATE: f32 = 0.05;

// Congestion: roles with more than this many agents suffer diminishing returns
pub const ROLE_SATURATION_POINT: f32 = 40.0;

// Economic Friction
pub const ROLE_SWITCH_COST: f32 = 75.0;
pub const MIN_PROFIT_THRESHOLD: f32 = 5.0;

// Anti-herding: agents pick randomly from the top N most profitable roles
pub const TOP_N_ROLES: usize = 4;
