use strum::IntoEnumIterator;

use crate::config::*;
use crate::inventory::{self, ResourceVec};
use crate::order::{Order, Side};
use crate::resource::{Recipe, Resource};

#[derive(Debug, Clone, Copy)]
pub struct Personality {
    pub ambition: f32, // 0.7 to 1.3, multiplier on profit requirement
    pub patience: u64, // 10 to 40, how often they check for a new role
}

pub struct Agent {
    pub id: usize,
    pub name: &'static str,
    pub gold: f32,
    pub inventory: ResourceVec,
    pub recipes: Vec<usize>,
    pub consumption: ResourceVec,
    pub is_merchant: bool,
    pub efficiency: f32,           // 0.0 to 1.0, affects production
    pub role_index: Option<usize>, // index of their primary role
    pub personality: Personality,
}

impl Agent {
    pub fn switch_to_role(&mut self, role_index: usize, recipes: &[Recipe]) {
        if self.is_merchant {
            return;
        }

        // Switching cost check
        if self.gold < ROLE_SWITCH_COST {
            return;
        }
        self.gold -= ROLE_SWITCH_COST;

        let recipe = &recipes[role_index];
        self.recipes = vec![role_index];
        self.role_index = Some(role_index);

        // Match on output to find a static name and set appropriate consumption
        self.name = match recipe.output {
            Resource::Grain => "Farmer",
            Resource::Timber => "Lumber",
            Resource::IronOre => "Miner",
            Resource::Flour => "Miller",
            Resource::Planks => "Sawyer",
            Resource::IronIngots => "Smelter",
            Resource::Tools => "Smith",
            Resource::Wool => "Shepherd",
            Resource::Cloth => "Weaver",
            _ => "Worker",
        };

        // Update consumption to match new role
        self.consumption = inventory::empty_vec();
        if recipe.inputs.is_empty() {
            // Primary producers: eat more flour, use tools and planks
            inventory::set(&mut self.consumption, Resource::Flour, 1.2);
            inventory::set(&mut self.consumption, Resource::Tools, TOOL_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Planks, PLANK_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Cloth, CLOTH_CONSUMPTION);
        } else if recipe.output == Resource::Tools {
            // Blacksmith: flour + planks + cloth
            inventory::set(&mut self.consumption, Resource::Flour, FLOUR_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Planks, PLANK_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Cloth, CLOTH_CONSUMPTION);
        } else if recipe.output == Resource::Cloth {
            // Weaver: flour + tools (uses own cloth)
            inventory::set(&mut self.consumption, Resource::Flour, FLOUR_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Tools, 0.3);
        } else {
            // Other intermediaries: flour + tools + planks + cloth
            inventory::set(&mut self.consumption, Resource::Flour, FLOUR_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Tools, 0.3);
            inventory::set(&mut self.consumption, Resource::Planks, PLANK_CONSUMPTION);
            inventory::set(&mut self.consumption, Resource::Cloth, CLOTH_CONSUMPTION);
        }

        // Learning penalty: switching roles drops efficiency
        self.efficiency = 0.4;
    }

    pub fn produce(&mut self, recipes: &[Recipe], congestion_factor: f32) {
        // Efficiency check: if we lack tools, we are less efficient (for primary producers)
        let tool_efficiency = if self.recipes.iter().any(|&ri| recipes[ri].inputs.is_empty()) {
            let tools = inventory::get(&self.inventory, Resource::Tools);
            if tools < 0.1 {
                0.2 // basic hand tools/manual labor
            } else {
                1.0
            }
        } else {
            1.0
        };

        let current_efficiency = self.efficiency * tool_efficiency * congestion_factor;

        for &ri in &self.recipes {
            let recipe = &recipes[ri];
            let actual_output_qty = recipe.output_qty * current_efficiency;

            // Surplus threshold: use base output_qty for the cap to avoid stalling when inefficient
            let output_cap = recipe.output_qty * SURPLUS_THRESHOLD;

            if recipe.inputs.is_empty() {
                // Primary production: cap output if surplus is too high
                let have = inventory::get(&self.inventory, recipe.output);
                if have < output_cap {
                    inventory::add(&mut self.inventory, recipe.output, actual_output_qty);
                }
                continue;
            }
            // Don't produce if output inventory is already high
            let output_have = inventory::get(&self.inventory, recipe.output);
            if output_have >= output_cap {
                continue;
            }
            // How many times can we run this recipe?
            let mut max_runs = f32::MAX;
            for &(res, qty) in recipe.inputs {
                let available = inventory::get(&self.inventory, res);
                max_runs = max_runs.min((available / qty).floor());
            }
            // Cap runs so output doesn't exceed surplus threshold
            let room = (output_cap - output_have) / actual_output_qty;
            max_runs = max_runs.min(room.floor().max(0.0));
            if max_runs < 1.0 {
                continue;
            }
            for &(res, qty) in recipe.inputs {
                inventory::sub(&mut self.inventory, res, qty * max_runs);
            }
            inventory::add(
                &mut self.inventory,
                recipe.output,
                actual_output_qty * max_runs,
            );
        }
    }

