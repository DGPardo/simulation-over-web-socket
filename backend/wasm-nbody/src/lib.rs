use nbody::simulation::Simulation;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


#[wasm_bindgen]
pub struct WasmSimulation {
    simulation: Simulation,
    x_position: Vec<f32>, // WASM-Shared memory with frontend
    y_position: Vec<f32>, // WASM-Shared memory with frontend
}


#[wasm_bindgen]
impl WasmSimulation {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmSimulation {
        let simulation = Simulation::default();
        let n_bodies = simulation.bodies().len();
        Self {
            simulation,
            x_position: vec![0.0; n_bodies],
            y_position: vec![0.0; n_bodies],
        }
    }

    #[wasm_bindgen(js_name = addBody)]
    pub fn add_body(&mut self, x: f32, y: f32, mass: f32) {
        self.simulation.add_body(x, y, mass);
        self.x_position.push(x);
        self.y_position.push(y);
    }

    #[wasm_bindgen(js_name = updateBody)]
    pub fn update_body(&mut self, idx: usize, mass: f32) {
        self.simulation.update_body(idx, mass);
    }

    pub fn step(&mut self) {
        self.simulation.step();

        for (idx, body) in self.simulation.bodies().iter().enumerate() {
            self.x_position[idx] = body.position[0];
            self.y_position[idx] = body.position[1];
        }
    }

    #[wasm_bindgen(js_name = numberOfBodies)]
    pub fn number_of_bodies(&self) -> usize {
        self.simulation.bodies().len()
    }

    #[wasm_bindgen(js_name = xPosition)]
    pub fn x_position(&self) -> *const f32 {
        self.x_position.as_ptr()
    }

    #[wasm_bindgen(js_name = yPosition)]
    pub fn y_position(&self) -> *const f32 {
        self.y_position.as_ptr()
    }
}
