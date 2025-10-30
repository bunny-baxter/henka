mod array_3d;
mod camera;
mod game_state;
mod render_util;
mod texture;
mod voxel;
mod window;

fn main() {
    env_logger::init();
    window::run();
}