    pub fn consume(&mut self) {
        let mut flour_actual = 0.0;
        let mut flour_need = 0.0;

        for r in Resource::iter() {
            let need = inventory::get(&self.consumption, r);
            if need > 0.0 {
                let have = inventory::get(&self.inventory, r);
                let actual = have.min(need);
                inventory::sub(&mut self.inventory, r, actual);

                if r == Resource::Flour {
                    flour_actual = actual;
                    flour_need = need;
                }
            }
        }

        // Efficiency recovery/decay: food (Flour) is the primary driver
        if flour_need > 0.0 {
            let ratio = (flour_actual / flour_need).min(1.0);
            if ratio > 0.8 {
                self.efficiency = (self.efficiency + 0.05).min(1.0);
            } else if ratio < 0.3 {
                self.efficiency = (self.efficiency - 0.05).max(0.1);
            }
        }
    }
}

pub fn generate_orders(
    agent: &Agent,
    recipes: &[Recipe],
    last_prices: &ResourceVec,
    next_order_id: &mut u64,
) -> Vec<Order> {
    if agent.is_merchant {
        return merchant_orders(agent, last_prices, next_order_id);
    }

    let mut orders = Vec::new();
    let mut produced = Vec::new();
    let mut needed_inputs = Vec::new();
    let mut output_rates = inventory::empty_vec();
    let mut input_rates = inventory::empty_vec();

    for &ri in &agent.recipes {
        let recipe = &recipes[ri];
        produced.push(recipe.output);
        inventory::add(&mut output_rates, recipe.output, recipe.output_qty);
        for &(res, qty) in recipe.inputs {
            if !needed_inputs.contains(&res) {
                needed_inputs.push(res);
            }
            inventory::add(&mut input_rates, res, qty);
        }
    }

    let mut budget = agent.gold;

    // Compute cost-floor for intermediaries: don't sell below input cost + margin
    let total_input_cost: f32 = Resource::iter()
        .map(|res| inventory::get(&input_rates, res) * inventory::get(last_prices, res))
        .sum();

    // === SELL surplus production first (generates gold for buys) ===
    for r in Resource::iter() {
        if !produced.contains(&r) {
            continue;
        }
        let have = inventory::get(&agent.inventory, r);
        let consume_rate = inventory::get(&agent.consumption, r);
        let prod_rate = inventory::get(&output_rates, r);
        let price = inventory::get(last_prices, r);

        let buffer = (consume_rate + prod_rate) * COMFORT_BUFFER_TICKS;
        let surplus = have - buffer;
        if surplus > MIN_ORDER_QTY {
            // Dynamic sell pricing: lower price when inventory is full, higher when scarce
            let cap = prod_rate * SURPLUS_THRESHOLD;
            let fullness = if cap > 0.0 {
                (have / cap).clamp(0.1, 1.0)
            } else {
                0.5
            };
            let sell_factor = SELL_PRICE_HIGH - (SELL_PRICE_HIGH - SELL_PRICE_LOW) * fullness;
            // Cost floor: intermediaries never sell below input cost + 10% margin
            let cost_floor = if prod_rate > 0.0 {
                (total_input_cost / prod_rate) * 1.1
            } else {
                0.0
            };
            let sell_price = (price * sell_factor).max(cost_floor).max(MIN_PRICE);
            orders.push(Order {
                id: *next_order_id,
                agent_id: agent.id,
                resource: r,
                side: Side::Sell,
                price: sell_price,
                quantity: surplus,
            });
            *next_order_id += 1;
        }
    }

    // === BUY production inputs (priority: these generate revenue) ===
    for r in Resource::iter() {
        if !needed_inputs.contains(&r) || produced.contains(&r) {
            continue;
        }
        let have = inventory::get(&agent.inventory, r);
        let need_rate = inventory::get(&input_rates, r);
        let price = inventory::get(last_prices, r);

        let target = need_rate * INPUT_TARGET_TICKS;
        let deficit = target - have;
        if deficit > MIN_ORDER_QTY && budget > 1.0 {
            let ticks_supply = if need_rate > 0.0 {
                have / need_rate
            } else {
                f32::MAX
            };
            // Dynamic buy pricing: higher urgency when supply is low
            let urgency = (1.0 - ticks_supply / (INPUT_TARGET_TICKS * 2.0)).clamp(0.0, 1.0);
            let buy_factor = BUY_PRICE_LOW + (BUY_PRICE_HIGH - BUY_PRICE_LOW) * urgency;
            let buy_price = (price * buy_factor).max(MIN_PRICE);
            let max_qty = budget / buy_price;
            let qty = deficit.min(max_qty);
            if qty > MIN_ORDER_QTY {
                budget -= qty * buy_price;
                orders.push(Order {
                    id: *next_order_id,
                    agent_id: agent.id,
                    resource: r,
                    side: Side::Buy,
                    price: buy_price,
                    quantity: qty,
                });
                *next_order_id += 1;
            }
        }
    }

    // === BUY consumption goods ===
    for r in Resource::iter() {
        let consume_rate = inventory::get(&agent.consumption, r);
        if consume_rate <= 0.0 || produced.contains(&r) {
            continue;
        }
        // Skip if already ordered as input
        if needed_inputs.contains(&r) {
            continue;
        }
        let have = inventory::get(&agent.inventory, r);
        let price = inventory::get(last_prices, r);
        let ticks_supply = have / consume_rate;
        let target = consume_rate * TARGET_INVENTORY_TICKS;
        let deficit = target - have;

        if deficit > MIN_ORDER_QTY && budget > 1.0 {
            // Dynamic buy pricing: higher urgency when supply is low
            let urgency = (1.0 - ticks_supply / (TARGET_INVENTORY_TICKS * 2.0)).clamp(0.0, 1.0);

            // Wealth effect: rich agents are less price-sensitive for food/necessities
            let wealth_factor = if agent.gold > STARTING_GOLD * 2.0 {
                1.1 // willing to pay 10% more
            } else {
                1.0
            };

            let buy_factor =
                (BUY_PRICE_LOW + (BUY_PRICE_HIGH - BUY_PRICE_LOW) * urgency) * wealth_factor;
            let buy_price = (price * buy_factor).max(MIN_PRICE);
            let max_qty = budget / buy_price;
            let qty = deficit.min(max_qty);
            if qty > MIN_ORDER_QTY {
                budget -= qty * buy_price;
                orders.push(Order {
                    id: *next_order_id,
                    agent_id: agent.id,
                    resource: r,
                    side: Side::Buy,
                    price: buy_price,
                    quantity: qty,
                });
                *next_order_id += 1;
            }
        }
    }

    orders
}

