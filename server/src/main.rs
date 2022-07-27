use log::{info, trace, warn};
use renet::{
    RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent,
    NETCODE_USER_DATA_BYTES,
};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

// TicTacTussle converted to utf-8 codes is 84 105 99 84 97 99 84 117 115 115 108 101
// If you add those up you get 1208.
// It is not necessary to do the protocol id like this but it is fun 🤷‍♂️
const PROTOCOL_ID: u64 = 1208;

/// Utility function for extracting a players name from renet user data
fn name_from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> String {
    let mut buffer = [0u8; 8];
    buffer.copy_from_slice(&user_data[0..8]);
    let mut len = u64::from_le_bytes(buffer) as usize;
    len = len.min(NETCODE_USER_DATA_BYTES - 8);
    let data = user_data[8..len + 8].to_vec();
    String::from_utf8(data).unwrap()
}

fn main() {
    env_logger::init();

    let server_addr: SocketAddr = format!("{}:{}", env!("HOST"), env!("PORT"))
        .parse()
        .unwrap();
    let mut server: RenetServer = RenetServer::new(
        // Pass the current time to renet, so it can use it to order messages
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        // Pass a server configuration specifying that we want to allow only 2 clients to connect
        // and that we don't want to authenticate them. Everybody is welcome!
        ServerConfig::new(2, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure),
        // Pass the default connection configuration. This will create a reliable, unreliable and blocking channel.
        // We only actually need the reliable one, but we can just not use the other two.
        RenetConnectionConfig::default(),
        UdpSocket::bind(server_addr).unwrap(),
    )
    .unwrap();

    trace!("🕹  TicTacTussle server listening on {}", server_addr);

    let mut game_state = store::GameState::default();
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
                    // Add the new player to the game
                    let event = store::GameEvent::PlayerJoined {
                        player_id: id,
                        name: name_from_user_data(&user_data),
                    };
                    game_state.dispatch(&event);

                    // Tell all other players that a new player has joined
                    for player_id in game_state.players.keys() {
                        server.send_message(*player_id, 0, bincode::serialize(&event).unwrap());
                    }

                    info!("Client {} connected.", id);
                    // In TicTacTussle the game can begin once two players has joined
                    if game_state.players.len() == 2 {
                        let event = store::GameEvent::BeginGame { goes_first: id };
                        game_state.dispatch(&event);
                        server.broadcast_message(0, bincode::serialize(&event).unwrap());
                        trace!("The game gas begun");
                    }
                }
                ServerEvent::ClientDisconnected(id) => {
                    // First dispatch a disconnect event
                    let event = store::GameEvent::PlayerDisconnected { player_id: id };
                    game_state.dispatch(&event);
                    server.broadcast_message(0, bincode::serialize(&event).unwrap());
                    info!("Client {} disconnected", id);

                    // Then end the game, since tic tac toe can't go on with a single player
                    let event = store::GameEvent::PlayerDisconnected { player_id: id };
                    game_state.dispatch(&event);
                    server.broadcast_message(0, bincode::serialize(&event).unwrap());

                    // NOTE: Since we don't authenticate users we can't do any reconnection attempts.
                    // We simply have no way to know if the next user is the same as the one that disconnected.
                }
            }
        }

        // Receive GameEvents from clients. Broadcast valid events.
        for client_id in server.clients_id().into_iter() {
            while let Some(message) = server.receive_message(client_id, 0) {
                if let Ok(event) = bincode::deserialize::<store::GameEvent>(&message) {
                    if event.is_valid_on(&game_state) {
                        game_state.dispatch(&event);
                        trace!("Player {} sent valid event:\n\t{:#?}", client_id, event);
                        server.broadcast_message(0, bincode::serialize(&event).unwrap());
                    } else {
                        warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                    }
                }
            }
        }

        server.send_packets().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}
