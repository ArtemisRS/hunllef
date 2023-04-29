*Inspired by https://github.com/Jawnny/hunllefSim*

## Corrupted Hunllef Simulator

Simulator for the Corrupted Hunllef fight in OSRS. The goal is to predict food
needed and time taken for a perfect executed fight. There are a number of
options available for setting simulation conditions including trials, combat
stats, armour tier, and eating strategies.

Features:
- Randomized player starting style (Ranged or Magic)
- Player accuracy and defence rolls taking into account levels, prayer, weapon,
  and armour
- Hunllef accuracy and defence rolls taking into account its stats
- Tornado spawn frequency
- The time cost for healing
- The ability to tick eat attacks from Hunllef

Limitations:
- Does not account for player natural HP regeneration
- Does not allow for melee simulation
- Does not allow for prayers other than Rigour/Augury
- Does not allow for redemption healing
- Does not account for hit delay (time between attacking and hit being
  registered)
- **Assumes perfect play on behalf of the player (no off prayer attacks, no lost
  ticks, no stomps, no damage from tornadoes**


The following variables can be set via the CLI:
```
Options:
  -t, --trials <TRIALS>        Number of simulations to complete [default: 100000]
  -f, --fish <FISH>            Number to eat (heal 20 hp) [default: 12]
  -a, --armour <ARMOUR>        Tier of CG armour [default: 1]
      --defence <DEFENCE>      Level to use [default: 99]
      --ranged <RANGED>        Level to use [default: 99]
      --magic <MAGIC>          Level to use [default: 99]
      --hp <HP>                Level to use [default: 99]
  -e, --eat-at-hp <EAT_AT_HP>  HP threshold to eat fish [default: 50]
      --histogram              Histogram values for times/fish_eaten
      --tick-eat               Simulate tick eating when hp is below Hunllef max
```

Will output percent of successful trials (defined as Hunllef dead and player
alive) and average (successful) completion time.


## How to run

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. clone this repo
3. `cargo run` 
4. `cargo run -- --help` for options

*Note: Recommend running release (not debug) for large number of trials (>10k)*
```rust
cargo run --release -- --trials 1000000
```

## Future features (in rough order of implementing)
- use different prayers (ee, mystic might)
- melee (halberd)
- graphs (success rate by fish/lvl)
- logging (annotated prints of individual kills)
- redemption healing
