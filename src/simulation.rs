use crate::agent::{generate_orders_with_context, Agent, AgentContext};
use crate::market::Market;
use crate::resource::Recipe;

pub fn tick(
    agents: &mut [Agent],
    market: &mut Market,
    recipes: &[Recipe],
    ctx: &AgentContext,
    next_order_id: &mut u64,
) {
    // 1. Production
    for agent in agents.iter_mut() {
        agent.produce(recipes);
    }

    // 2. Order submission
    for agent in agents.iter() {
        let orders = generate_orders_with_context(agent, ctx, &market.last_prices, next_order_id);
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

    // 6. Subsistence foraging (prevents total economic collapse)
    // If an agent is very poor and has no food, they can forage for a tiny amount of basic resources.
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
