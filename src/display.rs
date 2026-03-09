use strum::IntoEnumIterator;

use crate::agent::Agent;
use crate::market::Market;
use crate::resource::Resource;

pub fn print_header() {
    print!("{:>5} |", "Tick");
    for r in Resource::iter() {
        print!(" {:>6}", r.short_name());
    }
    println!(" | Trades");
    println!("{}", "-".repeat(100));
}

pub fn print_tick(tick: u64, market: &Market, agents: &[Agent]) {
    // Track which resources had trades this tick
    let mut had_trade = [false; crate::config::RESOURCE_COUNT];
    for trade in &market.trades_this_tick {
        had_trade[trade.resource as usize] = true;
    }

    print!("{:>5} |", tick);
    for r in Resource::iter() {
        if had_trade[r as usize] {
            print!(" {:>6.1}", market.last_price(r));
        } else {
            print!("     --");
        }
    }
    println!(" | {:>3}", market.trade_count());

    // Print sample agent (Farmer) every 10 ticks
    if tick.is_multiple_of(10) {
        print_agent_summary(agents);
    }
}

fn print_agent_summary(agents: &[Agent]) {
    println!("  Agents:");
    for agent in agents {
        print!("    {:>10} gold:{:>6.1} |", agent.name, agent.gold);
        for r in Resource::iter() {
            let qty = crate::inventory::get(&agent.inventory, r);
            if qty > 0.01 {
                print!(" {}:{:.0}", r.short_name().trim(), qty);
            }
        }
        println!();
    }
    println!();
}
