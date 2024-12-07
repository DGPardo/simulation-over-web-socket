use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{
    quadtree::{SquareBox, SquareQuadtree},
    SMALL,
};
use std::collections::{HashSet, VecDeque};

#[derive(Tsify, Serialize, Deserialize, Copy, Clone, Debug)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct Body {
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub mass: f64,
    pub radius: f64,
    pub color: [u8; 4], // rgba
}

impl Body {
    pub fn with_mass(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_position(mut self, position: [f64; 2]) -> Self {
        self.position = position;
        self
    }

    pub fn with_velocity(mut self, velocity: [f64; 2]) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn kinectic_energy(&self) -> f64 {
        0.5 * self.mass
            * (self.velocity[0] * self.velocity[0] + self.velocity[1] * self.velocity[1])
    }
}

impl Default for Body {
    fn default() -> Self {
        Body {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            mass: 1.0,
            radius: 1.0,
            color: [255; 4],
        }
    }
}

/// Compute the gravity forces on the i-th Body using the Barnes-Hut algorithm
pub fn compute_gravity_forces(
    ith_body: usize,
    forces: &mut [[f64; 2]],
    bodies: &[Body],
    qt: &SquareQuadtree,
    theta_sqr_threshold: f64,
    gravity_constant: f64,
) {
    let body = &bodies[ith_body];
    let qt_nodes = qt.get_nodes();

    let mut stack: VecDeque<usize> = vec![0].into();
    while let Some(node_idx) = stack.pop_front() {
        if qt_nodes[node_idx].is_leaf() {
            // Brute-force gravity computation
            for &nbr_body in qt_nodes[node_idx].referenced_indices() {
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
            let distance_sqr = dx * dx + dy * dy;
            if distance_sqr < SMALL {
                continue;
            }

            if size * size / distance_sqr < theta_sqr_threshold {
                let force = gravity_constant * bodies[ith_body].mass * qt_nodes[node_idx].mass()
                    / distance_sqr;
                let distance = distance_sqr.sqrt();
                forces[ith_body][0] += force * dx / distance;
                forces[ith_body][1] += force * dy / distance;
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
    forces: &mut [[f64; 2]],
    bodies: &[Body],
    gravity_constant: f64,
) {
    let dx = bodies[jth].position[0] - bodies[ith].position[0];
    let dy = bodies[jth].position[1] - bodies[ith].position[1];
    let distance_sqr = dx * dx + dy * dy;
    if distance_sqr < SMALL {
        return;
    }

    let force = gravity_constant * bodies[ith].mass * bodies[jth].mass / distance_sqr;

    let distance = distance_sqr.sqrt();
    forces[ith][0] += force * dx / distance;
    forces[ith][1] += force * dy / distance;
}

fn elastic_collission(
    bodies: &mut [Body],
    ith: usize,
    jth: usize,
    colliding_bodies: &mut HashSet<usize>,
) {
    let relative_position = [
        bodies[jth].position[0] - bodies[ith].position[0],
        bodies[jth].position[1] - bodies[ith].position[1],
    ];

    let distance_sqr =
        relative_position[0] * relative_position[0] + relative_position[1] * relative_position[1];

    let radii_sum = bodies[ith].radius + bodies[jth].radius;
    if distance_sqr > radii_sum * radii_sum {
        // Not colliding
        return;
    }
    colliding_bodies.insert(ith);
    colliding_bodies.insert(jth);

    let distance = distance_sqr.sqrt();

    let unit_delta_pos = [
        relative_position[0] / distance,
        relative_position[1] / distance,
    ];

    // Move the bodies so that they are just touching
    bodies[jth].position[0] = bodies[ith].position[0] + unit_delta_pos[0] * radii_sum;
    bodies[jth].position[1] = bodies[ith].position[1] + unit_delta_pos[1] * radii_sum;

    let relative_velocity = [
        bodies[jth].velocity[0] - bodies[ith].velocity[0],
        bodies[jth].velocity[1] - bodies[ith].velocity[1],
    ];

    let impact_speed =
        relative_velocity[0] * unit_delta_pos[0] + relative_velocity[1] * unit_delta_pos[1];

    if impact_speed > 0.0 {
        // Not approaching
        return;
    }

    let m_i = bodies[ith].mass;
    let m_j = bodies[jth].mass;

    let impulse = 2.0 * impact_speed / (m_i + m_j);

    let ke_start = bodies[ith].kinectic_energy() + bodies[jth].kinectic_energy();

    bodies[ith].velocity[0] += unit_delta_pos[0] * impulse * m_j;
    bodies[ith].velocity[1] += unit_delta_pos[1] * impulse * m_j;

    bodies[jth].velocity[0] -= unit_delta_pos[0] * impulse * m_i;
    bodies[jth].velocity[1] -= unit_delta_pos[1] * impulse * m_i;

    // Keep constant the kinetic energy
    let curr_ke_jth = bodies[jth].kinectic_energy();
    let correct_ke_jth = ke_start - bodies[ith].kinectic_energy();
    let ratio = (correct_ke_jth / curr_ke_jth).sqrt();
    bodies[jth].velocity[0] *= ratio;
    bodies[jth].velocity[1] *= ratio;
}

/// Compute the collisions between the bodies
/// Returns true if the bodies have been updated
pub fn compute_collisions(bodies: &mut [Body], qt: &SquareQuadtree) {
    let mut colliding_bodies: HashSet<usize> = HashSet::new();

    for ith_body in 0..bodies.len() {
        if colliding_bodies.contains(&ith_body) {
            continue;
        }
        let boundary = SquareBox::new(bodies[ith_body].position, 4.0 * bodies[ith_body].radius);
        let nbr_bodies = qt.query_range(boundary, bodies);
        for &jth_body in nbr_bodies.iter() {
            if ith_body == jth_body || colliding_bodies.contains(&jth_body) {
                continue;
            }
            elastic_collission(bodies, ith_body, jth_body, &mut colliding_bodies);
        }
    }
}
