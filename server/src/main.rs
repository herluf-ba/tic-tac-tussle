use log::trace;
use renet::{
    RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent,
    NETCODE_USER_DATA_BYTES,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

// TicTacTussle converted to utf-8 codes is 84 105 99 84 97 99 84 117 115 115 108 101
// If you add those up you get 1208.
// It is not necessary to do the protocol id like this but it is fun 🤷‍♂️
const PROTOCOL_ID: u64 = 1208;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
}

#[derive(Default)]
struct Lobby {
    players: Vec<u64>,
    game_state: store::GameState,
}

#[derive(Default)]
struct ServerState {
    players: HashMap<u64, Player>,
    lobbies: HashMap<u64, Lobby>,
    // A hashmap for looking up which lobby a client is playing in.
    // This needs to be maintained by the server.
    // IE. players that disconnect and lobbies that end need to be removed
    player_2_lobby: HashMap<u64, u64>,
}

impl ServerState {
    /// Get a mutable reference to the lobby a player is playing in
    fn get_player_lobby_mut(&mut self, player_id: &u64) -> Option<&mut Lobby> {
        match self.player_2_lobby.get(player_id) {
            Some(lobby_id) => self.lobbies.get_mut(lobby_id),
            None => None,
        }
    }

    /// Creates a new lobby
    fn add_lobby(&mut self) -> u64 {
        let lobby_id = 0; // TODO: generate this randomly
        self.lobbies.insert(lobby_id, Lobby::default());
        lobby_id
    }

    /// Adds a player to a lobby if both exist
    fn join_lobby(&mut self, player_id: &u64, lobby_id: &u64) -> Result<(), CannotJoinLobbyReason> {
        if let Some(lobby) = self.lobbies.get_mut(lobby_id) {
            if lobby.players.len() >= 2 {
                return Err(CannotJoinLobbyReason::LobbyIsFull);
            }

            lobby.players.push(*player_id);
            self.player_2_lobby.insert(*player_id, *lobby_id);
            return Ok(());
        }
        return Err(CannotJoinLobbyReason::NoSuchLobby);
    }

    /// Removes a player and cleans up all references to it
    fn remove_player(&mut self, player_id: &u64) -> Option<Player> {
        let player = self.players.remove(player_id);
        if player.is_some() {
            self.player_2_lobby.remove(player_id);
        }

        player
    }

    /// Removes a lobby and cleans up all references to it
    fn remove_lobby(&mut self, lobby_id: &u64) -> Option<Lobby> {
        if let Some(lobby) = self.lobbies.get(&lobby_id) {
            for player_id in lobby.players.iter() {
                self.player_2_lobby.remove(&player_id);
            }
        }

        self.lobbies.remove(&lobby_id)
    }
}

// These are the messages that the client is able to send to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    // The client would like to create a lobby to invite an opponent
    CreateLobby,
    // The client would like to join an existing lobby
    TryJoinLobby { lobby_id: u64 },
    // An event that the client would like to submit to the GameState
    GameEvent { event: store::GameEvent },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EndGameReason {
    OpponentDisconnect,
    GameEnded,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CannotJoinLobbyReason {
    NoSuchLobby,
    LobbyIsFull,
}

// These are the messages that the server is able to send to clients
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    // Tells the client to join a specific lobby
    JoinLobby {
        lobby_id: u64,
    },
    // Tells the client that they can't join a lobby
    CannotJoinLobby {
        lobby_id: u64,
        reason: CannotJoinLobbyReason,
    },
    // Tells the client that the lobby they are in has ended
    EndGame {
        reason: EndGameReason,
    },
    // Tells the client to begin the game with these players in the game
    BeginGame {
        goes_first: u64,
        players: HashMap<u64, Player>,
    },
    // An valid game event that the client should add to their GameState
    GameEvent {
        event: store::GameEvent,
    },
}

impl Player {
    fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&user_data[0..8]);
        let mut len = u64::from_le_bytes(buffer) as usize;
        len = len.min(NETCODE_USER_DATA_BYTES - 8);
        let data = user_data[8..len + 8].to_vec();
        let username = String::from_utf8(data).unwrap();
        Self { name: username }
    }
}

