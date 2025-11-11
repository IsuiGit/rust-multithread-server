use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io::{Read, BufReader};
use uuid::Uuid;
use std::fmt;

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

    // Non-blocking mode setter
    pub fn unlock(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        self.listener.set_nonblocking(true)?;
        Ok(())
    }

    // Blocking mode setter
    pub fn lock(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        self.listener.set_nonblocking(false)?;
        Ok(())
    }

    // Listener getatr
    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    // for incoming TcpStreams
    pub fn request(&self, stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
        let mut reader = BufReader::new(stream);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    // for outcomming TcpStreams
    fn response() {}
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
