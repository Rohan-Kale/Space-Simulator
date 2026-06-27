use crate::physics::octree::OctreeNode;
use std::time::Instant;
use bytemuck::{Pod, Zeroable};
pub struct Body {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub mass: f32,
    pub radius: f32,
}

impl Body {
    pub fn new( position: [f32; 3], velocity: [f32; 3], acceleration: [f32; 3], mass: f32, radius: f32 /*trail: Vec<[f32; 3]>*/) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            mass,
            radius,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuBody {
    pub position: [f32; 4],      // xyz + padding
    pub velocity: [f32; 4],      // xyz + padding
    pub acceleration: [f32; 4],  // xyz + padding
    pub mass: f32,
    pub radius: f32,
    pub _padding: [f32; 2],
}

impl From<&Body> for GpuBody {
    fn from(body: &Body) -> Self {
        Self {
            position: [
                body.position[0],
                body.position[1],
                body.position[2],
                0.0,
            ],
            velocity: [
                body.velocity[0],
                body.velocity[1],
                body.velocity[2],
                0.0,
            ],
            acceleration: [
                body.acceleration[0],
                body.acceleration[1],
                body.acceleration[2],
                0.0,
            ],
            mass: body.mass,
            radius: body.radius,
            _padding: [0.0; 2],
        }
    }
}

pub fn update_bodies(bodies: &mut Vec<Body>, delta_time: f32) {
    //let start = Instant::now();
    let mut tree = OctreeNode::new([0.0,0.0,0.0], 100.0);
    for i in 0..bodies.len() {
        tree.insert(i, bodies);
    }

    tree.compute_center_of_mass(bodies);
    //println!("tree build: {:?}", start.elapsed());
    // println!(
    //     "Tree mass: {}, COM: {:?}",
    //     tree.mass,
    //     tree.center_of_mass
    // );

    let gravitational_constant = 1.0; // Adjusted for simulation scale

    let theta = 1.0;

    for i in 0..bodies.len() {
        let net_force = tree.compute_force(i, bodies, theta, gravitational_constant);
        
        bodies[i].acceleration[0] = net_force[0] / bodies[i].mass;
        bodies[i].acceleration[1] = net_force[1] / bodies[i].mass;
        bodies[i].acceleration[2] = net_force[2] / bodies[i].mass;
    }

    for body in bodies.iter_mut() {
        body.velocity[0] += body.acceleration[0] * delta_time;
        body.velocity[1] += body.acceleration[1] * delta_time;
        body.velocity[2] += body.acceleration[2] * delta_time;

        body.position[0] += body.velocity[0] * delta_time;
        body.position[1] += body.velocity[1] * delta_time;
        body.position[2] += body.velocity[2] * delta_time;
        //println!("pos {:?}, vel {:?}", body.position, body.velocity);
    }
    //println!("force: {:?}", start.elapsed());
    
}

