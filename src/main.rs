mod biome;
mod block;
mod camera;
mod game;
mod input;
mod item;
mod mob;
mod network;
mod player;
mod redstone;
mod save;
mod sound;
mod world;

use game::Game;

fn main() {
    let mut game = Game::new();
    if let Err(e) = game.run() {
        eprintln!("Error: {e}");
    }
}
