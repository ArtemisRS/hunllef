*Inspired by https://github.com/Jawnny/hunllefSim*

## Corrupted Hunllef Simulator

Simulator for the Corrupted Hunllef fight in OSRS. The goal is  to predict food
needed and time taken for a perfect executed fight.

Can set the following variables from the CLI
```
Options:
  -t, --trials <TRIALS>        Number of simulations to complete [default: 100000]
  -f, --fish <FISH>            Number of eat (heal 20 hp) [default: 12]
  -a, --armour <ARMOUR>        Tier of CG armour [default: 1]
  -l, --level <LEVEL>          Ranged/Magic/Def/HP level to use (same for all) [default: 99]
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

## Future features
- use different prayers (ee, mystic might)
- graphs (success rate by fish/lvl)
- logging (annotated prints of individual kills)
- set levels for each stat
- melee (halberd)
