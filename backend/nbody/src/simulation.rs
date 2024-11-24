use crate::{
    physics::{compute_gravity_forces, Body},
    quadtree::{SquareBox, SquareQuadtree},
};

pub struct SolverParameters {
    dt: f32,
    barnes_hut_theta: f32,
    quadtree_capacity: usize,
}

impl Default for SolverParameters {
    fn default() -> Self {
        SolverParameters {
            dt: 0.01,
            barnes_hut_theta: 1.0,
            quadtree_capacity: 64,
        }
    }
}

pub struct PhyiscsParameters {
    gravity_constant: f32,
}

impl Default for PhyiscsParameters {
    fn default() -> Self {
        PhyiscsParameters {
            gravity_constant: 1.0,
        }
    }
}

#[derive(Default)]
pub struct SimulationParameters {
    solver: SolverParameters,
    physics: PhyiscsParameters,
}

pub struct Simulation {
    bodies: Vec<Body>,
    forces: Vec<[f32; 2]>,
    qt: SquareQuadtree,
    parameters: SimulationParameters,
}

impl Default for Simulation {
    fn default() -> Self {
        let parameters = SimulationParameters::default();
        Simulation {
            bodies: Vec::new(),
            forces: Vec::new(),
            qt: SquareQuadtree::new(SquareBox::new(
                /*center=*/ [0.0, 0.0],
                /*half size=*/ 1.0,
            ))
            .with_capacity(parameters.solver.quadtree_capacity),
            parameters,
        }
    }
}

impl Simulation {
    pub fn with_bodies(mut self, bodies: Vec<Body>) -> Self {
        self.forces = vec![[0.0, 0.0]; bodies.len()];
        self.bodies = bodies;
        self.update_quadtree();
        self
    }

    pub fn add_body(&mut self, x: f32, y: f32, mass: f32) {
        self.bodies.push(Body {
            position: [x, y],
            velocity: [0.0, 0.0],
            mass,
        });
        self.forces.push([0.0, 0.0]);
        self.update_quadtree();
    }

    pub fn update_body(&mut self, idx: usize, mass: f32) {
        self.bodies[idx].mass = mass;
    }

    pub fn with_dt(mut self, dt: f32) -> Self {
        self.parameters.solver.dt = dt;
        self
    }

    pub fn step(&mut self) {
        self.forces.iter_mut().for_each(|f| *f = [0.0, 0.0]);
        self.update_quadtree();

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

        let dt = self.parameters.solver.dt;
        for i in 0..self.bodies.len() {
            let body = &mut self.bodies[i];
            let force = self.forces[i];
            let acceleration = [force[0] / body.mass, force[1] / body.mass];

            body.velocity[0] += acceleration[0] * dt;
            body.velocity[1] += acceleration[1] * dt;

            body.position[0] += body.velocity[0] * dt;
            body.position[1] += body.velocity[1] * dt;
        }
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }
}

// Private helper functions
impl Simulation {
    fn update_quadtree(&mut self) {
        self.qt.clear(SquareBox::from_bodies(&self.bodies));
        (0..self.bodies.len()).for_each(|i| self.qt.insert_unchecked(i, &self.bodies));
    }
}
