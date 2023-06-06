use bevy::{
    ecs::system::Command,
    prelude::*,
    tasks::IoTaskPool,
};

use std::{
    io::{Read, Write},
    fs::File,
    sync::Mutex,
};

pub struct RngPlugin;

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EarthRng(fastrand::Rng::new().into()));
    }
}

#[derive(Resource, Debug)]
pub struct EarthRng(pub Mutex<fastrand::Rng>);

#[derive(Resource, Debug, Clone, Copy)]
pub struct LastGenerationSeed(pub u64);

pub struct SaveSeed {
    pub file: File,
}

impl Command for SaveSeed {
    fn write(mut self, world: &mut World) {
        let seed = world.get_resource::<LastGenerationSeed>();

        if seed.is_none() {
            warn!("save requested for non existent generation seed, saving nothing");
            return;
        }

        // It's probably overkill to use the IoTaskPool here, but
        // maybe we want to save more data in the future.
        let io_pool = IoTaskPool::get();

        let seed = seed.unwrap().0;
        io_pool.spawn(async move { write!(self.file, "{}", seed) }).detach();
    }
}

pub struct LoadSeed {
    pub file: File,
}

impl Command for LoadSeed {
    fn write(mut self, world: &mut World) {
        let rng = world.resource::<EarthRng>();

        // The width of u64::MAX in decimal is 20, we need a digit for
        // the sign as well.
        let mut seed_buf = [0u8; 21];

        // Doing this IO in a command write is sub optimal, but it's
        // easy, and we're reading a very small amount of data at a
        // time where the user probably wouldn't be surprised at a
        // microstutter. Therefore, this decision should be fine as a
        // placeholder.
        let read_result = self.file.read(&mut seed_buf);

        match read_result {
            Err(e) => error!("{}", e),
            Ok(length) => {
                let read_result = std::str::from_utf8(&seed_buf[..length])
                    .expect("seed file did not contain valid utf8")
                    .parse::<u64>();

                match read_result {
                    Err(e) => error!("unable to parse file contents as integer: {}", e),
                    Ok(new_seed) => {
                        let guard_result = rng.0.lock();

                        match guard_result {
                            Err(e) => error!("unable to lock rng for seed loading: {}", e),
                            Ok(guard) => guard.seed(new_seed),
                        }
                    },
                }
            }
        }
    }
}
