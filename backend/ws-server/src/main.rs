mod handler;
mod state;
mod ws;

use state::ServerState;
use std::sync::Arc;

#[macro_export]
macro_rules! lock {
    ($e:expr) => {
        // Relying on this macro such that we could add custom
        // debugging tools to try figure out deadlocks or other issues
        // for example printing the line number and file where the lock was acquired
        //
        // #[cfg(debug_assertions)]
        // println!("Lock acquired at line {} in file {}", line!(), file!());

        // This macro is used to lock a std::sync::Mutex
        // and return the inner value whether it was poisoned or not
        $e.lock().unwrap_or_else(|e| e.into_inner())
    };
}

#[tokio::main]
async fn main() {
    let state = Arc::new(ServerState::new());
    let r = ws::launch_ws_server(Arc::clone(&state)).await;
    if let Err(e) = r {
        eprintln!("Existing server with error: {:?}", e);
    }
}
