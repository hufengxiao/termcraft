#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    PlayerPos { id: u32, x: f64, y: f64, z: f64, yaw: f64, pitch: f64 },
    BlockUpdate { x: i32, y: i32, z: i32, block: crate::block::BlockType },
    PlayerJoin { id: u32, name: String },
    PlayerLeave { id: u32 },
    Chat { id: u32, message: String },
}

const DEFAULT_PORT: u16 = 7878;

pub struct GameServer {
    socket: UdpSocket,
    players: Vec<(u32, SocketAddr)>,
    next_id: u32,
}

impl GameServer {
    pub async fn new(port: Option<u16>) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("0.0.0.0:{}", port.unwrap_or(DEFAULT_PORT));
        let socket = UdpSocket::bind(&addr).await?;
        println!("🎮 TermCraft server on {}", addr);
        Ok(Self { socket, players: Vec::new(), next_id: 1 })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; 65536];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            if let Ok(msg) = bincode::deserialize::<NetMessage>(&buf[..len]) {
                self.handle(msg, addr).await?;
            }
        }
    }

    async fn handle(&mut self, msg: NetMessage, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        match msg {
            NetMessage::PlayerJoin { name: _, .. } => {
                let id = self.next_id;
                self.next_id += 1;
                self.players.push((id, addr));
                let reply = NetMessage::PlayerJoin { id, name: "Server".into() };
                self.socket.send_to(&bincode::serialize(&reply)?, addr).await?;
            }
            NetMessage::PlayerPos { id, x, y, z, yaw, pitch } => {
                let msg = NetMessage::PlayerPos { id, x, y, z, yaw, pitch };
                let data = bincode::serialize(&msg)?;
                for &(_, peer) in &self.players {
                    if peer != addr { self.socket.send_to(&data, peer).await?; }
                }
            }
            NetMessage::BlockUpdate { x, y, z, block } => {
                let msg = NetMessage::BlockUpdate { x, y, z, block };
                let data = bincode::serialize(&msg)?;
                for &(_, peer) in &self.players {
                    if peer != addr { self.socket.send_to(&data, peer).await?; }
                }
            }
            NetMessage::Chat { id, message } => {
                let msg = NetMessage::Chat { id, message };
                let data = bincode::serialize(&msg)?;
                for &(_, peer) in &self.players {
                    if peer != addr { self.socket.send_to(&data, peer).await?; }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct GameClient {
    socket: UdpSocket,
    server_addr: SocketAddr,
    pub player_id: u32,
    pub remote_players: Vec<(u32, f64, f64, f64)>,
}

impl GameClient {
    pub async fn connect(server: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let server_addr: SocketAddr = server.parse()?;
        let join = NetMessage::PlayerJoin { id: 0, name: whoami::username() };
        socket.send_to(&bincode::serialize(&join)?, server_addr).await?;
        let mut buf = vec![0u8; 65536];
        let (len, _) = socket.recv_from(&mut buf).await?;
        let reply: NetMessage = bincode::deserialize(&buf[..len])?;
        let player_id = match reply { NetMessage::PlayerJoin { id, .. } => id, _ => 1 };
        Ok(Self { socket, server_addr, player_id, remote_players: Vec::new() })
    }

    pub async fn send_position(&self, x: f64, y: f64, z: f64, yaw: f64, pitch: f64) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetMessage::PlayerPos { id: self.player_id, x, y, z, yaw, pitch };
        self.socket.send_to(&bincode::serialize(&msg)?, self.server_addr).await?;
        Ok(())
    }

    pub async fn send_block_update(&self, x: i32, y: i32, z: i32, block: crate::block::BlockType) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetMessage::BlockUpdate { x, y, z, block };
        self.socket.send_to(&bincode::serialize(&msg)?, self.server_addr).await?;
        Ok(())
    }

    pub fn poll(&mut self) -> Vec<NetMessage> {
        let mut messages = Vec::new();
        let mut buf = vec![0u8; 65536];
        loop {
            match self.socket.try_recv_from(&mut buf) {
                Ok((len, _)) => {
                    if let Ok(msg) = bincode::deserialize::<NetMessage>(&buf[..len]) {
                        if let NetMessage::PlayerPos { id, x, y, z, .. } = &msg {
                            if *id != self.player_id {
                                if let Some(p) = self.remote_players.iter_mut().find(|p| p.0 == *id) {
                                    *p = (*id, *x, *y, *z);
                                } else {
                                    self.remote_players.push((*id, *x, *y, *z));
                                }
                            }
                        }
                        messages.push(msg);
                    }
                }
                Err(_) => break,
            }
        }
        messages
    }
}
