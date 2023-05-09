*Inspired by https://github.com/Jawnny/hunllefSim*

## Corrupted Hunllef Simulator

Simulator for the Corrupted Hunllef fight in OSRS. The goal is to predict food
needed and time taken for a perfectly executed fight. There are a number of
options available for setting simulation conditions including number of trials,
combat stats, combat styles, armour/weapon tiers, and eating strategies.

Features:
- All 3 player combat styles supported (Melee, Ranged, and Magic)
- Player accuracy and defence rolls take into account levels, prayer,
  weapon/tier, and armour
- Hunllef accuracy and defence rolls take into account its stats
- Supports Piety/Rigour/Augury as well as Eagle Eye and Mystic Might (which are
  coupled with Steel Skin)
- Accounts for tornado spawn frequency, time spent healing, and an estimate of
  other lost ticks
- Has the ability to tick eat attacks from Hunllef
- Can set a maximum time for a run to take to be counted as a success
- Histogram produced for food used and kill times

Limitations:
- Does not account for player natural HP regeneration
- Does not allow for redemption healing
- Does not account for hit delay (time between attacking and hit being
  registered)
- **Assumes perfect play on behalf of the player (no off prayer attacks, no
  stomps, no damage from tornadoes**


The following variables can be set via the CLI:
```
Usage: hunllef [OPTIONS]

Options:
  -t, --trials <TRIALS>                Number of simulations [default: 100000]
  -f, --fish <FISH>                    Number to eat (heal 20 hp) [default: 12]
  -a, --armour <ARMOUR>                Tier of CG armour [default: 1]
      --setup1 <SETUP1>                1st setup weapon [default: bow]
                                         [possible values: bow, staff, halberd]
      --setup2 <SETUP2>                2nd setup weapon [default: staff]
                                         [possible values: bow, staff, halberd]
      --setup1-tier <SETUP1_TIER>      1st weapon tier (1, 2, 3) [default: 3]
      --setup2-tier <SETUP2_TIER>      2nd weapon tier (1, 2, 3) [default: 3]
      --setup1-prayer <SETUP1_PRAYER>  1st setup prayer [default: rigour]
                                         [possible values: rigour, augury, piety, eagle-eye, mystic-might]
      --setup2-prayer <SETUP2_PRAYER>  2nd setup prayer [default: augury]
                                         [possible values: rigour, augury, piety, eagle-eye, mystic-might]
      --attack <ATTACK>                Player Attack Level [default: 99]
      --strength <STRENGTH>            Player Strength Level [default: 99]
      --defence <DEFENCE>              Player Defence Level [default: 99]
      --ranged <RANGED>                Player Ranged Level [default: 99]
      --magic <MAGIC>                  Player Magic Level [default: 99]
      --hp <HP>                        Player HP Level [default: 99]
  -e, --eat-at-hp <EAT_AT_HP>          HP threshold to eat fish [default: 50]
      --tick-eat                       Simulate tick eating when hp is below Hunllef max
      --lost-ticks <LOST_TICKS>        Account for ticks lost by player [default: 0]
      --max-time <MAX_TIME>            Max time for successful run (in ticks) [default: 6000]
      --histogram                      Histogram values for times/fish_eaten
  -h, --help                           Print help
  -V, --version                        Print version

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
$ time cargo run --release -- -t 1000000 --histogram --attack 78 --strength 85 --defence 75 --ranged 92 --magic 92 --hp 85 --fish 20 --setup1 bow --setup1-prayer eagle-eye --setup2 staff --setup2-prayer mystic-might
    Finished release [optimized] target(s) in 0.07s
     Running `target/release/hunllef -t 1000000 --histogram --attack 78 --strength 85 --defence 75 --ranged 92 --magic 92 --hp 85 --fish 20 --setup1 bow --setup1-prayer eagle-eye --setup2 staff --setup2-prayer mystic-might`
success rate: 99.43%
avg fish eaten: 14.7
avg time: 375.9 ticks

Histograms
Time (m:ss) - 994324 samples
  .5'th %: 2:38
 2.5'th %: 2:51
16.7'th %: 3:16
50.0'th %: 3:44
83.0'th %: 4:14
97.5'th %: 4:46
99.5'th %: 5:04

Fish eaten - 1000000 samples (includes failures)
  .5'th %: 8
 2.5'th %: 9
16.7'th %: 12
50.0'th %: 15
83.0'th %: 18
97.5'th %: 20
99.5'th %: 20

real	0m1.988s
user	0m1.725s
sys	0m0.022s
```

## Future features (in rough order of implementing)
- logging (annotated prints of individual kills)
- 5:1
- redemption healing
- graphs (success rate by fish/lvl)
