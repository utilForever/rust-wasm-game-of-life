#![feature(test)]

extern crate test;
extern crate rust_wasm_game_of_life;

#[bench]
fn universe_ticks(b: &mut test::Bencher) {
    let mut universe = rust_wasm_game_of_life::Universe::new();

    b.iter(|| {
        universe.tick();
    });
}