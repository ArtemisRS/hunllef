*Inspired by https://github.com/Jawnny/hunllefSim*

## Corrupted Hunllef Simulator

Simulator for the Corrupted Hunllef fight in OSRS. The goal is  to predict food
needed and time taken for a perfect executed fight.

Can set the following variables from the CLI
- `-t / --trials` - number of simulations to complete
- `-f / --fish` - number to eat (heal 20 hp)
- `-e / --eat_at_hp` - hp threshold to eat fish
- `-a / --armour` - tier of CG armour
- `-l / --level` - Ranged/Magic/Def/HP level (same for all)

Will output percent of successful trials (defined as Hunllef dead and player
alive) and average (successful) completion time.


## How to run

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. clone this repo
3. `cargo run` 
4. `cargo run -- --help` for options

## Future features
- tick eating
- graphs (success rate by fish/lvl)
- logging (annotated prints of individual kills)
- use different prayers (ee, mystic might)
- set levels for each stat
- melee (halberd)
