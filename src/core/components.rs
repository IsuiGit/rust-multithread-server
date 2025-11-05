use uuid::Uuid;
use std::fmt;
use std::net::{TcpListener, TcpStream, Ipv4Addr};

pub struct Server{
    id: Uuid,
    name: String,
    host: [u8; 4],
    port: u16,
    listener: TcpListener
}

impl Server{
    pub fn new(name: String, host: [u8; 4], port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id: Uuid::new_v4(),
            name: name,
            host: host,
            port: port,
            listener: TcpListener::bind(format!("{}.{}.{}.{}:{}", host[0], host[1], host[2], host[3], port))?,
        })
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Server info:\n\tid: {}\n\tname: {}\n\thost: {}\n\tport: {}", self.id, self.name, Ipv4Addr::from(self.host).to_string(), self.port)
    }
}

pub struct Monitor{
    id: Uuid
}

impl Monitor{
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4()
        }
    }
}

impl fmt::Display for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Monitor info:\n\tid: {}", self.id)
    }
}
