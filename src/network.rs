#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    /// Player position update
    PlayerPos {
        id: u32,
        x: f64,
        y: f64,
        z: f64,
        yaw: f64,
        pitch: f64,
    },
    /// Block modification
    BlockUpdate {
        x: i32,
        y: i32,
        z: i32,
        block: crate::block::BlockType,
    },
    /// New player joined
    PlayerJoin {
        id: u32,
        name: String,
    },
    /// Player left
    PlayerLeave {
        id: u32,
    },
    /// Server full state sync
    WorldChunk {
        cx: i32,
        cz: i32,
        data: Vec<u8>, // compressed chunk data
    },
}

const DEFAULT_PORT: u16 = 7878;
const TICK_RATE_MS: u64 = 50; // 20 updates/sec

/// Game server
pub struct GameServer {
    socket: UdpSocket,
    players: Vec<(u32, SocketAddr)>,
    next_id: u32,
}

impl GameServer {
    pub async fn new(port: Option<u16>) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("0.0.0.0:{}", port.unwrap_or(DEFAULT_PORT));
        let socket = UdpSocket::bind(&addr).await?;
        println!("🎮 TermCraft server listening on {}", addr);
        Ok(Self {
            socket,
            players: Vec::new(),
            next_id: 1,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; 65536];
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(TICK_RATE_MS),
        );

        loop {
            tokio::select! {
                result = self.socket.recv_from(&mut buf) => {
                    let (len, addr) = result?;
                    if let Ok(msg) = bincode::deserialize::<NetMessage>(&buf[..len]) {
                        self.handle_message(msg, addr).await?;
                    }
                }
                _ = interval.tick() => {
                    // Periodic server tick (cleanup, etc.)
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: NetMessage,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match msg {
            NetMessage::PlayerJoin { name, .. } => {
                let id = self.next_id;
                self.next_id += 1;
                self.players.push((id, addr));
                println!("✅ {} joined as player #{}", name, id);

                // Send join confirmation
                let reply = NetMessage::PlayerJoin { id, name: "Server".into() };
                let data = bincode::serialize(&reply)?;
                self.socket.send_to(&data, addr).await?;
            }
            NetMessage::PlayerPos { id, x, y, z, yaw, pitch } => {
                // Broadcast to all other players
                let msg = NetMessage::PlayerPos { id, x, y, z, yaw, pitch };
                let data = bincode::serialize(&msg)?;
                for &(_, peer_addr) in &self.players {
                    if peer_addr != addr {
                        self.socket.send_to(&data, peer_addr).await?;
                    }
                }
            }
            NetMessage::BlockUpdate { x, y, z, block } => {
                // Broadcast block change to all
                let msg = NetMessage::BlockUpdate { x, y, z, block };
                let data = bincode::serialize(&msg)?;
                for &(_, peer_addr) in &self.players {
                    if peer_addr != addr {
                        self.socket.send_to(&data, peer_addr).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Game client
pub struct GameClient {
    socket: UdpSocket,
    server_addr: SocketAddr,
    pub player_id: u32,
    pub remote_players: Vec<(u32, f64, f64, f64)>, // id, x, y, z
}

impl GameClient {
    pub async fn connect(server: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let server_addr: SocketAddr = server.parse()?;

        // Send join message
        let join = NetMessage::PlayerJoin {
            id: 0,
            name: whoami::username(),
        };
        let data = bincode::serialize(&join)?;
        socket.send_to(&data, server_addr).await?;

        // Wait for confirmation
        let mut buf = vec![0u8; 65536];
        let (len, _) = socket.recv_from(&mut buf).await?;
        let reply: NetMessage = bincode::deserialize(&buf[..len])?;
        let player_id = match reply {
            NetMessage::PlayerJoin { id, .. } => id,
            _ => 1,
        };

        println!("✅ Connected as player #{}", player_id);

        Ok(Self {
            socket,
            server_addr,
            player_id,
            remote_players: Vec::new(),
        })
    }

    /// Send position update to server
    pub async fn send_position(
        &self,
        x: f64, y: f64, z: f64, yaw: f64, pitch: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetMessage::PlayerPos {
            id: self.player_id,
            x, y, z, yaw, pitch,
        };
        let data = bincode::serialize(&msg)?;
        self.socket.send_to(&data, self.server_addr).await?;
        Ok(())
    }

    /// Send block update to server
    pub async fn send_block_update(
        &self,
        x: i32, y: i32, z: i32,
        block: crate::block::BlockType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetMessage::BlockUpdate { x, y, z, block };
        let data = bincode::serialize(&msg)?;
        self.socket.send_to(&data, self.server_addr).await?;
        Ok(())
    }

    /// Poll for incoming messages (non-blocking)
    pub async fn poll(&mut self) -> Vec<NetMessage> {
        let mut messages = Vec::new();
        let mut buf = vec![0u8; 65536];

        loop {
            match self.socket.try_recv_from(&mut buf) {
                Ok((len, _)) => {
                    if let Ok(msg) = bincode::deserialize::<NetMessage>(&buf[..len]) {
                        // Update remote player positions
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
