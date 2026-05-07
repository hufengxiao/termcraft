mod block;
mod camera;
mod game;
mod input;
mod player;
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
