use clap::{Parser};
use fastrand::Rng;


#[derive(Parser, Debug)]
#[command(name = "Hunllef")]
#[command(version = "0.1")]
#[command(about = "Simulates the Corrupted Hunllef fight", long_about = None)]
struct Cli {
    /// Number of simulations to complete
    #[arg(short, long, default_value_t = 100_000)]
    trials: u32,

    /// Number of eat (heal 20 hp)
    #[arg(short, long, default_value_t = 12)]
    fish: u8,

    /// Tier of CG armour
    #[arg(short, long, default_value_t = 1)]
    armour: u8,

    /// Ranged/Magic/Def/HP level to use (same for all)
    #[arg(short, long, default_value_t = 99)]
    level: u8,

    /// HP threshold to eat fish
    #[arg(short, long, default_value_t = 50)]
    eat_at_hp: u16,

    //prayer: TBD
}


#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Weapon {
    Bow,
    Staff,
    Halberd,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
enum Prayer {
    Rigour,
    Augury,
    Piety,
}

#[derive(Debug, Clone, Copy)]
struct Setup {
    weapon: Weapon,
    attack_delay: u8,
    max_hit: u16,
    accuracy_roll: u16,
    defensive_roll: u16,
}

impl Setup {
    fn new(weapon: Weapon, prayer: Prayer, level: u8, armour_tier: u8) -> Setup {
        fn effective_level(level: u8, prayer_bonus: u8, weapon: Option<Weapon>) -> u16 {
            let extra_bonus = match weapon {
                Some(Weapon::Bow) => 8,
                Some(Weapon::Staff) => 11,
                Some(Weapon::Halberd) => 11,
                None => 8,
            };
            (level as u16) * (100 + prayer_bonus as u16) / 100 + extra_bonus
        }

        let (armour_accuracy, armour_defense) = match armour_tier {
            1 => (16, 166),
            2 => (28, 224),
            3 => (40, 284),
            _ => (0, 0),
        };
        let (weapon_accuracy, equipment_strength) = match weapon {
            Weapon::Bow => (172, 138),
            Weapon::Staff => (184, 0),
            Weapon::Halberd => (168, 138),
        };
        let equipment_accuracy = armour_accuracy + weapon_accuracy;
        let (prayer_accuracy, prayer_strength, prayer_defensive) = match prayer {
            Prayer::Rigour => (20, 23, 25),
            Prayer::Augury => (25, 0, 25),
            Prayer::Piety => (20, 23, 25),
        };
        let effective_accuracy_level = effective_level(level, prayer_accuracy, Some(weapon));
        let accuracy_roll = effective_accuracy_level * (equipment_accuracy + 64);

        let effective_strength_level = effective_level(level, prayer_strength, Some(weapon));
        let max_hit = match weapon {
            Weapon::Bow | Weapon::Halberd => (effective_strength_level * (equipment_strength as u16 + 64) + 320) / 640 ,
            Weapon::Staff => 39,
        };

        let effective_defense_level = effective_level(level, prayer_defensive, None);
        let defensive_roll = effective_defense_level * (armour_defense + 64);

        Setup {
            weapon,
            attack_delay: 4,
            max_hit,
            accuracy_roll,
            defensive_roll,
        }
    }

