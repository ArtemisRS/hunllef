use clap::Parser;
use hdrhistogram::Histogram;
use hunllef::{Hunllef, Levels, Player, Prayer, Setup, Weapon};

#[derive(Parser, Debug)]
#[command(name = "Hunllef")]
#[command(version = "0.1")]
#[command(about = "Simulates the Corrupted Hunllef fight", long_about = None)]
struct Cli {
    /// Number of simulations
    #[arg(short, long, default_value_t = 100_000)]
    trials: u32,

    /// Number to eat (heal 20 hp)
    #[arg(short, long, default_value_t = 12)]
    fish: u8,

    /// Tier of CG armour
    #[arg(short, long, default_value_t = 1)]
    armour: u8,

    ///1st setup weapon
    #[arg(long, value_enum, default_value_t = Weapon::Bow)]
    setup1: Weapon,

    ///2nd setup weapon
    #[arg(long, value_enum, default_value_t = Weapon::Staff)]
    setup2: Weapon,

    ///1st setup prayer
    #[arg(long, value_enum, default_value_t = Prayer::Rigour)]
    setup1_prayer: Prayer,

    ///2nd setup prayer
    #[arg(long, value_enum, default_value_t = Prayer::Augury)]
    setup2_prayer: Prayer,

    /// Player Attack Level
    #[arg(long, default_value_t = 99)]
    attack: u8,

    /// Player Strength Level
    #[arg(long, default_value_t = 99)]
    strength: u8,

    /// Player Defence Level
    #[arg(long, default_value_t = 99)]
    defence: u8,

    /// Player Ranged Level
    #[arg(long, default_value_t = 99)]
    ranged: u8,

    /// Player Magic Level
    #[arg(long, default_value_t = 99)]
    magic: u8,

    /// Player HP Level
    #[arg(long, default_value_t = 99)]
    hp: u8,

    /// HP threshold to eat fish
    #[arg(short, long, default_value_t = 50)]
    eat_at_hp: u16,

    ///Simulate tick eating when hp is below Hunllef max
    #[arg(long, default_value_t = false)]
    tick_eat: bool,

    ///Account for ticks lost by player
    #[arg(long, default_value_t = 0)]
    lost_ticks: u8,

    /// Max time for successful run (in ticks)
    #[arg(long, default_value_t = 6000)]
    max_time: u16,

    /// Histogram values for times/fish_eaten
    #[arg(long, default_value_t = false)]
    histogram: bool,
}
fn generate_histogram(times: &[u16], fish_eaten: &[u64]) {
    fn tick_to_secs(ticks: u64) -> String {
        let min = ticks / 100;
        let sec = ticks * 3 / 5 % 60;
        format!("{}:{:#02}", min, sec)
    }

    let mut hist = Histogram::<u64>::new(3).unwrap();
    for num in times {
        hist.record(*num as u64).unwrap();
    }

    println!("\nHistograms");
    println!("Time (m:ss) - {} samples", hist.len());
    println!("  .5'th %: {}", tick_to_secs(hist.value_at_quantile(0.005)));
    println!(" 2.5'th %: {}", tick_to_secs(hist.value_at_quantile(0.025)));
    println!("16.7'th %: {}", tick_to_secs(hist.value_at_quantile(0.167)));
    println!("50.0'th %: {}", tick_to_secs(hist.value_at_quantile(0.50)));
    println!("83.0'th %: {}", tick_to_secs(hist.value_at_quantile(0.83)));
    println!("97.5'th %: {}", tick_to_secs(hist.value_at_quantile(0.975)));
    println!("99.5'th %: {}", tick_to_secs(hist.value_at_quantile(0.995)));

    let mut hist = Histogram::<u64>::new(3).unwrap();
    for num in fish_eaten {
        hist.record(*num).unwrap();
    }

    println!("\nFish eaten - {} samples (includes failures)", hist.len());
    println!("  .5'th %: {}", hist.value_at_quantile(0.005));
    println!(" 2.5'th %: {}", hist.value_at_quantile(0.025));
    println!("16.7'th %: {}", hist.value_at_quantile(0.167));
    println!("50.0'th %: {}", hist.value_at_quantile(0.50));
    println!("83.0'th %: {}", hist.value_at_quantile(0.83));
    println!("97.5'th %: {}", hist.value_at_quantile(0.975));
    println!("99.5'th %: {}", hist.value_at_quantile(0.995));
}

fn main() {
    let args = Cli::parse();

    let levels = Levels {
        attack: args.attack,
        strength: args.strength,
        defence: args.defence,
        ranged: args.ranged,
        magic: args.magic,
        prayer: 99,
        hp: args.hp,
    };

    let setup1 = Setup::new(args.setup1, args.setup1_prayer, &levels, args.armour);
    let setup2 = Setup::new(args.setup2, args.setup2_prayer, &levels, args.armour);

    let player = Player::new(
        &setup1,
        &setup2,
        levels.hp as u16,
        args.fish,
        args.lost_ticks,
    );

    let hunllef = Hunllef::new(args.armour);

    let (success, fish_eaten, times) = hunllef::run_simulation(
        args.trials,
        &player,
        &hunllef,
        args.eat_at_hp,
        args.tick_eat,
        args.max_time,
    );

    let success_rate = (success as f32 * 100.0) / (args.trials as f32);
    let avg_fish: f64 = fish_eaten.iter().sum::<u64>() as f64 / fish_eaten.len() as f64;
    let avg_time = times.iter().map(|t| *t as u64).sum::<u64>() as f64 / times.len() as f64;
    println!("success rate: {:.2}%", success_rate);
    println!("avg fish eaten: {:.1}", avg_fish);
    println!("avg time: {:.1} ticks", avg_time);

    if args.histogram {
        generate_histogram(&times, &fish_eaten);
    }
}
