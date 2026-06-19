pub struct Body {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub acceleration: [f32; 2],
    pub mass: f32,
    pub radius: f32,

}

impl Body {
    pub fn new( position: [f32; 2], velocity: [f32; 2], acceleration: [f32; 2], mass: f32, radius: f32) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            mass,
            radius,
        }
    }
}

pub fn update_bodies(bodies: &mut Vec<Body>, delta_time: f32) {
    let gravitational_constant = 6.67430e-11; // Gravitational constant

    for i in 0..bodies.len() {
        let mut net_force = [0.0, 0.0]; // initialize force for body i

        for j in 0..bodies.len() {
            if i != j {
                let dx = bodies[j].position[0] - bodies[i].position[0];
                let dy = bodies[j].position[1] - bodies[i].position[1];

                let distance_squared = dx * dx + dy * dy;

                if distance_squared > 0.0 {
                    let distance = distance_squared.sqrt();
                    let force = gravitational_constant * bodies[i].mass * bodies[j].mass / distance_squared;    // F = G * (m1 * m2) / r^2

                    net_force[0] += force * dx / distance;  // calculate force components and add to net force
                    net_force[1] += force * dy / distance;

                }
            }
        }

        // Update acceleration before updating velocity and position
        bodies[i].acceleration[0] = net_force[0] / bodies[i].mass;
        bodies[i].acceleration[1] = net_force[1] / bodies[i].mass;

    }

    for body in bodies.iter_mut() {
        // Update velocity based on acceleration
        body.velocity[0] += body.acceleration[0] * delta_time;
        body.velocity[1] += body.acceleration[1] * delta_time;

        // Update position based on velocity
        body.position[0] += body.velocity[0] * delta_time;
        body.position[1] += body.velocity[1] * delta_time;
    }
    
}
