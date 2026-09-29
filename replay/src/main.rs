//! Headless replay of the repository's CPU Barnes-Hut implementation.
#[allow(dead_code, unused_imports)]
#[path = "../../src/physics/mod.rs"]
mod physics;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use std::io::{self, Write};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: space-replay SEED STEPS DT ASTEROIDS".into());
    }
    let seed: u64 = args[0].parse()?;
    let steps: u32 = args[1].parse()?;
    let dt: f32 = args[2].parse()?;
    let asteroids: usize = args[3].parse()?;
    if steps == 0 || steps > 1_000_000 || !dt.is_finite() || dt <= 0.0 || asteroids > 10000 {
        return Err("require steps 1..1000000, finite positive dt, asteroids 0..10000".into());
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut bodies = physics::solar_system::create_solar_system_with_rng(&mut rng, asteroids);
    for _ in 0..steps {
        physics::body::update_bodies(&mut bodies, dt);
    }
    let mut states = Vec::with_capacity(bodies.len());
    for b in bodies {
        if !b.position.iter().chain(&b.velocity).chain(&b.acceleration).all(|x| x.is_finite()) {
            return Err("non-finite simulation state".into());
        }
        states.push(json!({"position": b.position, "velocity": b.velocity,
            "acceleration": b.acceleration, "mass": b.mass, "radius": b.radius}));
    }
    let output = json!({"schema_version": 1, "backend": "cpu-barnes-hut-v1",
        "seed": seed, "steps": steps, "dt": dt, "asteroids": asteroids, "bodies": states});
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    serde_json::to_writer(&mut stdout, &output)?;
    writeln!(stdout)?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
