use crate::agent::{generate_orders, Agent};
use crate::config::RECIPE_COUNT;
use crate::market::Market;
use crate::resource::Recipe;

fn compute_role_counts(agents: &[Agent]) -> [u32; RECIPE_COUNT] {
    let mut counts = [0u32; RECIPE_COUNT];
    for agent in agents.iter() {
        if let Some(ri) = agent.role_index {
            if ri < RECIPE_COUNT {
                counts[ri] += 1;
            }
        }
    }
    counts
}

fn congestion_factor(role_index: Option<usize>, role_counts: &[u32; RECIPE_COUNT]) -> f32 {
    if let Some(ri) = role_index {
        if ri < RECIPE_COUNT {
            let count = role_counts[ri];
            if count > 0 {
                (crate::config::ROLE_SATURATION_POINT / count as f32).min(1.0)
            } else {
                1.0
            }
        } else {
            1.0
        }
    } else {
        1.0
    }
}

pub fn tick(
    tick_num: u64,
    agents: &mut [Agent],
    market: &mut Market,
    recipes: &[Recipe],
    next_order_id: &mut u64,
) {
    // 0. Calculate congestion factors
    let role_counts = compute_role_counts(agents);

    // 1. Production
    for agent in agents.iter_mut() {
        let cf = congestion_factor(agent.role_index, &role_counts);
        agent.produce(recipes, cf);
    }

    // 2. Order submission
    for agent in agents.iter() {
        let orders = generate_orders(agent, recipes, &market.last_prices, next_order_id);
        for order in orders {
            market.submit_order(order);
        }
    }

    // 3. Market clearing
    market.clear_all();

    // 4. Apply trades to agents
    for trade in &market.trades_this_tick {
        let cost = trade.price * trade.quantity;

        // Transfer gold
        agents[trade.buyer_id].gold -= cost;
        agents[trade.seller_id].gold += cost;

        // Transfer goods
        crate::inventory::add(
            &mut agents[trade.buyer_id].inventory,
            trade.resource,
            trade.quantity,
        );
        crate::inventory::sub(
            &mut agents[trade.seller_id].inventory,
            trade.resource,
            trade.quantity,
        );
    }

    // 5. Consumption
    for agent in agents.iter_mut() {
        agent.consume();
    }

    // 6. Labor Reallocation (Individualized)
    reallocate_labor(tick_num, agents, market, recipes);

    // 7. Subsistence foraging (prevents total economic collapse)
    for agent in agents.iter_mut() {
        if agent.gold < crate::config::POVERTY_THRESHOLD {
            let flour_have =
                crate::inventory::get(&agent.inventory, crate::resource::Resource::Flour);
            if flour_have < 1.0 {
                crate::inventory::add(&mut agent.inventory, crate::resource::Resource::Flour, 0.5);
            }
        }
    }
}

fn reallocate_labor(tick_num: u64, agents: &mut [Agent], market: &Market, recipes: &[Recipe]) {
    use crate::resource::Resource;
    use rand::Rng;

    let role_counts = compute_role_counts(agents);

    // 1. Calculate potential profit for each recipe
    let mut profits = Vec::new();
    for (i, recipe) in recipes.iter().enumerate() {
        if i >= RECIPE_COUNT {
            break;
        }
        // Anticipated congestion: what if I join this trade?
        let count = role_counts[i] + 1;
        let congestion = (crate::config::ROLE_SATURATION_POINT / count as f32).min(1.0);

        let revenue = recipe.output_qty * congestion * market.last_price(recipe.output);
        let input_cost: f32 = recipe
            .inputs
            .iter()
            .map(|&(res, qty)| qty * market.last_price(res))
            .sum();

        // Estimated maintenance cost (Flour + Tools/Planks)
        let maint_cost = 1.0 * market.last_price(Resource::Flour)
            + 0.2 * market.last_price(Resource::Tools)
            + 0.1 * market.last_price(Resource::Planks);

        let potential_profit = revenue - input_cost - maint_cost;
        profits.push((i, potential_profit));
    }

    if profits.is_empty() {
        return;
    }

    // Sort by profit descending
    profits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // 2. Individualized switching logic
    let mut rng = rand::thread_rng();
    for agent in agents.iter_mut() {
        if agent.is_merchant {
            continue;
        }

        // Each agent checks based on their own patience
        if !tick_num.is_multiple_of(agent.personality.patience) {
            continue;
        }

        let current_role = agent.role_index.unwrap_or(0);
        let current_profit = profits
            .iter()
            .find(|p| p.0 == current_role)
            .map(|p| p.1)
            .unwrap_or(0.0);

        // Anti-herding: pick randomly from the top N profitable roles
        let top_n = profits.len().min(crate::config::TOP_N_ROLES);
        let pick = rng.gen_range(0..top_n);
        let (best_role, best_profit) = profits[pick];

        // Profit threshold based on ambition: ambitious agents switch for less
        let switch_threshold = 1.3 * agent.personality.ambition;

        if best_profit > current_profit * switch_threshold
            && best_profit > crate::config::MIN_PROFIT_THRESHOLD
            && agent.gold >= crate::config::ROLE_SWITCH_COST
        {
            agent.switch_to_role(best_role, recipes);
        }
    }
}
