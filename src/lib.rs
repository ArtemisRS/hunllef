use clap::ValueEnum;
use fastrand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Weapon {
    Bow,
    Staff,
    Halberd,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Prayer {
    Rigour,
    Augury,
    Piety,
    EagleEye,
    MysticMight,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct Levels {
    pub attack: u8,
    pub strength: u8,
    pub defence: u8,
    pub ranged: u8,
    pub magic: u8,
    pub prayer: u8,
    pub hp: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Setup {
    weapon: Weapon,
    attack_delay: u8,
    max_hit: u16,
    acc_roll: u16,
    rdr: u16, //ranged defensive roll
    mdr: u16, //magic defensive roll
}

impl Setup {
    pub fn new(
        weapon: Weapon,
        weapon_tier: u8,
        prayer: Prayer,
        levels: &Levels,
        armour_tier: u8,
    ) -> Setup {
        fn effective_level(level: u8, prayer_bonus: u8, stance_bonus: u8) -> u16 {
            (level as u16) * (100 + prayer_bonus as u16) / 100 + 8 + stance_bonus as u16
        }

        let (armour_acc, armour_def) = match armour_tier {
            1 => (16, 166),
            2 => (28, 224),
            3 => (40, 284),
            _ => (0, 0),
        };

        //for the staff, eq_str == max_hit
        let (weapon_acc, eq_str) = match (weapon, weapon_tier) {
            (Weapon::Bow, 3) => (172, 138),
            (Weapon::Staff, 3) => (184, 39),
            (Weapon::Halberd, 3) => (166, 138),
            (Weapon::Bow, 2) => (118, 88),
            (Weapon::Staff, 2) => (128, 31),
            (Weapon::Halberd, 2) => (114, 88),
            (Weapon::Bow, 1) => (72, 42),
            (Weapon::Staff, 1) => (84, 23),
            (Weapon::Halberd, 1) => (68, 42),
            (_, _) => panic!("weapon_tier must be 1, 2, or 3"),
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

        let stance_bonus = if weapon == Weapon::Staff { 3 } else { 0 };
        let eff_acc_lvl = effective_level(acc_lvl, prayer_acc, stance_bonus);
        let acc_roll = eff_acc_lvl * (eq_acc + 64);

        let stance_bonus = if weapon == Weapon::Halberd { 3 } else { 0 };
        let eff_str_lvl = effective_level(dam_lvl, prayer_str, stance_bonus);
        let max_hit = match weapon {
            Weapon::Bow | Weapon::Halberd => (eff_str_lvl * (eq_str + 64) + 320) / 640,
            Weapon::Staff => eq_str,
        };

        let eff_def_lvl = effective_level(levels.defence, prayer_def, 0);
        let rdr = eff_def_lvl * (armour_def + 64);

        let stance_bonus = if weapon == Weapon::Staff { 3 } else { 0 };
        let eff_magic_lvl = effective_level(levels.magic, prayer_def_magic, stance_bonus);

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
pub struct Hunllef {
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
    pub fn new(armour_tier: u8) -> Hunllef {
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
            if rng.u16(0..self.acc_roll + 1) > rng.u16(0..pdr + 1) {
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
pub struct Player<'a, 'b> {
    setup1: &'a Setup,
    setup2: &'a Setup,
    levels: &'b Levels,
    hp: u16,
    fish: u8,
    redemption: u8,   //number of times to attempt redemption
    attack_cd: u8,    //ticks
    attacks_left: u8, //before switching styles
    current: &'a Setup,
}

impl<'a, 'b> Player<'a, 'b> {
    pub fn new<'s, 'l>(
        setup1: &'s Setup,
        setup2: &'s Setup,
        levels: &'l Levels,
        fish: u8,
        redemption: u8,
        lost_ticks: u8,
    ) -> Player<'s, 'l> {
        let attack_cd = lost_ticks;
        let attacks_left = 6;
        let hp = levels.hp as u16;
        Player {
            setup1,
            setup2,
            levels,
            hp,
            fish,
            redemption,
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

    fn _redemption_heal(&mut self) {
        if self.redemption > 0 {
            self.redemption -= 1;
            self.hp += self.levels.prayer as u16 / 4;
            //println!("  redemption healing to take us to {}", self.hp);
        }
    }
}

pub fn run_simulation(
    trials: u32,
    player: &Player,
    hunllef: &Hunllef,
    eat_at_hp: u16,
    _tick_eat: bool,
    _max_time: u16,
) -> (u32, Vec<u64>, Vec<u16>) {
    let mut times = Vec::new();
    let mut fish_rem = Vec::new();
    let mut success = 0;
    let rng = fastrand::Rng::new();

    #[cfg(feature = "advanced")]
    //this ensures that in tick eat sims we don't heal up too much
    let eat_at_hp = if _tick_eat { 0 } else { eat_at_hp };

    // dbg!(&Player);

    for _ in 0..trials {
        //println!("loop {n}");
        let mut player = Player::new(
            player.setup1,
            player.setup2,
            player.levels,
            player.fish,
            player.redemption,
            player.attack_cd,
        );
        let mut hunllef = *hunllef;
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
                let _starting_hp = player.hp;
                if player.hp < damage {
                    player.hp = 0;
                } else {
                    player.hp -= damage;
                }
                //println!("  player takes {} damage", _starting_hp - player.hp);

                //only tick eat/redemption when hunllef is attacking
                #[cfg(feature = "advanced")]
                {
                    if _starting_hp > hunllef.max_hit {
                        //redemption when hp is under 10% of max hp
                        //@90 hp be below 9, @91hp be below 10
                        if player.hp < (player.levels.hp as u16 - 1) / 10 + 1 {
                            player._redemption_heal();
                        }
                    } else if _tick_eat {
                        player.eat_fish();
                        //println!("  tick ate from {} to {}", _starting_hp, player.hp);
                    }
                }
            }

            //TODO: This should probably move under Hunllef attacks. Player HP
            //can only drop below the threshold after being attacked
            if player.hp < eat_at_hp {
                player.eat_fish();
            }

            time += 1;

            #[cfg(feature = "advanced")]
            {
                if time > _max_time {
                    break;
                }
            }
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

    let fish_eaten: Vec<u64> = fish_rem.iter().map(|n| (player.fish - *n) as u64).collect();

    (success, fish_eaten, times)
}

pub fn data_mode(
    trials: u32,
    player: &Player,
    hunllef: &Hunllef,
    eat_at_hp: u16,
    tick_eat: bool,
    max_time: u16,
) -> Vec<u32> {
    let mut success_rate: Vec<u32> = Vec::with_capacity(player.fish as usize);
    for i in 0..=player.fish {
        let player = Player::new(
            player.setup1,
            player.setup2,
            player.levels,
            i,
            player.redemption,
            player.attack_cd,
        );

        let (success, _, _) = run_simulation(
            trials,
            &player,
            hunllef,
            eat_at_hp,
            tick_eat,
            max_time,
        );

        success_rate.push(success);
    }
    success_rate
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
    fn t1_bow() {
        let setup = Setup::new(Weapon::Bow, 1, Prayer::Rigour, &LVLS, 1);
        assert_eq!(setup.max_hit, 21);
        assert_eq!(setup.acc_roll, 19152);
        assert_eq!(setup.rdr, 30130);
        assert_eq!(setup.mdr, 25990);
    }

    #[test]
    fn t2_bow() {
        let setup = Setup::new(Weapon::Bow, 2, Prayer::Rigour, &LVLS, 2);
        assert_eq!(setup.max_hit, 31);
        assert_eq!(setup.acc_roll, 26460);
        assert_eq!(setup.rdr, 37728);
        assert_eq!(setup.mdr, 32544);
    }

    #[test]
    fn t3_bow() {
        let setup = Setup::new(Weapon::Bow, 3, Prayer::Rigour, &LVLS, 3);
        assert_eq!(setup.max_hit, 41);
        assert_eq!(setup.acc_roll, 34776);
        assert_eq!(setup.rdr, 45588);
        assert_eq!(setup.mdr, 39324);
    }

    #[test]
    fn t1_staff() {
        let setup = Setup::new(Weapon::Staff, 1, Prayer::Augury, &LVLS, 1);
        assert_eq!(setup.max_hit, 23);
        assert_eq!(setup.acc_roll, 21976);
        assert_eq!(setup.rdr, 30130);
        assert_eq!(setup.mdr, 30360);
    }

    #[test]
    fn t2_staff() {
        let setup = Setup::new(Weapon::Staff, 2, Prayer::Augury, &LVLS, 2);
        assert_eq!(setup.max_hit, 31);
        assert_eq!(setup.acc_roll, 29480);
        assert_eq!(setup.rdr, 37728);
        assert_eq!(setup.mdr, 38016);
    }

    #[test]
    fn t3_staff() {
        let setup = Setup::new(Weapon::Staff, 3, Prayer::Augury, &LVLS, 3);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 38592);
        assert_eq!(setup.rdr, 45588);
        assert_eq!(setup.mdr, 45936);
    }

    #[test]
    fn t1_halberd() {
        let setup = Setup::new(Weapon::Halberd, 1, Prayer::Piety, &LVLS, 1);
        assert_eq!(setup.max_hit, 22);
        assert_eq!(setup.acc_roll, 18648);
        assert_eq!(setup.rdr, 30130);
        assert_eq!(setup.mdr, 25990);
    }

    #[test]
    fn t2_halberd() {
        let setup = Setup::new(Weapon::Halberd, 2, Prayer::Piety, &LVLS, 2);
        assert_eq!(setup.max_hit, 31);
        assert_eq!(setup.acc_roll, 25956);
        assert_eq!(setup.rdr, 37728);
        assert_eq!(setup.mdr, 32544);
    }

    #[test]
    fn t3_halberd() {
        let setup = Setup::new(Weapon::Halberd, 3, Prayer::Piety, &LVLS, 3);
        assert_eq!(setup.max_hit, 42);
        assert_eq!(setup.acc_roll, 34020);
        assert_eq!(setup.rdr, 45588);
        assert_eq!(setup.mdr, 39324);
    }

    #[test]
    fn stats_70() {
        let lvls = Levels {
            ranged: 70,
            magic: 70,
            defence: 70,
            attack: 70,
            strength: 70,
            ..LVLS
        };
        let setup = Setup::new(Weapon::Staff, 3, Prayer::MysticMight, &lvls, 1);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 24024);
        assert_eq!(setup.rdr, 20240);
        assert_eq!(setup.mdr, 20470);

        let setup = Setup::new(Weapon::Bow, 3, Prayer::EagleEye, &lvls, 1);
        assert_eq!(setup.max_hit, 28);
        assert_eq!(setup.acc_roll, 22176);
        assert_eq!(setup.rdr, 20240);
        assert_eq!(setup.mdr, 18400);

        let setup = Setup::new(Weapon::Halberd, 3, Prayer::Piety, &lvls, 1);
        assert_eq!(setup.max_hit, 31);
        assert_eq!(setup.acc_roll, 22632);
        assert_eq!(setup.rdr, 21850);
        assert_eq!(setup.mdr, 18860);
    }

    #[test]
    fn stats_mixed() {
        let lvls = Levels {
            ranged: 90,
            magic: 85,
            defence: 80,
            attack: 82,
            strength: 87,
            ..LVLS
        };
        let setup = Setup::new(Weapon::Staff, 3, Prayer::MysticMight, &lvls, 2);
        assert_eq!(setup.max_hit, 39);
        assert_eq!(setup.acc_roll, 29808);
        assert_eq!(setup.rdr, 28800);
        assert_eq!(setup.mdr, 30240);

        let setup = Setup::new(Weapon::Bow, 3, Prayer::EagleEye, &lvls, 2);
        assert_eq!(setup.max_hit, 35);
        assert_eq!(setup.acc_roll, 29304);
        assert_eq!(setup.rdr, 28800);
        assert_eq!(setup.mdr, 27360);

        let setup = Setup::new(Weapon::Halberd, 3, Prayer::Piety, &lvls, 2);
        assert_eq!(setup.max_hit, 37);
        assert_eq!(setup.acc_roll, 27348);
        assert_eq!(setup.rdr, 31104);
        assert_eq!(setup.mdr, 27936);
    }

    #[test]
    fn hunllef_stats() {
        let hunllef = Hunllef::new(1);
        assert_eq!(hunllef.max_hit, 13);
        assert_eq!(hunllef.acc_roll, 38346);
        assert_eq!(hunllef.defensive_roll, 20916);
    }
}
