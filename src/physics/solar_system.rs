use crate::physics::body::Body;

const G: f32 = 39.4784176;

pub fn create_solar_system() -> Vec<Body> {
    let mut bodies = Vec::new();

    // Sun
    bodies.push(Body {
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        acceleration: [0.0, 0.0, 0.0],
        mass: 1.0,
        radius: 0.25,
    });

    // mass (solar masses), distance (AU), visual radius
    let planets = [
        // Mercury
        (1.65e-7, 0.39, 0.02),

        // Venus
        (2.45e-6, 0.72, 0.04),

        // Earth
        (3.00e-6, 1.00, 0.05),

        // Mars
        (3.22e-7, 1.52, 0.05),

        // Jupiter
        (9.54e-4, 5.20, 0.22),

        // Saturn
        (2.85e-4, 9.54, 0.20),

        // Uranus
        (4.37e-5, 19.2, 0.17),

        // Neptune
        (5.15e-5, 30.1, 0.17),
    ];

    for (mass, distance, radius) in planets {
        let orbital_speed = (G / distance).sqrt();

        bodies.push(Body {
            position: [distance, 0.0, 0.0],
            velocity: [0.0, 0.0, orbital_speed],
            acceleration: [0.0, 0.0, 0.0],
            mass,
            radius,
        });
    }

    bodies
}
