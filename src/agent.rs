use strum::IntoEnumIterator;

use crate::config::*;
use crate::inventory::{self, ResourceVec};
use crate::order::{Order, Side};
use crate::resource::{Recipe, Resource};

pub struct Agent {
    pub id: usize,
    pub name: &'static str,
    pub gold: f32,
    pub inventory: ResourceVec,
    pub recipes: Vec<usize>,
    pub consumption: ResourceVec,
    pub is_merchant: bool,
    pub efficiency: f32, // 0.0 to 1.0, affects production
}

impl Agent {
    pub fn produce(&mut self, recipes: &[Recipe]) {
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

        let current_efficiency = self.efficiency * tool_efficiency;

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

pub struct AgentContext {
    pub produced_resources: Vec<Vec<Resource>>,
    pub input_resources: Vec<Vec<Resource>>,
    pub output_rate: Vec<ResourceVec>,
    pub input_rate: Vec<ResourceVec>, // how much of each input consumed per production cycle
}

impl AgentContext {
    pub fn build(agents: &[Agent], recipes: &[Recipe]) -> Self {
        let mut produced = Vec::new();
        let mut inputs = Vec::new();
        let mut output_rate = Vec::new();
        let mut input_rate = Vec::new();

        for agent in agents {
            let mut pr = Vec::new();
            let mut ir = Vec::new();
            let mut orates = inventory::empty_vec();
            let mut irates = inventory::empty_vec();
            for &ri in &agent.recipes {
                let recipe = &recipes[ri];
                pr.push(recipe.output);
                inventory::add(&mut orates, recipe.output, recipe.output_qty);
                for &(res, qty) in recipe.inputs {
                    if !ir.contains(&res) {
                        ir.push(res);
                    }
                    inventory::add(&mut irates, res, qty);
                }
            }
            produced.push(pr);
            inputs.push(ir);
            output_rate.push(orates);
            input_rate.push(irates);
        }

        Self {
            produced_resources: produced,
            input_resources: inputs,
            output_rate,
            input_rate,
        }
    }
}

pub fn generate_orders_with_context(
    agent: &Agent,
    ctx: &AgentContext,
    last_prices: &ResourceVec,
    next_order_id: &mut u64,
) -> Vec<Order> {
    if agent.is_merchant {
        return merchant_orders(agent, last_prices, next_order_id);
    }

    let mut orders = Vec::new();
    let produced = &ctx.produced_resources[agent.id];
    let needed_inputs = &ctx.input_resources[agent.id];
    let output_rates = &ctx.output_rate[agent.id];
    let input_rates = &ctx.input_rate[agent.id];

    let mut budget = agent.gold;

    // Compute cost-floor for intermediaries: don't sell below input cost + margin
    let total_input_cost: f32 = Resource::iter()
        .map(|res| inventory::get(input_rates, res) * inventory::get(last_prices, res))
        .sum();

    // === SELL surplus production first (generates gold for buys) ===
    for r in Resource::iter() {
        if !produced.contains(&r) {
            continue;
        }
        let have = inventory::get(&agent.inventory, r);
        let consume_rate = inventory::get(&agent.consumption, r);
        let prod_rate = inventory::get(output_rates, r);
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
        let need_rate = inventory::get(input_rates, r);
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

pub fn create_agents() -> Vec<Agent> {
    let mut agents = Vec::new();

    // 0: Farmer - produces grain, consumes flour (food) + tools + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, 1.2);
    inventory::set(&mut c, Resource::Tools, TOOL_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::Tools, 5.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 0,
        name: "Farmer",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![0],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 1: Miller - grain -> flour, consumes flour (food) + tools + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Tools, 0.3);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Grain, 20.0);
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::Tools, 3.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 1,
        name: "Miller",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![3],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 2: Lumberjack - produces timber, consumes flour + tools + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, 1.2);
    inventory::set(&mut c, Resource::Tools, TOOL_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::Tools, 5.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 2,
        name: "Lumberjack",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![1],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 3: Sawmill - timber -> planks, consumes flour + tools
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Tools, 0.3);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::Timber, 15.0);
    inventory::set(&mut inv, Resource::Tools, 3.0);
    agents.push(Agent {
        id: 3,
        name: "Sawmill",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![4],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 4: Miner - produces iron ore, consumes flour + tools + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, 1.2);
    inventory::set(&mut c, Resource::Tools, TOOL_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::Tools, 5.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 4,
        name: "Miner",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![2],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 5: Smelter - iron ore + timber -> iron ingots, consumes flour + tools + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Tools, 0.3);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::IronOre, 10.0);
    inventory::set(&mut inv, Resource::Timber, 5.0);
    inventory::set(&mut inv, Resource::Tools, 3.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 5,
        name: "Smelter",
        gold: STARTING_GOLD,
        inventory: inv,
        recipes: vec![5],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 6: Blacksmith - iron ingots + planks -> tools, consumes flour + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 15.0);
    inventory::set(&mut inv, Resource::IronIngots, 5.0);
    inventory::set(&mut inv, Resource::Planks, 5.0);
    agents.push(Agent {
        id: 6,
        name: "Blacksmith",
        gold: STARTING_GOLD * 1.5,
        inventory: inv,
        recipes: vec![6],
        consumption: c,
        is_merchant: false,
        efficiency: 1.0,
    });

    // 7: Merchant - buys low sells high, consumes flour + planks
    let mut c = inventory::empty_vec();
    inventory::set(&mut c, Resource::Flour, FLOUR_CONSUMPTION);
    inventory::set(&mut c, Resource::Planks, PLANK_CONSUMPTION);
    let mut inv = inventory::empty_vec();
    inventory::set(&mut inv, Resource::Flour, 20.0);
    inventory::set(&mut inv, Resource::Planks, 3.0);
    agents.push(Agent {
        id: 7,
        name: "Merchant",
        gold: STARTING_GOLD * 5.0,
        inventory: inv,
        recipes: vec![],
        consumption: c,
        is_merchant: true,
        efficiency: 1.0,
    });

    agents
}
