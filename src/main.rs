mod array_3d;
mod camera;
mod fixed_point;
mod game_state;
mod physics_world;
mod render_util;
mod texture;
mod voxel;
mod window;

fn main() {
    env_logger::init();
    window::run();
}
