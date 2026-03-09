mod agent;
mod config;
mod display;
mod inventory;
mod market;
mod order;
mod orderbook;
mod resource;
mod simulation;

use std::thread;
use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
#[command(name = "market", about = "Medieval market simulator")]
struct Args {
    /// RNG seed (unused for now, reserved for future randomization)
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Maximum ticks (0 = unlimited)
    #[arg(long, default_value_t = 100)]
    ticks: u64,

    /// Milliseconds per tick
    #[arg(long, default_value_t = 500)]
    speed: u64,
}

fn main() {
    let args = Args::parse();

    let recipes = resource::all_recipes();
    let mut agents = agent::create_agents();
    let ctx = agent::AgentContext::build(&agents, &recipes);
    let mut market = market::Market::new();
    let mut next_order_id: u64 = 0;

    display::print_header();

    let mut tick = 0u64;
    loop {
        tick += 1;

        simulation::tick(&mut agents, &mut market, &recipes, &ctx, &mut next_order_id);
        display::print_tick(tick, &market, &agents);

        if args.ticks > 0 && tick >= args.ticks {
            break;
        }

        if args.speed > 0 {
            thread::sleep(Duration::from_millis(args.speed));
        }
    }

    println!("\nSimulation complete after {} ticks.", tick);
}
