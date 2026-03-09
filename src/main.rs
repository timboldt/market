mod agent;
mod charts;
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
    /// RNG seed for reproducible simulations
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Maximum ticks (0 = unlimited)
    #[arg(long, default_value_t = 100)]
    ticks: u64,

    /// Milliseconds per tick
    #[arg(long, default_value_t = 500)]
    speed: u64,

    /// Generate PNG charts after simulation
    #[arg(long)]
    graph: bool,

    /// Suppress per-tick output (only show summary)
    #[arg(long)]
    quiet: bool,

    /// Number of trailing ticks to use for health summary (default: 500)
    #[arg(long, default_value_t = 500)]
    summary_window: usize,
}

fn main() {
    let args = Args::parse();

    let recipes = resource::all_recipes();
    let mut agents = agent::create_agents(args.seed);
    let mut market = market::Market::new();
    let mut next_order_id: u64 = 0;
    let mut history = charts::SimHistory::new();

    if !args.quiet {
        display::print_header();
    }

    let mut tick = 0u64;
    loop {
        tick += 1;

        simulation::tick(tick, &mut agents, &mut market, &recipes, &mut next_order_id);

        if !args.quiet {
            display::print_tick(tick, &market, &agents);
        }

        // Always record if graphing or if we need summary metrics
        if args.graph || args.quiet {
            history.record_tick(&market, &agents, &recipes);
        }

        if args.ticks > 0 && tick >= args.ticks {
            break;
        }

        if args.speed > 0 {
            thread::sleep(Duration::from_millis(args.speed));
        }
    }

    println!(
        "\nSimulation complete after {} ticks (seed={}).",
        tick, args.seed
    );

    history.print_health_summary(args.summary_window);

    if args.graph {
        println!("Generating charts...");
        history.generate_charts(&recipes);
    }
}
