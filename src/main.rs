use clap::{Parser, ValueEnum};
use fastrand::Rng;
use hdrhistogram::Histogram;

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

    /// Histogram values for times/fish_eaten
    #[arg(long, default_value_t = false)]
    histogram: bool,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Weapon {
    Bow,
    Staff,
    Halberd,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Prayer {
    Rigour,
    Augury,
    Piety,
    EagleEye,
    MysticMight,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
struct Levels {
    attack: u8,
    strength: u8,
    defence: u8,
    ranged: u8,
    magic: u8,
    prayer: u8,
    hp: u8,
}

#[derive(Debug, Clone, Copy)]
struct Setup {
    weapon: Weapon,
    attack_delay: u8,
    max_hit: u16,
    acc_roll: u16,
    rdr: u16, //ranged defensive roll
    mdr: u16, //magic defensive roll
}

impl Setup {
    fn new(weapon: Weapon, prayer: Prayer, levels: &Levels, armour_tier: u8) -> Setup {
        fn effective_level(level: u8, prayer_bonus: u8, weapon: Option<Weapon>) -> u16 {
            let extra_bonus = match weapon {
                Some(Weapon::Bow) => 8,
                Some(Weapon::Staff) => 11,
                Some(Weapon::Halberd) => 11,
                None => 8,
            };
            (level as u16) * (100 + prayer_bonus as u16) / 100 + extra_bonus
        }

        let (armour_acc, armour_def) = match armour_tier {
            1 => (16, 166),
            2 => (28, 224),
            3 => (40, 284),
            _ => (0, 0),
        };
        let (weapon_acc, eq_str) = match weapon {
            Weapon::Bow => (172, 138),
            Weapon::Staff => (184, 0),
            Weapon::Halberd => (168, 138),
        };
        let eq_acc = armour_acc + weapon_acc;
        let (prayer_acc, prayer_str, prayer_def, prayer_def_magic) = match prayer {
            Prayer::Rigour => (20, 23, 25, 0),
            Prayer::Augury => (25, 0, 25, 25),
            Prayer::Piety => (20, 23, 25, 0),
            Prayer::EagleEye => (15, 15, 15, 0),
            Prayer::MysticMight => (15, 0, 15, 15),
        };

        let (acc_lvl, dam_lvl) = match weapon {
            Weapon::Bow => (levels.ranged, levels.ranged),
            Weapon::Staff => (levels.magic, levels.magic),
            Weapon::Halberd => (levels.attack, levels.strength),
        };

        let eff_acc_lvl = effective_level(acc_lvl, prayer_acc, Some(weapon));
        let acc_roll = eff_acc_lvl * (eq_acc + 64);

        let eff_str_lvl = effective_level(dam_lvl, prayer_str, Some(weapon));
        let max_hit = match weapon {
            Weapon::Bow | Weapon::Halberd => (eff_str_lvl * (eq_str + 64) + 320) / 640,
            Weapon::Staff => 39,
        };

        let eff_def_lvl = effective_level(levels.defence, prayer_def, None);
        let rdr = eff_def_lvl * (armour_def + 64);

        let magic_def_wep = if weapon == Weapon::Staff {
            Some(Weapon::Staff)
        } else {
            None
        };
        let eff_magic_lvl = effective_level(levels.magic, prayer_def_magic, magic_def_wep);

        let eff_magic_def_lvl = eff_def_lvl * 3 / 10 + eff_magic_lvl * 7 / 10;
        let mdr = eff_magic_def_lvl * (armour_def + 64);

        Setup {
            weapon,
            attack_delay: 4,
            max_hit,
            acc_roll,
            rdr,
            mdr,
        }
    }

    fn attack(self, rng: &Rng, hunllef_defensive_roll: u16) -> u16 {
        //ranges are not inclusive of top, but the rolls need to be
        if rng.u16(0..self.acc_roll + 1) > rng.u16(0..hunllef_defensive_roll + 1) {
            rng.u16(0..self.max_hit + 1)
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AttackStyle {
    Ranged,
    Magic,
}

#[derive(Debug, Clone, Copy)]
struct Hunllef {
    hp: u16,
    max_hit: u16,
    attack_delay: u8,    //ticks
    acc_roll: u16,       //same for ranged and magic
    defensive_roll: u16, //same for all styles
    tornado_cd: u8,      //number of attacks until a tornado attack
    attack_cd: u8,       //ticks until next attack
    style: AttackStyle,
    attacks_left: u8, //before switching styles
}

impl Hunllef {
    fn new(armour_tier: u8) -> Hunllef {
        let max_hit = match armour_tier {
            1 => 13,
            2 => 10,
            3 => 8,
            _ => 0,
        };
        let attack_delay = 5;
        let acc_roll = (240 + 9) * (90 + 64);
        let defensive_roll = (240 + 9) * (20 + 64);
        let hp = 1000;
        let tornado_cd = 12;
        let attack_cd = 0;
        let style = AttackStyle::Ranged;
        let attacks_left = 4;

        Hunllef {
            max_hit,
            attack_delay,
            acc_roll,
            defensive_roll,
            hp,
            tornado_cd,
            attack_cd,
            style,
            attacks_left,
        }
    }

    fn switch_style(&mut self) {
        if let AttackStyle::Ranged = self.style {
            self.style = AttackStyle::Magic;
        } else {
            self.style = AttackStyle::Ranged;
        }
    }

    fn attack(&mut self, rng: &Rng, player_rdr: u16, player_mdr: u16) -> Option<u16> {
        if self.attack_cd == 0 {
            //Hunllef switches between ranged/magic after every 4 attacks. This
            //includes attacks replaced by a tornado.
            if self.attacks_left == 0 {
                self.switch_style();
                self.attacks_left = 4;
            }

            self.attacks_left -= 1;
            self.attack_cd += self.attack_delay - 1;

            //This is close, but not precisely the same as how tornadoes are
            //actually spawned. The true mechanism is not yet known.
            if self.tornado_cd == 0 {
                //println!("  tornado!");
                self.tornado_cd = rng.u8(10..15);
                return None;
            } else {
                self.tornado_cd -= 1;
            }

            let pdr = if let AttackStyle::Ranged = self.style {
                player_rdr
            } else {
                player_mdr
            };

            //ranges are not inclusive of top, but the rolls need to be
            if rng.u16(0..self.acc_roll) > rng.u16(0..pdr + 1) {
                Some(rng.u16(0..self.max_hit + 1))
            } else {
                Some(0)
            }
        } else {
            self.attack_cd -= 1;
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Player<'a> {
    setup1: &'a Setup,
    setup2: &'a Setup,
    hp: u16,
    fish: u8,
    attack_cd: u8,    //ticks
    attacks_left: u8, //before switching styles
    current: &'a Setup,
}

impl<'a> Player<'a> {
    fn new<'s>(
        setup1: &'s Setup,
        setup2: &'s Setup,
        hp: u16,
        fish: u8,
        lost_ticks: u8,
    ) -> Player<'s> {
        let attack_cd = lost_ticks;
        let attacks_left = 6;
        Player {
            setup1,
            setup2,
            hp,
            fish,
            attack_cd,
            attacks_left,
            current: if fastrand::bool() { setup1 } else { setup2 },
        }
    }

    fn switch_setup(mut self) {
        if self.current.weapon == self.setup1.weapon {
            self.current = self.setup2;
        } else {
            self.current = self.setup1;
        }
    }

    fn attack(&mut self, rng: &Rng, hunllef_defensive_roll: u16) -> Option<u16> {
        if self.attack_cd == 0 {
            //dbg!(self);
            if self.attacks_left == 0 {
                self.switch_setup();
                self.attacks_left = 6;
            }
            let setup = self.current;

            self.attack_cd += setup.attack_delay - 1; //first tick of delay is
                                                      //the attack
            self.attacks_left -= 1;
            Some(setup.attack(rng, hunllef_defensive_roll))
        } else {
            self.attack_cd -= 1;
            None
        }
    }

    fn eat_fish(&mut self) {
        if self.fish > 0 {
            self.fish -= 1;
            self.attack_cd += 3;
            self.hp += 20;
            //println!("  eating a fish to take us to {}", self.hp);
        }
    }
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

    let mut times = Vec::new();
    let mut fish_rem = Vec::new();
    let mut success = 0;
    let rng = fastrand::Rng::new();

    // dbg!(Player::new(&setup1, &setup2, levels.hp as u16, args.fish));

    for _ in 0..args.trials {
        //println!("loop {n}");
        let mut player = Player::new(
            &setup1,
            &setup2,
            levels.hp as u16,
            args.fish,
            args.lost_ticks,
        );
        let mut hunllef = Hunllef::new(args.armour);
        let mut time: u16 = 0; //elapsed time for this trial

        while hunllef.hp > 0 && player.hp > 0 {
            //println!("t={:0>3}, php: {}, hhp: {}", time, player.hp, hunllef.hp);
            //println!("  {}, {}", player.attack_cd, player.attacks_left);
            if let Some(damage) = player.attack(&rng, hunllef.defensive_roll) {
                if hunllef.hp < damage {
                    hunllef.hp = 0;
                } else {
                    hunllef.hp -= damage;
                }
                //println!("  hunllef takes {damage} damage");
            }

            let setup = player.current;

            if let Some(damage) = hunllef.attack(&rng, setup.rdr, setup.mdr) {
                let starting_hp = player.hp;
                if player.hp < damage {
                    player.hp = 0;
                } else {
                    player.hp -= damage;
                }
                //println!("  player takes {} damage", starting_hp - player.hp);
                //only tick eat when hunllef is attacking
                if args.tick_eat && starting_hp <= hunllef.max_hit {
                    player.eat_fish();
                    //println!("  tick ate from {} to {}", starting_hp, player.hp);
                }
            }

            //don't heal up when we're tick eating
            if !args.tick_eat && player.hp < args.eat_at_hp {
                player.eat_fish();
            }

            time += 1;
        }

        fish_rem.push(player.fish); //have the count include failure cases
        if player.hp > 0 && hunllef.hp == 0 {
            success += 1;
            times.push(time);
            //fish_rem.push(player.fish);
            //println!("SUCCESS\n");
        } else {
            //println!("FAILURE\n");
        }
    }

    let fish_eaten: Vec<u64> = fish_rem.iter().map(|n| (args.fish - *n) as u64).collect();

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

#[cfg(test)]
mod tests {
    use crate::*;

    const LVLS: Levels = Levels {
        attack: 99,
        strength: 99,
        defence: 99,
        ranged: 99,
        magic: 99,
        prayer: 99,
        hp: 99,
    };

    #[test]
    fn t1armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, &LVLS, 1);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.acc_roll, 31752);
        assert_eq!(setup.rdr, 30130);
        assert_eq!(setup.mdr, 25990);
    }

    #[test]
    fn t2armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, &LVLS, 2);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.acc_roll, 33264);
        assert_eq!(setup.rdr, 37728);
        assert_eq!(setup.mdr, 32544);
    }

    #[test]
    fn t3armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, &LVLS, 3);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.acc_roll, 34776);
        assert_eq!(setup.rdr, 45588);
        assert_eq!(setup.mdr, 39324);
    }

    #[test]
    fn t1armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, &LVLS, 1);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 35376);
        assert_eq!(setup.rdr, 30130);
        assert_eq!(setup.mdr, 30360);
    }

    #[test]
    fn t2armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, &LVLS, 2);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 36984);
        assert_eq!(setup.rdr, 37728);
        assert_eq!(setup.mdr, 38016);
    }

    #[test]
    fn t3armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, &LVLS, 3);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 38592);
        assert_eq!(setup.rdr, 45588);
        assert_eq!(setup.mdr, 45936);
    }

    #[test]
    fn stats_70() {
        let lvls = Levels {
            ranged: 70,
            magic: 70,
            defence: 70,
            ..LVLS
        };
        let setup = Setup::new(Weapon::Staff, Prayer::MysticMight, &lvls, 1);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 24024);
        assert_eq!(setup.rdr, 20240);
        assert_eq!(setup.mdr, 20470);

        let setup = Setup::new(Weapon::Bow, Prayer::EagleEye, &lvls, 1);
        assert_eq!(setup.max_hit, 28);
        assert_eq!(setup.acc_roll, 22176);
        assert_eq!(setup.rdr, 20240);
        assert_eq!(setup.mdr, 18400);
    }

    #[test]
    fn stats_mixed() {
        let lvls = Levels {
            ranged: 90,
            magic: 85,
            defence: 80,
            ..LVLS
        };
        let setup = Setup::new(Weapon::Staff, Prayer::MysticMight, &lvls, 2);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 29808);
        assert_eq!(setup.rdr, 28800);
        assert_eq!(setup.mdr, 30240);

        let setup = Setup::new(Weapon::Bow, Prayer::EagleEye, &lvls, 2);
        assert_eq!(setup.max_hit, 35);
        assert_eq!(setup.acc_roll, 29304);
        assert_eq!(setup.rdr, 28800);
        assert_eq!(setup.mdr, 27360);
    }
}
