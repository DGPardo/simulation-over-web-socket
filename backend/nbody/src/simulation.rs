use crate::{
    physics::{compute_collisions, compute_gravity_forces, Body},
    quadtree::{SquareBox, SquareQuadtree},
};

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct SolverParameters {
    dt: f64, // seconds
    barnes_hut_theta: f64,
}

impl Default for SolverParameters {
    fn default() -> Self {
        SolverParameters {
            dt: 0.01,
            barnes_hut_theta: 0.0,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct PhyiscsParameters {
    gravity_constant: f64,
}

impl Default for PhyiscsParameters {
    fn default() -> Self {
        PhyiscsParameters {
            gravity_constant: 100.0,
        }
    }
}

#[derive(Default)]
pub struct SimulationParameters {
    pub solver: SolverParameters,
    pub physics: PhyiscsParameters,
}

#[wasm_bindgen]
pub struct Simulation {
    forces: Vec<[f64; 2]>,
    current_time: std::time::Duration,
    bodies: Vec<Body>,
    qt: SquareQuadtree,
    parameters: SimulationParameters,
    kinetic_energy: f64,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            forces: Vec::new(),
            current_time: std::time::Duration::new(0, 0),
            qt: SquareQuadtree::new(SquareBox::new(
                /*center=*/ [0.0, 0.0],
                /*half size=*/ 1.0,
            )),
            parameters: SimulationParameters::default(),
            kinetic_energy: 0.0,
        }
    }
}

impl Simulation {
    pub fn add_bodies(&mut self, bodies: Vec<Body>) {
        self.forces.extend(vec![[0.0, 0.0]; bodies.len()]);
        self.bodies.extend(bodies);
        self.update_quadtree();
    }
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Simulation::default()
    }

    #[wasm_bindgen(js_name = addBody)]
    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
        self.forces.push([0.0, 0.0]);
        self.update_quadtree();
    }

    #[wasm_bindgen(js_name = setSolverParameters)]
    pub fn set_solver_parameters(&mut self, parameters: SolverParameters) {
        self.parameters.solver = parameters;
    }

    #[wasm_bindgen(js_name = setPhysicsParameters)]
    pub fn set_physics_parameters(&mut self, parameters: PhyiscsParameters) {
        self.parameters.physics = parameters;
    }

    #[wasm_bindgen(js_name = getXPosition)]
    pub fn get_x_position(&self, body_idx: usize) -> f64 {
        self.bodies[body_idx].position[0]
    }

    #[wasm_bindgen(js_name = getYPosition)]
    pub fn get_y_position(&self, body_idx: usize) -> f64 {
        self.bodies[body_idx].position[1]
    }

    #[wasm_bindgen(js_name = getBody)]
    pub fn get_body(&self, body_idx: usize) -> Body {
        self.bodies[body_idx]
    }

    #[wasm_bindgen(js_name = getPhysicalTime)]
    pub fn get_physical_time(&self) -> f64 {
        self.current_time.as_secs_f64()
    }

    #[wasm_bindgen(js_name = getNumberOfBodies)]
    pub fn get_number_of_bodies(&self) -> usize {
        self.bodies.len()
    }

    #[wasm_bindgen(js_name = getKineticEnergy)]
    pub fn get_kinetic_energy(&self) -> f64 {
        self.kinetic_energy
    }

    pub fn step(&mut self) {
        self.forces.iter_mut().for_each(|f| *f = [0.0, 0.0]);
        self.update_quadtree();

        compute_collisions(&mut self.bodies, &self.qt);

        // Update physics
        let theta_sqr = self.parameters.solver.barnes_hut_theta.powi(2);
        for i in 0..self.bodies.len() {
            compute_gravity_forces(
                i,
                &mut self.forces,
                &self.bodies,
                &self.qt,
                theta_sqr,
                self.parameters.physics.gravity_constant,
            );
        }

        // Integrate
        let dt = self.parameters.solver.dt;
        self.kinetic_energy = 0.0;
        for i in 0..self.bodies.len() {
            let body = &mut self.bodies[i];
            let force = self.forces[i];
            let acceleration = [force[0] / body.mass, force[1] / body.mass];

            body.velocity[0] += acceleration[0] * dt;
            body.velocity[1] += acceleration[1] * dt;
            self.kinetic_energy += body.kinectic_energy();

            body.position[0] += body.velocity[0] * dt;
            body.position[1] += body.velocity[1] * dt;
        }
        self.current_time += std::time::Duration::from_secs_f64(dt);
    }

    pub fn reset(&mut self) {
        self.bodies.clear();
        self.forces.clear();
        self.current_time = std::time::Duration::new(0, 0);
        self.kinetic_energy = 0.0;
        self.qt = SquareQuadtree::new(SquareBox::new(
            /*center=*/ [0.0, 0.0],
            /*half size=*/ 1.0,
        ));
    }
}

// Private helper functions
impl Simulation {
    fn update_quadtree(&mut self) {
        self.qt.clear(SquareBox::from_bodies(&self.bodies));
        (0..self.bodies.len()).for_each(|i| self.qt.insert_unchecked(i, &self.bodies));
    }
}
