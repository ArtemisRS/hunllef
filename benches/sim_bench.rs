use criterion::{criterion_group, criterion_main, Criterion};
use hunllef::{Hunllef, Levels, Player, Prayer, Setup, Weapon};

fn criterion_benchmark(c: &mut Criterion) {
    let levels = Levels {
        attack: 99,
        strength: 99,
        defence: 99,
        ranged: 99,
        magic: 99,
        prayer: 99,
        hp: 99,
    };

    let bow_setup = Setup::new(Weapon::Bow, 3, Prayer::Rigour, &levels, 1);
    let staff_setup = Setup::new(Weapon::Staff, 3, Prayer::Augury, &levels, 1);

    let player = Player::new(&bow_setup, &staff_setup, &levels, 12, 0, 0);

    let hunllef = Hunllef::new(1);

    c.bench_function("10k basic", |b| {
        b.iter(|| hunllef::run_simulation(10_000, &player, &hunllef, 50, false, 6000))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
