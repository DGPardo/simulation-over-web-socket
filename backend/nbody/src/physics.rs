use crate::{quadtree::SquareQuadtree, SMALL};
use std::collections::VecDeque;

pub struct Body {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
}

impl Body {
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }

    pub fn with_velocity(mut self, velocity: [f32; 2]) -> Self {
        self.velocity = velocity;
        self
    }
}

impl Default for Body {
    fn default() -> Self {
        Body {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            mass: 1.0,
        }
    }
}

/// Compute the gravity forces on the i-th Body using the Barnes-Hut algorithm
pub fn compute_gravity_forces(
    ith_body: usize,
    forces: &mut [[f32; 2]],
    bodies: &[Body],
    qt: &SquareQuadtree,
    theta_sqr_threshold: f32,
    gravity_constant: f32,
) {
    let body = &bodies[ith_body];
    let qt_nodes = qt.get_nodes();

    let mut stack: VecDeque<usize> = vec![0].into();
    while let Some(node_idx) = stack.pop_front() {
        if qt_nodes[node_idx].is_leaf() {
            // Brute-force gravity computation
            for &nbr_body in qt_nodes[node_idx].referenced_indexes() {
                if nbr_body != ith_body {
                    // TODO: Can we make use of symmetry to avoid double computation?
                    accumulate_gravity_force(ith_body, nbr_body, forces, bodies, gravity_constant);
                }
            }
        } else {
            let bdry = qt_nodes[node_idx].boundary();
            let center = bdry.center();
            let size = bdry.size();
            let dx = center[0] - body.position[0];
            let dy = center[1] - body.position[1];
            let distance_sqr = (dx * dx + dy * dy).max(SMALL);

            if size * size / distance_sqr < theta_sqr_threshold {
                let force = gravity_constant * bodies[ith_body].mass * qt_nodes[node_idx].mass()
                    / distance_sqr;
                let distance = distance_sqr.sqrt();
                forces[ith_body][0] -= force * dx / distance;
                forces[ith_body][1] -= force * dy / distance;
            } else {
                let first_idx = qt_nodes[node_idx].children_idx();
                stack.extend(first_idx..first_idx + 4);
            }
        }
    }
}

/// Accumulates the gravity force on the i-th body due to the j-th body
/// It does not make use of symmetry as this cannot be mixed with Barnes-Hut
#[inline(always)]
fn accumulate_gravity_force(
    ith: usize,
    jth: usize,
    forces: &mut [[f32; 2]],
    bodies: &[Body],
    gravity_constant: f32,
) {
    let dx = bodies[jth].position[0] - bodies[ith].position[0];
    let dy = bodies[jth].position[1] - bodies[ith].position[1];
    let distance_sqr = (dx * dx + dy * dy).max(SMALL);

    let force = gravity_constant * bodies[ith].mass * bodies[jth].mass / distance_sqr;

    let distance = distance_sqr.sqrt();
    forces[ith][0] -= force * dx / distance;
    forces[ith][1] -= force * dy / distance;
}
