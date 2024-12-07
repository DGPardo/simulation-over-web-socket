use nbody::simulation::Simulation;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

pub struct ServerState {
    pub simulation: (Arc<AtomicUsize>, Arc<Mutex<Simulation>>),
    pub connected_clients: Arc<Mutex<Vec<UnboundedSender<Message>>>>,
}

impl ServerState {
    pub fn new() -> Self {
        let simulation = Arc::new(Mutex::new(Simulation::new()));
        let stepper = Arc::new(AtomicUsize::new(0));

        // spawn a new task to run the simulation
        spawn_simulation(Arc::clone(&stepper), Arc::clone(&simulation));

        Self {
            simulation: (stepper, simulation),
            connected_clients: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn spawn_simulation(
    counter: Arc<AtomicUsize>,
    simulation: Arc<Mutex<Simulation>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut last_update = std::time::Instant::now();
        let max_fps = std::time::Duration::from_secs_f64(1.0 / 60.0); // maximum front-end limit
        loop {
            if last_update.elapsed() > max_fps {
                let mut simulation = simulation.lock().unwrap_or_else(|p| p.into_inner());
                simulation.step();
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                last_update = std::time::Instant::now();
            }
        }
    })
}