fn main() {
    env_logger::init();

    // TODO: Add HOST AND PORT to env
    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let mut server: RenetServer = RenetServer::new(
        // Pass the current time to renet, so it can use it to order messages
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        // Pass a server configuration specifying that we want to allow usize::MAX clients to connect
        // and that we don't want to authenticate any of them. Everybody is welcome!
        ServerConfig::new(
            usize::MAX,
            PROTOCOL_ID,
            server_addr,
            ServerAuthentication::Unsecure,
        ),
        // Pass the default connection configuration. This will create a reliable, unreliable and blocking channel.
        // We only actually need the reliable one, but we can just not use the other two.
        RenetConnectionConfig::default(),
        UdpSocket::bind(server_addr).unwrap(),
    )
    .unwrap();

    let mut state = ServerState::default();
    let mut last_updated = Instant::now();

    loop {
        // Update server time
        let now = Instant::now();
        server.update(now - last_updated).unwrap();
        last_updated = now;

        // Receive connection events from clients
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected(id, user_data) => {
                    let player = Player::from_user_data(&user_data);
                    state.players.insert(id, player);
                    trace!("Client {} connected.", id)
                }
                ServerEvent::ClientDisconnected(id) => {
                    // Read lobby id before removing anything, as doing so also modifies player_2_lobby
                    let lobby_id = {
                        match state.player_2_lobby.get(&id) {
                            Some(id) => Some(id.clone()),
                            None => None,
                        }
                    };

                    // Remove the player
                    state.remove_player(&id);
                    trace!("Client {} disconnected", id);

                    // In tic tac toe it doesn't make sense to keep playing when one of the players disconnect.
                    // Therefore we shut down the lobby on disconnects.
                    // Note that it might make sense to keep playing in some other game (like Team Fight Tactics for instance).
                    if let Some(lobby_id) = lobby_id {
                        if let Some(lobby) = state.remove_lobby(&lobby_id) {
                            for player_id in lobby.players.iter() {
                                let message = bincode::serialize(&ServerMessage::EndGame {
                                    reason: EndGameReason::OpponentDisconnect,
                                })
                                .unwrap();
                                server.send_message(*player_id, 0, message);
                            }
                            trace!("Lobby {} ended", lobby_id);
                        }
                    }
                }
            }
        }

        // Receive ClientMessages from clients
        for client_id in server.clients_id().into_iter() {
            while let Some(message) = server.receive_message(client_id, 0) {
                if let Ok(message) = bincode::deserialize::<ClientMessage>(&message) {
                    match message {
                        ClientMessage::CreateLobby => {
                            let lobby_id = state.add_lobby();
                            if state.join_lobby(&lobby_id, &client_id).is_ok() {
                                let message =
                                    bincode::serialize(&ServerMessage::JoinLobby { lobby_id })
                                        .unwrap();
                                server.send_message(client_id, 0, message);
                            }
                        }
                        ClientMessage::TryJoinLobby { lobby_id } => {
                            if let Err(reason) = state.join_lobby(&client_id, &lobby_id) {
                                let message = bincode::serialize(&ServerMessage::CannotJoinLobby {
                                    lobby_id,
                                    reason,
                                })
                                .unwrap();
                                server.send_message(client_id, 0, message);
                                continue;
                            }

                            let message =
                                bincode::serialize(&ServerMessage::JoinLobby { lobby_id }).unwrap();
                            server.send_message(client_id, 0, message);
                            // The game can begin now, since once a single player has joined there must be two in total 👍
                            // TODO: Begin game
                        }
                        ClientMessage::GameEvent { event } => {
                            if let Some(lobby) = state.get_player_lobby_mut(&client_id) {
                                if event.is_valid_on(&lobby.game_state) {
                                    lobby.game_state.dispatch(&event);

                                    let server_event = ServerMessage::GameEvent { event };
                                    for player_id in lobby.players.iter() {
                                        server.send_message(
                                            *player_id,
                                            0,
                                            bincode::serialize(&server_event).unwrap(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        server.send_packets().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}
