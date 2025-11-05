use super::components::{Server, Monitor};

pub fn pre_build(name: String, host: [u8; 4], port: u16) -> Result<(Server, Monitor), Box<dyn std::error::Error>> {
    let server = Server::new(name, host, port)?;
    let monitor = Monitor::new();
    Ok((server, monitor))
}