    fn attack(self, rng: &Rng, hunllef_defensive_roll: u16) -> i16 {
        if rng.u16(0..self.accuracy_roll) > rng.u16(0..hunllef_defensive_roll) {
            rng.u16(0..self.max_hit+1) as i16 //range is not inclusive of top
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Hunllef {
    max_hit: u16,
    attack_delay: u8, //ticks
    accuracy_roll: u16,
    defensive_roll: u16,
    tornado_cd: u8, //number of attacks until a tornado attack
    hp: i16,
    attack_cd: u8, //ticks
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
        let accuracy_roll = (240+9) * (90+64);
        let defensive_roll = (240+9) * (20+64);
        let hp = 1000;
        let tornado_cd = 12;
        let attack_cd = 0;

        Hunllef {
            max_hit,
            attack_delay,
            accuracy_roll,
            defensive_roll,
            hp,
            tornado_cd,
            attack_cd,
        }
    }

    fn attack(&mut self, rng: &Rng, player_defensive_roll: u16) -> Option<u16> {
        if self.attack_cd == 0 {
            self.attack_cd += self.attack_delay - 1;
            if self.tornado_cd == 0 {
                //println!("tornado!");
                self.tornado_cd = rng.u8(10..15);
                return None;
            } else {
                self.tornado_cd -= 1;
            }

            if rng.u16(0..self.accuracy_roll) > rng.u16(0..player_defensive_roll) {
                Some(rng.u16(0..self.max_hit+1)) //range is not inclusive of top
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
struct Player {
    range: Setup,
    mage: Setup,
    hp: u16,
    fish: u8,
    attack_cd: u8, //ticks
    attacks_left: u8, //before switching styles
    current: Weapon,
}

impl Player {
    fn new(range: Setup, mage: Setup, hp: u16, fish: u8) -> Player {
        let attack_cd = 0;
        let attacks_left = 6;
        Player {
            range,
            mage,
            hp,
            fish,
            attack_cd,
            attacks_left,
            current: if fastrand::bool() {
                range.weapon
            } else {
                mage.weapon
            },
        }
    }

    fn switch_weapon(mut self) {
        if self.current == self.range.weapon {
            self.current = self.mage.weapon;
        } else {
            self.current = self.range.weapon;
        }
    }

    fn attack(&mut self, rng: &Rng, hunllef_defensive_roll: u16) -> Option<i16> {
        if self.attack_cd == 0 {
            //dbg!(self);
            if self.attacks_left == 0 {
                self.switch_weapon();
                self.attacks_left = 6;
            }
            let setup: &Setup = if self.current == self.range.weapon {
                &self.range
            } else {
                &self.mage
            };

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
            //println!("eating a fish to take us to {}", self.hp);
        }
    }
}



fn main() {
    let args = Cli::parse();

    let range = Setup::new(Weapon::Bow, Prayer::Rigour, args.level, args.armour);
    let mage = Setup::new(Weapon::Staff, Prayer::Augury, args.level, args.armour);

    let mut times = Vec::new();
    let mut fish_eaten = Vec::new();
    let mut success = 0;
    let rng = fastrand::Rng::new();

    for _ in 0..args.trials {
        //println!("loop {n}");
        let mut player = Player::new(range, mage, args.level as u16, args.fish);
        let mut hunllef = Hunllef::new(args.armour);
        let mut time: u16 = 0; //elapsed time for this trial

        while hunllef.hp > 0 && player.hp > 0 {
            //println!("t={:0>3}, php: {}, hhp: {}", time, player.hp, hunllef.hp);
            //println!("  {}, {}", player.attack_cd, player.attacks_left);
            if let Some(damage) = player.attack(&rng, hunllef.defensive_roll) {
                hunllef.hp -= damage;
                //println!("  hunllef takes {damage}");
            }

            if let Some(damage) = hunllef.attack(&rng, player.range.defensive_roll) {
                if player.hp < damage {
                    player.hp = 0;
                } else {
                    player.hp -= damage;
                }
                //println!("  player takes {damage}");
            }

            if player.hp < args.eat_at_hp {
                player.eat_fish();
            }

            time += 1;

        }
        if player.hp > 0 && hunllef.hp <= 0 {
            success += 1;
            times.push(time);
            fish_eaten.push(player.fish);
        }
    }
    //println!("{:?}", times);
    //println!("{:?}", fish_eaten);
    println!("success rate: {}", (success as f32 * 100.0) / (args.trials as f32));
    println!("avg fish eaten: {:.1}", args.fish as f64 - fish_eaten.iter().map(|n| *n as u64).sum::<u64>() as f64 / fish_eaten.len() as f64);
    println!("avg time: {:.1}", times.iter().map(|t| *t as u64).sum::<u64>() as f64 / times.len() as f64);
}



#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn t1armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, 99, 1);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.accuracy_roll, 31752);
        assert_eq!(setup.defensive_roll, 30130);
    }

    #[test]
    fn t2armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, 99, 2);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.accuracy_roll, 33264);
        assert_eq!(setup.defensive_roll, 37728);
    }

    #[test]
    fn t3armour_bow() {
        let setup = Setup::new(Weapon::Bow, Prayer::Rigour, 99, 3);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.accuracy_roll, 34776);
        assert_eq!(setup.defensive_roll, 45588);
    }

    #[test]
    fn t1armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, 99, 1);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.accuracy_roll, 35376);
        assert_eq!(setup.defensive_roll, 30130);
    }

    #[test]
    fn t2armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, 99, 2);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.accuracy_roll, 36984);
        assert_eq!(setup.defensive_roll, 37728);
    }

    #[test]
    fn t3armour_staff() {
        let setup = Setup::new(Weapon::Staff, Prayer::Augury, 99, 3);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.accuracy_roll, 38592);
        assert_eq!(setup.defensive_roll, 45588);
    }
}
