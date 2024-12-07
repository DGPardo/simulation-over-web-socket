use std::io::Write;

use flate2::{
    write::{GzDecoder, GzEncoder},
    Compression,
};
use nbody::physics::Body;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(from_wasm_abi, into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum ClientToServerMessage {
    Subscribe,
    AddBodies(Vec<Body>),
    State,
    Reset,
}

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(from_wasm_abi, into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum ServerToClientMessage {
    #[serde(rename_all = "camelCase")]
    StateUpdate {
        bodies: Vec<Body>,
        physical_time: f64,
        kinetic_energy: f64,
    },
}

#[wasm_bindgen(js_name = serializeServerMsg)]
pub fn serialize_server_msg(msg: ServerToClientMessage) -> Option<Vec<u8>> {
    serialize_and_compress(msg)
}

#[wasm_bindgen(js_name = deserializeServerMsg)]
pub fn deserialize_server_msg(msg: &[u8]) -> Option<ServerToClientMessage> {
    decompress_and_deserialize(msg)
}

#[wasm_bindgen(js_name = serializeClientMsg)]
pub fn serialize_client_msg(msg: ClientToServerMessage) -> Option<Vec<u8>> {
    serialize_and_compress(msg)
}

#[wasm_bindgen(js_name = deserializeClientMsg)]
pub fn deserialize_client_msg(msg: &[u8]) -> Option<ClientToServerMessage> {
    decompress_and_deserialize(msg)
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::fast());
    e.write_all(data)?;
    Ok(e.finish()?)
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut d = GzDecoder::new(Vec::new());
    d.write_all(data)?;
    Ok(d.finish()?)
}

fn serialize_and_compress<T: Serialize>(msg: T) -> Option<Vec<u8>> {
    bincode::serialize(&msg)
        .map(|data| compress_data(&data).ok())
        .ok()
        .flatten()
}

fn decompress_and_deserialize<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Option<T> {
    decompress_data(data)
        .map(|msg| bincode::deserialize(&msg).ok())
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serialization_test() {
        let bodies = vec![Body::default(); 10];
        let serialized =
            serialize_client_msg(ClientToServerMessage::AddBodies(bodies.clone())).unwrap();
        let deserialized = deserialize_client_msg(&serialized).unwrap();

        match deserialized {
            ClientToServerMessage::AddBodies(bodies) => assert!(bodies.len() == 10),
            _ => panic!("Expected Subscribe"),
        };
    }
}
