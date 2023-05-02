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
- Using either Rigour or Eagle Eye (with Steel Skin) with Ranged
- Using either Augury or Mystic Might (with Steel Skin) with Magic
- Tornado spawn frequency
- The time cost for healing
- The ability to tick eat attacks from Hunllef

Limitations:
- Does not account for player natural HP regeneration
- Does not allow for melee simulation
- Does not allow for redemption healing
- Does not account for hit delay (time between attacking and hit being
  registered)
- **Assumes perfect play on behalf of the player (no off prayer attacks, no lost
  ticks, no stomps, no damage from tornadoes**


The following variables can be set via the CLI:
```
Options:
  -t, --trials <TRIALS>                Number of simulations to complete [default: 100000]
  -f, --fish <FISH>                    Number to eat (heal 20 hp) [default: 12]
  -a, --armour <ARMOUR>                Tier of CG armour [default: 1]
      --defence <DEFENCE>              Defence Level to use [default: 99]
      --ranged <RANGED>                Ranged Level to use [default: 99]
      --magic <MAGIC>                  Magic Level to use [default: 99]
      --hp <HP>                        HP Level to use [default: 99]
      --ranged-prayer <RANGED_PRAYER>  Set the Ranged prayer [default: rigour]
                                         [possible values: rigour, eagle-eye]
      --magic-prayer <MAGIC_PRAYER>    Set the Magic prayer [default: augury]
                                         [possible values: augury, mystic-might]
  -e, --eat-at-hp <EAT_AT_HP>          HP threshold to eat fish [default: 50]
      --tick-eat                       Simulate tick eating when hp is below Hunllef max
      --histogram                      Histogram values for times/fish_eaten
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

### Sample run at the point a player might be starting CG

```
$ time cargo run --release -- -t 1000000 --histogram --fish 25 --ranged 92 --magic 92 --hp 85 --defence 75 --ranged-prayer eagle-eye --magic-prayer mystic-might
    Finished release [optimized] target(s) in 0.16s
     Running `target/release/hunllef -t 1000000 --histogram --fish 25 --ranged 92 --magic 92 --hp 85 --defence 75 --ranged-prayer eagle-eye --magic-prayer mystic-might`
success rate: 99.99%
avg fish eaten: 14.8
avg time: 376.6 ticks
Histograms
Time (m:ss) - 999926 samples
  .5'th %: 2:38
 2.5'th %: 2:51
16.7'th %: 3:16
50.0'th %: 3:44
83.0'th %: 4:15
97.5'th %: 4:48
99.5'th %: 5:10
Fish eaten - 1000000 samples (includes failures)
  .5'th %: 8
 2.5'th %: 9
16.7'th %: 12
50.0'th %: 15
83.0'th %: 18
97.5'th %: 21
99.5'th %: 24

real    0m2.034s
user    0m1.699s
sys     0m0.030s
```

## Future features (in rough order of implementing)
- melee (halberd)
- graphs (success rate by fish/lvl)
- logging (annotated prints of individual kills)
- redemption healing
- 5:1
