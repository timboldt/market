# Medieval Market Simulator

An agent-based economic simulation of a medieval marketplace. Autonomous agents produce, trade, and consume goods through a price-discovery order book, with emergent market dynamics including business cycles, role specialization, and wealth distribution.

## Overview

The simulation models a small medieval economy where agents specialize in different roles (farming, milling, mining, etc.) and trade goods on an open market. Each agent independently decides what to produce, what to buy and sell, and at what price — there is no central planner. Prices emerge from supply and demand through a double-auction order book.

### Key Features

- **Autonomous agents** with individual personalities (ambition, patience) that affect economic decisions
- **Production chains** — raw materials are refined into intermediate and final goods
- **Price discovery** via order book with price-time priority matching
- **Role switching** — agents evaluate profitability and can change occupations
- **Royal treasury** — proportional taxation with equal redistribution prevents gold hoarding
- **Spoilage** — goods decay over time, preventing infinite stockpiling
- **Congestion** — diminishing returns when too many agents share a role

## Economy

### Resources

| Resource | Type | Notes |
|----------|------|-------|
| Grain | Raw | Produced by Farmers |
| Timber | Raw | Produced by Lumberjacks |
| Iron Ore | Raw | Produced by Miners |
| Wool | Raw | Produced by Shepherds |
| Flour | Processed | Milled from Grain; universal food |
| Planks | Processed | Sawn from Timber |
| Iron Ingots | Processed | Smelted from Ore + Timber |
| Tools | Final | Forged from Ingots + Planks |
| Cloth | Final | Woven from Wool |

Stone, Clay, and Herbs exist in the resource enum but are not yet used.

### Production Chains

```
Farmer ──→ Grain ──→ Miller ──→ Flour (everyone eats)

Lumberjack ──→ Timber ──→ Sawyer ──→ Planks
                  │
                  └──→ Smelter ──→ Iron Ingots ──→ Smith ──→ Tools
                          ↑
Miner ──→ Iron Ore ───────┘

Shepherd ──→ Wool ──→ Weaver ──→ Cloth
```

Primary producers create resources from nothing (representing land/labor). Processors convert raw materials into higher-value goods. Everyone consumes flour, planks, cloth, and tools.

### Agent Behavior

Each tick, agents:
1. **Produce** goods according to their recipe (capped by surplus threshold)
2. **Submit orders** — sell surplus production, buy needed inputs and consumption goods
3. **Trade** on the order book (price-time priority matching)
4. **Consume** flour, tools, planks, and cloth
5. **Evaluate** whether to switch roles based on estimated profitability

Pricing is dynamic: agents adjust buy/sell prices based on inventory levels (urgency when low, discounting when overstocked).

### Economic Mechanisms

- **Spoilage**: Perishables (grain, flour) decay at 5%/tick, raw materials at 2%/tick, processed goods at 1%/tick
- **Taxation**: 3% proportional tax redistributed equally — prevents gold concentration
- **Role switching**: Agents pay a fee (collected by the treasury) and suffer an efficiency penalty when changing roles
- **Congestion**: Roles with more agents than the saturation point see reduced output
- **Subsistence foraging**: Starving agents can forage minimal flour to survive
- **Price memory**: EMA-smoothed prices decay toward the default when no trades occur

## Usage

```bash
# Run with default settings (100 ticks, 500ms delay)
cargo run

# Fast run with no delay
cargo run -- --speed 0

# Run 500 ticks with chart output
cargo run -- --ticks 500 --speed 0 --graph

# Set a specific random seed
cargo run -- --seed 123
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--seed` | 42 | RNG seed for reproducible simulations |
| `--ticks` | 100 | Number of ticks to simulate (0 = unlimited) |
| `--speed` | 500 | Milliseconds between ticks (0 = no delay) |
| `--graph` | off | Generate PNG charts in `output/` after simulation |

### Chart Output

When run with `--graph`, four PNG charts are generated in the `output/` directory:

- **prices.png** — Time series of all resource prices
- **trades.png** — Trade volume (number of trades per tick)
- **roles.png** — Role distribution over time (how many agents in each role)
- **wealth.png** — Wealth percentiles over time + final wealth histogram

## Building

Requires Rust (2021 edition).

```bash
cargo build --release
```

### Dependencies

- [clap](https://crates.io/crates/clap) — CLI argument parsing
- [rand](https://crates.io/crates/rand) — Random number generation
- [strum](https://crates.io/crates/strum) — Enum iteration utilities
- [plotters](https://crates.io/crates/plotters) — Chart generation (used only with `--graph`)

## Project Structure

```
src/
  main.rs        — CLI entry point and simulation loop
  config.rs      — All tunable constants
  resource.rs    — Resource enum and production recipes
  inventory.rs   — Fixed-size resource vector operations
  order.rs       — Order and Trade types
  orderbook.rs   — Price-time priority order matching
  market.rs      — Market state and price tracking (EMA)
  agent.rs       — Agent behavior, production, consumption, order generation
  simulation.rs  — Per-tick simulation logic, role reallocation, taxation
  display.rs     — Terminal output formatting
  charts.rs      — PNG chart generation with plotters
```

## License

Personal project — no license specified.
