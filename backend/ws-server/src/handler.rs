use nbody::simulation::Simulation;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use wasm_bindings::{serialize_server_msg, ClientToServerMessage, ServerToClientMessage};

use crate::{lock, state::ServerState};

pub async fn handle_client_to_server_messages(
    msg: ClientToServerMessage,
    state: Arc<ServerState>,
    tx: UnboundedSender<Message>,
) {
    match msg {
        ClientToServerMessage::Subscribe => {
            lock!(state.connected_clients).push(tx);
        }
        ClientToServerMessage::AddBodies(bodies) => {
            let mut simulation = lock!(state.simulation.1);
            bodies
                .into_iter()
                .for_each(|body| simulation.add_body(body));
        }
        ClientToServerMessage::State => {
            let sim_state = {
                let simulation = lock!(state.simulation.1);
                gather_state(&simulation)
            };
            match serialize_server_msg(sim_state).map(|msg| tx.send(Message::binary(msg))) {
                Some(Ok(_)) => {}
                Some(Err(e)) => eprintln!("Failed to send state update: {:?}", e),
                None => eprintln!("Failed to serialize state update"),
            }
        }
        ClientToServerMessage::Reset => {
            let mut simulation = lock!(state.simulation.1);
            simulation.reset();
        }
    }
}

pub fn gather_state(simulation: &Simulation) -> ServerToClientMessage {
    let nbodies = simulation.get_number_of_bodies();
    let bodies = (0..nbodies).map(|i| simulation.get_body(i)).collect();
    ServerToClientMessage::StateUpdate {
        bodies,
        physical_time: simulation.get_physical_time(),
        kinetic_energy: simulation.get_kinetic_energy(),
    }
}
