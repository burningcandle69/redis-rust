use crate::resp::{RESPHandler, RESP};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

pub struct Slave {
    pub master: (Ipv4Addr, u16),
    pub resp: RESPHandler,
}

impl Slave {
    pub fn new(addr: Ipv4Addr, port: u16) -> std::io::Result<Self> {
        let tcp = TcpStream::connect(SocketAddrV4::new(addr, port))?;
        Ok(Slave {
            master: (addr, port),
            resp: RESPHandler::new(Box::new(tcp)),
        })
    }
    
    pub fn handle(&mut self) {
        self.handshake().unwrap();
    }
    
    pub fn handshake(&mut self) -> std::io::Result<()> {
        self.ping()?;
        Ok(())
    }

    pub fn ping(&mut self) -> std::io::Result<()> {
        let ping: RESP = vec!["PING".into()].into();
        self.resp.send(ping)?;
        let response = self.resp.next().unwrap().string().unwrap();
        assert_eq!(response.to_lowercase(), "pong");
        Ok(())
    }
}