fn merchant_orders(
    agent: &Agent,
    last_prices: &ResourceVec,
    next_order_id: &mut u64,
) -> Vec<Order> {
    let mut orders = Vec::new();
    let mut budget = agent.gold;

    for r in Resource::iter() {
        let have = inventory::get(&agent.inventory, r);
        let price = inventory::get(last_prices, r);

        // Sell with a tighter spread to increase volume
        let reserve = if r == Resource::Flour { 5.0 } else { 0.0 };
        let sellable = have - reserve;
        if sellable > MIN_ORDER_QTY {
            let sell_price = (price * 1.02).max(MIN_PRICE);
            orders.push(Order {
                id: *next_order_id,
                agent_id: agent.id,
                resource: r,
                side: Side::Sell,
                price: sell_price,
                quantity: sellable,
            });
            *next_order_id += 1;
        }

        // Buy at market price if inventory is low, otherwise discount
        if budget > 1.0 {
            let buy_price = if have < 5.0 {
                price * 0.95
            } else {
                price * 0.85
            };
            let buy_price = buy_price.max(MIN_PRICE);
            let max_qty = (budget * 0.1) / buy_price; // use 10% of budget per resource
            if max_qty > MIN_ORDER_QTY {
                budget -= max_qty * buy_price;
                orders.push(Order {
                    id: *next_order_id,
                    agent_id: agent.id,
                    resource: r,
                    side: Side::Buy,
                    price: buy_price,
                    quantity: max_qty,
                });
                *next_order_id += 1;
            }
        }
    }

    orders
}

