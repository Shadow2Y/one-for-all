use log::info;
use std::io::Error;

mod executor;

pub fn handle_lifecycle(lifecycle: &str, args: Vec<String>) -> Result<(), Error> {
    info!(
        "Handling lifecycle for :: {} :: with args :: {:?}",
        lifecycle, args
    );

    executor::Ok(())
}
