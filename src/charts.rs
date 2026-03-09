use std::path::Path;

use plotters::prelude::*;
use strum::IntoEnumIterator;

use crate::resource::Resource;

pub const OUTPUT_DIR: &str = "output";

/// Collected data from each tick for charting.
#[derive(Default)]
pub struct SimHistory {
    pub prices: Vec<[f32; crate::config::RESOURCE_COUNT]>,
    pub trade_counts: Vec<usize>,
    pub role_counts: Vec<[u32; crate::config::RECIPE_COUNT]>,
    pub agent_golds: Vec<Vec<f32>>,
}

impl SimHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_tick(
        &mut self,
        market: &crate::market::Market,
        agents: &[crate::agent::Agent],
        _recipes: &[crate::resource::Recipe],
    ) {
        // Prices
        let mut prices = [0.0f32; crate::config::RESOURCE_COUNT];
        for r in Resource::iter() {
            prices[r as usize] = market.last_price(r);
        }
        self.prices.push(prices);

        // Trade count
        self.trade_counts.push(market.trade_count());

        // Role counts
        let mut counts = [0u32; crate::config::RECIPE_COUNT];
        for agent in agents {
            if let Some(ri) = agent.role_index {
                if ri < crate::config::RECIPE_COUNT {
                    counts[ri] += 1;
                }
            }
        }
        self.role_counts.push(counts);

        // Agent gold values (sorted for distribution)
        let mut golds: Vec<f32> = agents.iter().map(|a| a.gold).collect();
        golds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.agent_golds.push(golds);
    }

    pub fn generate_charts(&self, recipes: &[crate::resource::Recipe]) {
        if self.prices.is_empty() {
            return;
        }

        std::fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");

        self.draw_price_chart().unwrap_or_else(|e| {
            eprintln!("Error drawing price chart: {}", e);
        });
        self.draw_trade_volume_chart().unwrap_or_else(|e| {
            eprintln!("Error drawing trade volume chart: {}", e);
        });
        self.draw_role_chart(recipes).unwrap_or_else(|e| {
            eprintln!("Error drawing role chart: {}", e);
        });
        self.draw_wealth_chart().unwrap_or_else(|e| {
            eprintln!("Error drawing wealth chart: {}", e);
        });
    }

    /// Print health metrics for the last N ticks of the simulation.
    pub fn print_health_summary(&self, tail_ticks: usize) {
        let ticks = self.prices.len();
        if ticks == 0 {
            println!("No data collected.");
            return;
        }

        let start = ticks.saturating_sub(tail_ticks);
        let window = &self.trade_counts[start..];
        let price_window = &self.prices[start..];
        let role_window = &self.role_counts[start..];
        let gold_window = &self.agent_golds[start..];

        // Trade volume
        let avg_trades: f32 = window.iter().map(|&c| c as f32).sum::<f32>() / window.len() as f32;
        let min_trades = *window.iter().min().unwrap_or(&0);

        // Price health: average price per resource, count of resources stuck at MIN_PRICE
        let num_resources = crate::config::RESOURCE_COUNT;
        let mut avg_prices = vec![0.0f32; num_resources];
        let mut min_prices = vec![f32::MAX; num_resources];
        for p in price_window {
            for (i, &val) in p.iter().enumerate() {
                avg_prices[i] += val;
                min_prices[i] = min_prices[i].min(val);
            }
        }
        for p in &mut avg_prices {
            *p /= price_window.len() as f32;
        }

        let floor_price = crate::config::MIN_PRICE * 1.1;
        let stuck_at_floor: Vec<&str> = Resource::iter()
            .filter(|r| avg_prices[*r as usize] < floor_price)
            .map(|r| r.short_name())
            .collect();

        // Role health: any roles with 0 agents?
        let last_roles = role_window.last().unwrap();
        let empty_roles: usize = last_roles.iter().filter(|&&c| c == 0).count();

        // Wealth distribution (Gini coefficient from last tick)
        let last_golds = gold_window.last().unwrap();
        let gini = compute_gini(last_golds);

        // Average efficiency from last tick gold values
        // (we don't track efficiency directly, but we can infer from gold spread)
        let total_gold: f32 = last_golds.iter().sum();
        let median_gold = last_golds[last_golds.len() / 2];
        let max_gold = *last_golds.last().unwrap_or(&0.0);
        let min_gold = *last_golds.first().unwrap_or(&0.0);

        // Overall health score (0-100)
        let trade_score = (avg_trades / 50.0).min(1.0) * 25.0; // 25 pts for >=50 trades/tick
        let floor_score = (1.0 - stuck_at_floor.len() as f32 / num_resources as f32) * 25.0;
        let role_score = (1.0 - empty_roles as f32 / crate::config::RECIPE_COUNT as f32) * 25.0;
        let gini_score = (1.0 - gini).max(0.0) * 25.0; // 25 pts for perfect equality
        let health = trade_score + floor_score + role_score + gini_score;

        println!("=== Health Summary (last {} ticks) ===", window.len());
        println!(
            "  Avg trades/tick:  {:.1} (min: {})",
            avg_trades, min_trades
        );
        println!("  Total gold:       {:.0}", total_gold);
        println!(
            "  Gold range:       {:.1} / {:.1} / {:.1} (min/median/max)",
            min_gold, median_gold, max_gold
        );
        println!("  Gini coefficient: {:.3}", gini);
        println!("  Empty roles:      {}", empty_roles);
        if stuck_at_floor.is_empty() {
            println!("  At price floor:   none");
        } else {
            println!("  At price floor:   {}", stuck_at_floor.join(", "));
        }
        println!();

        // Per-resource average prices
        print!("  Avg prices:      ");
        for r in Resource::iter() {
            let p = avg_prices[r as usize];
            print!(" {}:{:.1}", r.short_name().trim(), p);
        }
        println!();

        // Role distribution
        print!("  Roles:           ");
        let role_names = [
            "Farm", "Lumb", "Mine", "Mill", "Sawy", "Smlt", "Smth", "Shep", "Weav",
        ];
        for (i, name) in role_names.iter().enumerate() {
            if i < last_roles.len() {
                print!(" {}:{}", name, last_roles[i]);
            }
        }
        println!();

        println!();
        println!("  HEALTH SCORE:     {:.0}/100", health);
    }

    fn draw_price_chart(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ticks = self.prices.len();
        let max_price = self
            .prices
            .iter()
            .flat_map(|p| p.iter())
            .cloned()
            .fold(0.0f32, f32::max)
            .min(100.0); // Cap for readability

        let path = Path::new(OUTPUT_DIR).join("prices.png");
        let root = BitMapBackend::new(path.to_str().unwrap(), (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Resource Prices Over Time", ("sans-serif", 24))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..ticks, 0.0f32..max_price * 1.1)?;

        chart
            .configure_mesh()
            .x_desc("Tick")
            .y_desc("Price")
            .draw()?;

        let colors = [
            &RED,
            &BLUE,
            &GREEN,
            &MAGENTA,
            &CYAN,
            &BLACK,
            &RGBColor(255, 165, 0),  // orange
            &RGBColor(128, 0, 128),  // purple
            &RGBColor(0, 128, 128),  // teal
            &RGBColor(139, 69, 19),  // brown
            &RGBColor(70, 130, 180), // steel blue
            &RGBColor(220, 20, 60),  // crimson
        ];

        for (ri, r) in Resource::iter().enumerate() {
            let data: Vec<(usize, f32)> = (0..ticks)
                .map(|t| (t, self.prices[t][ri].min(max_price)))
                .collect();

            let color = colors[ri % colors.len()];
            chart
                .draw_series(LineSeries::new(data, color.stroke_width(2)))?
                .label(format!("{:?}", r))
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
                });
        }

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        println!("Saved: {}/prices.png", OUTPUT_DIR);
        Ok(())
    }

    fn draw_trade_volume_chart(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ticks = self.trade_counts.len();
        let max_trades = *self.trade_counts.iter().max().unwrap_or(&1) as f32;

        let path = Path::new(OUTPUT_DIR).join("trades.png");
        let root = BitMapBackend::new(path.to_str().unwrap(), (1200, 400)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Trade Volume Over Time", ("sans-serif", 24))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..ticks, 0.0f32..max_trades * 1.1)?;

        chart
            .configure_mesh()
            .x_desc("Tick")
            .y_desc("Trades")
            .draw()?;

        let data: Vec<(usize, f32)> = self
            .trade_counts
            .iter()
            .enumerate()
            .map(|(t, &c)| (t, c as f32))
            .collect();

        chart.draw_series(LineSeries::new(data, BLUE.stroke_width(2)))?;

        root.present()?;
        println!("Saved: {}/trades.png", OUTPUT_DIR);
        Ok(())
    }

    fn draw_role_chart(
        &self,
        recipes: &[crate::resource::Recipe],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticks = self.role_counts.len();
        let max_count = self
            .role_counts
            .iter()
            .flat_map(|c| c.iter())
            .cloned()
            .max()
            .unwrap_or(1) as f32;

        let path = Path::new(OUTPUT_DIR).join("roles.png");
        let root = BitMapBackend::new(path.to_str().unwrap(), (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Role Distribution Over Time", ("sans-serif", 24))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..ticks, 0.0f32..max_count + 1.0)?;

        chart
            .configure_mesh()
            .x_desc("Tick")
            .y_desc("Agents")
            .draw()?;

        let colors = [
            &RED,
            &BLUE,
            &GREEN,
            &MAGENTA,
            &CYAN,
            &BLACK,
            &RGBColor(255, 165, 0),
            &RGBColor(128, 0, 128),
            &RGBColor(0, 128, 128),
        ];

        let role_names: Vec<&str> = recipes
            .iter()
            .map(|r| match r.output {
                Resource::Grain => "Farmer",
                Resource::Timber => "Lumberjack",
                Resource::IronOre => "Miner",
                Resource::Flour => "Miller",
                Resource::Planks => "Sawyer",
                Resource::IronIngots => "Smelter",
                Resource::Tools => "Smith",
                Resource::Wool => "Shepherd",
                Resource::Cloth => "Weaver",
                _ => "Unknown",
            })
            .collect();

        for ri in 0..crate::config::RECIPE_COUNT {
            let data: Vec<(usize, f32)> = (0..ticks)
                .map(|t| (t, self.role_counts[t][ri] as f32))
                .collect();

            let color = colors[ri % colors.len()];
            let name = role_names.get(ri).copied().unwrap_or("Unknown").to_string();
            chart
                .draw_series(LineSeries::new(data, color.stroke_width(2)))?
                .label(name)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
                });
        }

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        println!("Saved: {}/roles.png", OUTPUT_DIR);
        Ok(())
    }

    fn draw_wealth_chart(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ticks = self.agent_golds.len();
        if ticks == 0 {
            return Ok(());
        }

        let max_gold = self
            .agent_golds
            .iter()
            .flat_map(|g| g.iter())
            .cloned()
            .fold(0.0f32, f32::max);

        let path = Path::new(OUTPUT_DIR).join("wealth.png");
        let root = BitMapBackend::new(path.to_str().unwrap(), (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        // Split into two charts: wealth over time (top) and final distribution (bottom)
        let (upper, lower) = root.split_vertically(350);

        // Upper: wealth bands over time (min, 25th, median, 75th, max)
        {
            let mut chart = ChartBuilder::on(&upper)
                .caption("Wealth Distribution Over Time", ("sans-serif", 24))
                .margin(10)
                .x_label_area_size(30)
                .y_label_area_size(60)
                .build_cartesian_2d(0..ticks, 0.0f32..max_gold * 1.1)?;

            chart
                .configure_mesh()
                .x_desc("Tick")
                .y_desc("Gold")
                .draw()?;

            let percentiles = [
                ("Min", 0.0, &RED),
                ("25th", 0.25, &BLUE),
                ("Median", 0.5, &GREEN),
                ("75th", 0.75, &MAGENTA),
                ("Max", 1.0, &BLACK),
            ];

            for &(name, pct, color) in &percentiles {
                let data: Vec<(usize, f32)> = (0..ticks)
                    .map(|t| {
                        let golds = &self.agent_golds[t];
                        let idx = ((golds.len() as f32 - 1.0) * pct) as usize;
                        (t, golds[idx.min(golds.len() - 1)])
                    })
                    .collect();

                chart
                    .draw_series(LineSeries::new(data, color.stroke_width(2)))?
                    .label(name)
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
                    });
            }

            chart
                .configure_series_labels()
                .position(SeriesLabelPosition::UpperLeft)
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()?;
        }

        // Lower: final wealth histogram
        {
            let final_golds = self.agent_golds.last().unwrap();
            let bucket_size = (max_gold / 20.0).max(1.0);
            let num_buckets = (max_gold / bucket_size).ceil() as usize + 1;
            let mut buckets = vec![0u32; num_buckets];
            for &g in final_golds {
                let idx = (g / bucket_size) as usize;
                if idx < buckets.len() {
                    buckets[idx] += 1;
                }
            }
            let max_count = *buckets.iter().max().unwrap_or(&1);

            let mut chart = ChartBuilder::on(&lower)
                .caption("Final Wealth Distribution", ("sans-serif", 20))
                .margin(10)
                .x_label_area_size(30)
                .y_label_area_size(60)
                .build_cartesian_2d((0..num_buckets).into_segmented(), 0u32..max_count + 1)?;

            chart
                .configure_mesh()
                .x_desc("Gold (buckets)")
                .y_desc("Agents")
                .x_label_formatter(&|x| {
                    if let SegmentValue::CenterOf(v) = x {
                        format!("{:.0}", *v as f32 * bucket_size)
                    } else {
                        String::new()
                    }
                })
                .draw()?;

            chart.draw_series(
                Histogram::vertical(&chart)
                    .style(BLUE.mix(0.7).filled())
                    .data(buckets.iter().enumerate().map(|(i, &c)| (i, c))),
            )?;
        }

        root.present()?;
        println!("Saved: {}/wealth.png", OUTPUT_DIR);
        Ok(())
    }
}

/// Compute the Gini coefficient from a sorted list of values.
fn compute_gini(sorted_values: &[f32]) -> f32 {
    let n = sorted_values.len() as f32;
    if n == 0.0 {
        return 0.0;
    }
    let total: f32 = sorted_values.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut numerator = 0.0f32;
    for (i, &val) in sorted_values.iter().enumerate() {
        numerator += (2.0 * (i as f32 + 1.0) - n - 1.0) * val;
    }
    numerator / (n * total)
}
