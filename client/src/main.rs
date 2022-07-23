use renet::{
    ClientAuthentication, RenetClient, RenetConnectionConfig, RenetServer, ServerAuthentication,
    ServerConfig, ServerEvent, NETCODE_USER_DATA_BYTES,
};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{
    net::{SocketAddr, UdpSocket},
    time::Instant,
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::SystemTime,
};

// Helper struct to pass an username in user data inside the ConnectToken
struct Username(String);

impl Username {
    fn to_netcode_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        if self.0.len() > NETCODE_USER_DATA_BYTES - 8 {
            panic!("Username is too big");
        }
        user_data[0..8].copy_from_slice(&(self.0.len() as u64).to_le_bytes());
        user_data[8..self.0.len() + 8].copy_from_slice(self.0.as_bytes());

        user_data
    }

    fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&user_data[0..8]);
        let mut len = u64::from_le_bytes(buffer) as usize;
        len = len.min(NETCODE_USER_DATA_BYTES - 8);
        let data = user_data[8..len + 8].to_vec();
        let username = String::from_utf8(data).unwrap();
        Self(username)
    }
}

fn main() {
    println!("Usage: [USER_NAME]");
    let args: Vec<String> = std::env::args().collect();
    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let username = Username(args[1].clone());
    client(server_addr, username);
}

const PROTOCOL_ID: u64 = 7;

fn server(addr: SocketAddr) {
    let socket = UdpSocket::bind(addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config = ServerConfig::new(64, PROTOCOL_ID, addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let mut server: RenetServer =
        RenetServer::new(current_time, server_config, connection_config, socket).unwrap();

    let mut usernames: HashMap<u64, String> = HashMap::new();
    let mut received_messages = vec![];
    let mut last_updated = Instant::now();

    loop {
        let now = Instant::now();
        server.update(now - last_updated).unwrap();
        last_updated = now;
        received_messages.clear();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected(id, user_data) => {
                    let username = Username::from_user_data(&user_data);
                    usernames.insert(id, username.0);
                    println!("Client {} connected.", id)
                }
                ServerEvent::ClientDisconnected(id) => {
                    println!("Client {} disconnected", id);
                    usernames.remove_entry(&id);
                }
            }
        }

        for client_id in server.clients_id().into_iter() {
            while let Some(message) = server.receive_message(client_id, 0) {
                let text = String::from_utf8(message).unwrap();
                let username = usernames.get(&client_id).unwrap();
                println!("Client {} ({}) sent text: {}", username, client_id, text);
                let text = format!("{}: {}", username, text);
                received_messages.push(text);
            }
        }

        for text in received_messages.iter() {
            server.broadcast_message(0, text.as_bytes().to_vec());
        }

        server.send_packets().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}

fn client(server_addr: SocketAddr, username: Username) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = RenetConnectionConfig::default();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        server_addr,
        client_id,
        user_data: Some(username.to_netcode_user_data()),
        protocol_id: PROTOCOL_ID,
    };
    let mut client = RenetClient::new(
        current_time,
        socket,
        client_id,
        connection_config,
        authentication,
    )
    .unwrap();
    let stdin_channel = spawn_stdin_channel();

    let mut last_updated = Instant::now();
    loop {
        let now = Instant::now();
        client.update(now - last_updated).unwrap();
        last_updated = now;
        if client.is_connected() {
            match stdin_channel.try_recv() {
                Ok(text) => client.send_message(0, text.as_bytes().to_vec()),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }

            while let Some(text) = client.receive_message(0) {
                let text = String::from_utf8(text).unwrap();
                println!("{}", text);
            }
        }

        client.send_packets().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer.trim_end().to_string()).unwrap();
    });
    rx
}

// use bevy::prelude::*;
// use bevy_renet::{
//     renet::{
//         ConnectToken, RenetClient, RenetConnectionConfig, RenetError, NETCODE_KEY_BYTES,
//         NETCODE_USER_DATA_BYTES,
//     },
//     RenetClientPlugin,
// };
// use store::{GameEvent, GameState};

// use std::net::UdpSocket;
// use std::time::SystemTime;

// const PRIVATE_KEY: &[u8; NETCODE_KEY_BYTES] = b"an example very very secret key."; // 32-bytes
// const PROTOCOL_ID: u64 = 7;

// fn name_to_netcode_user_data(name: &str) -> [u8; NETCODE_USER_DATA_BYTES] {
//     let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
//     if name.len() > NETCODE_USER_DATA_BYTES - 8 {
//         panic!("Username is too big");
//     }
//     user_data[0..8].copy_from_slice(&(name.len() as u64).to_le_bytes());
//     user_data[8..name.len() + 8].copy_from_slice(name.as_bytes());

//     user_data
// }

// fn new_renet_client(name: &str) -> RenetClient {
//     let server_addr = "127.0.0.1:5000".parse().unwrap();
//     let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
//     let connection_config = RenetConnectionConfig::default();
//     let current_time = SystemTime::now()
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .unwrap();
//     let client_id = current_time.as_millis() as u64;

//     // This connect token should come from another system, NOT generated from the client.
//     // Usually from a matchmaking system
//     // The client should not have access to the PRIVATE_KEY from the server.
//     let token = ConnectToken::generate(
//         current_time,
//         PROTOCOL_ID,
//         300,
//         client_id,
//         15,
//         vec![server_addr],
//         Some(&name_to_netcode_user_data(name)),
//         PRIVATE_KEY,
//     )
//     .unwrap();
//     RenetClient::new(current_time, socket, client_id, token, connection_config).unwrap()
// }

// fn name_to_id(name: &str) -> u64 {
//     let str_bytes = name.clone().as_bytes();
//     let mut bytes: [u8; 8] = [0; 8];
//     for i in 0..str_bytes.len().min(8) {
//         bytes[i] = str_bytes[i];
//     }
//     u64::from_ne_bytes(bytes.try_into().unwrap())
// }

// fn main() {
//     let args = std::env::args().collect::<Vec<String>>();
//     let username = &args[1];

//     // APP CONFIG
//     let mut app = App::new();
//     app.insert_resource(WindowDescriptor {
//         title: format!("TicTacTussle <{}>", username),
//         width: 480.0,
//         height: 480.0,
//         ..Default::default()
//     });
//     app.insert_resource(ClearColor(Color::hex("036D5C").unwrap()));
//     app.add_plugins(DefaultPlugins);

//     let mut game_state = GameState::default();
//     app.insert_resource(game_state);
//     app.add_plugin(RenetClientPlugin);
//     app.insert_resource(new_renet_client(&username));
//     app.add_system(panic_on_error);

//     // GAME STATE CONFIG
//     app.add_event::<GameEvent>();
//     app.init_resource::<GameEventClient>();
//     // app.add_system_to_stage(
//     //     CoreStage::PreUpdate,
//     //     game_event_client::dispatch_incoming_events,
//     // );
//     // app.add_system_to_stage(
//     //     CoreStage::PostUpdate,
//     //     game_event_client::offload_outgoing_events,
//     // );

//     app.run();
// }

// fn debug_log_game_state(input: Res<Input<KeyCode>>, game_state: Res<GameState>) {
//     if input.just_pressed(KeyCode::D) {
//         info!("{:#?}", game_state);
//     }
// }

// // If any error is found we just panic
// fn panic_on_error(mut renet_error: EventReader<RenetError>) {
//     for e in renet_error.iter() {
//         panic!("{}", e);
//     }
// }
