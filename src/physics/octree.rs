use crate::physics::Body;


pub struct OctreeNode {
    // cube center
    pub center: [f32; 3],

    // half the cube width
    pub half_size: f32,

    // eight children
    pub children: [Option<Box<OctreeNode>>; 8],

    // body stored if this is a leaf
    pub body: Option<usize>,

    // Barnes-Hut data
    pub mass: f32,
    pub center_of_mass: [f32; 3],
}

impl OctreeNode {
    pub fn new(center: [f32; 3], half_size: f32) -> Self {
        Self {
            center,
            half_size,
            children: Default::default(),
            body: None,
            mass: 0.0,
            center_of_mass: [0.0, 0.0, 0.0],
        }
    }

    pub fn insert(&mut self, body_index: usize, bodies: &[Body]) {
        let position = bodies[body_index].position;

        //empty leaf
        if self.body.is_none() && self.children.iter().all(|c| c.is_none()) {
            self.body = Some(body_index);
            return;
        }

        //we already have a body here
        if self.children.iter().all(|c| c.is_none()) {
            self.subdivide();

            // move old body into child
            if let Some(old_body) = self.body.take() {
                let child = self.get_child_index(bodies[old_body].position);
                self.children[child]
                    .as_mut()
                    .unwrap()
                    .insert(old_body, bodies);
            }
        }

        // insert new body into correct child
        let child = self.get_child_index(position);
        self.children[child]
            .as_mut()
            .unwrap()
            .insert(body_index, bodies);
    }

     fn subdivide(&mut self) {
        let size = self.half_size / 2.0;

        for i in 0..8 {
            let offset = [
                if i & 1 == 0 { -size } else { size },
                if i & 2 == 0 { -size } else { size },
                if i & 4 == 0 { -size } else { size },
            ];
            self.children[i] = Some(Box::new(
                OctreeNode::new(
                    [
                        self.center[0] + offset[0],
                        self.center[1] + offset[1],
                        self.center[2] + offset[2],
                    ],
                    size,
                )
            ));
        }
    }

    fn get_child_index(&self, position: [f32;3]) -> usize {
        let mut index = 0;
        if position[0] >= self.center[0] {
            index |= 1;
        }
        if position[1] >= self.center[1] {
            index |= 2;
        }
        if position[2] >= self.center[2] {
            index |= 4;
        }
        index
    }

    pub fn compute_center_of_mass(&mut self, bodies: &[Body]) {
        // Leaf node
        if self.children.iter().all(|c | c.is_none()) {
            if let Some(body_index) = self.body {
                self.mass = bodies[body_index].mass;
                self.center_of_mass = bodies[body_index].position;
            } 
            else {
                self.mass = 0.0;
                self.center_of_mass = [0.0; 3];
            }
            return;
        }
        // Internal node
        self.mass = 0.0;
        self.center_of_mass = [0.0; 3];

        for child in self.children.iter_mut() {
            if let Some(child) = child {
                child.compute_center_of_mass(bodies);

                self.mass += child.mass;

                self.center_of_mass[0] += child.center_of_mass[0] * child.mass;
                self.center_of_mass[1] += child.center_of_mass[1] * child.mass;
                self.center_of_mass[2] += child.center_of_mass[2] * child.mass;
            }
        }

        if self.mass > 0.0 {
            self.center_of_mass[0] /= self.mass;
            self.center_of_mass[1] /= self.mass;
            self.center_of_mass[2] /= self.mass;
        }
    }

    pub fn compute_force(&self, body_index: usize, bodies: &[Body], theta: f32, g: f32) -> [f32; 3] {
        let mut force = [0.0; 3];

        self.compute_force_recursive(body_index, bodies, theta, g, &mut force);

        force
    }

    fn compute_force_recursive(&self, body_index: usize, bodies: &[Body], theta: f32, g: f32, force: &mut [f32; 3]) {
        if self.mass == 0.0 {
            return;
        }

        //at a leaf
        if self.children.iter().all(|c| c.is_none()) {
            if let Some(other) = self.body {
                if other == body_index {
                    return;
                }
                //compute the exact gravity
                let dx = bodies[other].position[0] - bodies[body_index].position[0];
                let dy = bodies[other].position[1] - bodies[body_index].position[1];
                let dz = bodies[other].position[2] - bodies[body_index].position[2];

                let softening = 0.01;

                let dist2 = dx * dx + dy * dy + dz * dz + softening;

                let dist = dist2.sqrt();

                let f = g * bodies[body_index].mass * bodies[other].mass / dist2;

                force[0] += f * dx / dist;
                force[1] += f * dy / dist;
                force[2] += f * dz / dist;

                return;
            }
            return;
        }

        //Barnes-Hut calculations
        let dx = self.center_of_mass[0] - bodies[body_index].position[0];
        let dy = self.center_of_mass[1] - bodies[body_index].position[1];
        let dz = self.center_of_mass[2] - bodies[body_index].position[2];

        let distance = (dx * dx + dy * dy + dz * dz + 0.01).sqrt();
        let size = self.half_size * 2.0;

        if size / distance < theta {
            let dist2 = distance * distance;
            let f = g * bodies[body_index].mass * self.mass / dist2;

            force[0] += f * dx / distance;
            force[1] += f * dy / distance;
            force[2] += f * dz / distance;
        } else {
            for child in &self.children {
                if let Some(child) = child {
                    child.compute_force_recursive(body_index, bodies, theta, g, force);
                }
            }
        }
    }

}