pub fn create_agents(seed: u64) -> Vec<Agent> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);
    let mut agents = Vec::new();
    let mut id = 0;

    // Define role counts (~30 agents total)
    let roles = vec![
        (
            "Farmer",
            0,
            vec![0],
            vec![
                (Resource::Flour, 1.2),
                (Resource::Tools, TOOL_CONSUMPTION),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            5,
        ),
        (
            "Miller",
            3,
            vec![3],
            vec![
                (Resource::Flour, FLOUR_CONSUMPTION),
                (Resource::Tools, 0.3),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            4,
        ),
        (
            "Lumber",
            1,
            vec![1],
            vec![
                (Resource::Flour, 1.2),
                (Resource::Tools, TOOL_CONSUMPTION),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            4,
        ),
        (
            "Sawyer",
            4,
            vec![4],
            vec![
                (Resource::Flour, FLOUR_CONSUMPTION),
                (Resource::Tools, 0.3),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            2,
        ),
        (
            "Miner",
            2,
            vec![2],
            vec![
                (Resource::Flour, 1.2),
                (Resource::Tools, TOOL_CONSUMPTION),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            3,
        ),
        (
            "Smelter",
            5,
            vec![5],
            vec![
                (Resource::Flour, FLOUR_CONSUMPTION),
                (Resource::Tools, 0.3),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            2,
        ),
        (
            "Smith",
            6,
            vec![6],
            vec![
                (Resource::Flour, FLOUR_CONSUMPTION),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            3,
        ),
        (
            "Shepherd",
            7,
            vec![7],
            vec![
                (Resource::Flour, 1.2),
                (Resource::Tools, TOOL_CONSUMPTION),
                (Resource::Planks, PLANK_CONSUMPTION),
                (Resource::Cloth, CLOTH_CONSUMPTION),
            ],
            3,
        ),
        (
            "Weaver",
            8,
            vec![8],
            vec![(Resource::Flour, FLOUR_CONSUMPTION), (Resource::Tools, 0.3)],
            2,
        ),
    ];

    for (name, role_idx, recipes, cons, count) in roles {
        for _ in 0..count {
            let mut c = inventory::empty_vec();
            for (res, qty) in &cons {
                inventory::set(&mut c, *res, *qty);
            }

            let mut inv = inventory::empty_vec();
            // Start with some basic food, tools, and cloth
            inventory::set(&mut inv, Resource::Flour, rng.gen_range(10.0..20.0));
            inventory::set(&mut inv, Resource::Tools, rng.gen_range(2.0..5.0));
            inventory::set(&mut inv, Resource::Cloth, rng.gen_range(2.0..4.0));

            // Initial role-specific inventory
            if role_idx == 3 {
                inventory::set(&mut inv, Resource::Grain, 20.0);
            }
            if role_idx == 4 {
                inventory::set(&mut inv, Resource::Timber, 15.0);
            }
            if role_idx == 5 {
                inventory::set(&mut inv, Resource::IronOre, 10.0);
                inventory::set(&mut inv, Resource::Timber, 5.0);
            }
            if role_idx == 6 {
                inventory::set(&mut inv, Resource::IronIngots, 5.0);
                inventory::set(&mut inv, Resource::Planks, 5.0);
            }
            if role_idx == 8 {
                inventory::set(&mut inv, Resource::Wool, 15.0);
            }

            agents.push(Agent {
                id,
                name,
                gold: STARTING_GOLD * rng.gen_range(0.8..1.2),
                inventory: inv,
                recipes: recipes.clone(),
                consumption: c,
                is_merchant: false,
                efficiency: 1.0,
                role_index: Some(role_idx),
                personality: Personality {
                    ambition: rng.gen_range(0.7..1.3),
                    patience: rng.gen_range(10..40),
                },
            });
            id += 1;
        }
    }

    // 1 Merchant
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    inventory::set(&mut c, Resource::Cloth, CLOTH_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 20.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    inventory::set(&mut inv, Resource::Cloth, 3.0);
    agents.push(Agent {
        id,
        name: "Merchant",
        gold: STARTING_GOLD * 5.0,
        inventory: inv,
        recipes: vec![],
        consumption: c,
        is_merchant: true,
        efficiency: 1.0,
        role_index: None,
        personality: Personality {
            ambition: 1.0,
            patience: 100,
        },
    });

    agents
}
