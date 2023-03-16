use bevy::prelude::*;
use bevy_rapier3d::{prelude::Sleeping, dynamics::Velocity};
use bevy::tasks::IoTaskPool;
use bevy_ggrs::ggrs::{SessionBuilder, Config, PlayerHandle};
use matchbox_socket::WebRtcSocket;

use bevy_ggrs::{GGRSPlugin, PlayerInputs, Rollback, RollbackIdProvider};
use bytemuck::{Pod, Zeroable};

pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Session { info: None });
        GGRSPlugin::<GGRSConfig>::new()
            .with_update_frequency(60)
            .with_input_system(net_input)
            .register_rollback_resource::<PhysicsRollbackState>()
            .register_rollback_component::<crate::fps_controller::FpsControllerInput>()
            .register_rollback_component::<GlobalTransform>()
            .register_rollback_component::<Transform>()
            .register_rollback_component::<Velocity>()
            .register_rollback_component::<Sleeping>()
            .build(app);
    }
}

#[derive(Default, Resource, Reflect, PartialEq, Eq, Hash)]
#[reflect(Hash, Resource, PartialEq)]
pub struct PhysicsRollbackState {
    pub rapier_state: Option<Vec<u8>>,
    pub rapier_checksum: u16,
}

pub struct SessionInfo {
    room_id: String,
    room_size: usize,
    socket: WebRtcSocket,
}

#[derive(Resource)]
pub struct Session {
    info: Option<SessionInfo>
}

pub fn connect(room_id: &str, room_size: usize) -> Session {
    let room_url = format!("ws://127.0.0.1:3536/{}?next={}",
                           room_id, room_size);
    info!("Connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);
    IoTaskPool::get().spawn(message_loop).detach();
    Session {
        info: Some(SessionInfo {
            room_id: room_id.to_string(),
            room_size,
            socket,
        }),
    }
}

pub fn wait_for_players(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut state: ResMut<NextState<crate::assets::GameState>>,
) {
    let Some(info) = &mut session.info else {
        // If there is no socket we've already started the game
        return;
    };

    // Check for new connections
    info.socket.accept_new_connections();
    let players = info.socket.connected_peers();

    if players.len() < info.room_size - 1 {
        return; // wait for more players
    }

    println!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder =
        bevy_ggrs::ggrs::SessionBuilder::<GGRSConfig>::new()
        .with_num_players(info.room_size)
        .with_input_delay(2)
        .with_sparse_saving_mode(false);

    session_builder =
        session_builder.add_player(bevy_ggrs::ggrs::PlayerType::Local, 0)
        .expect("Failed to add local player");
    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(bevy_ggrs::ggrs::PlayerType::Remote(player), i + 1)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = std::mem::take(&mut session.info).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2PSession(ggrs_session));
    state.set(crate::assets::GameState::Ready);
}

use bevy_ggrs::ggrs::Message;

impl bevy_ggrs::ggrs::NonBlockingSocket<String> for SessionInfo {
    fn send_to(&mut self, msg: &Message, addr: &String) {
        let buf = bincode::serialize(&msg).unwrap();
        self.socket.send(buf.into_boxed_slice(), addr);
    }

    fn receive_all_messages(&mut self) -> Vec<(String, Message)> {
        let mut result = Vec::new();
        for (source, bytes) in self.socket.receive() {
            if let Ok(message) = bincode::deserialize(&bytes) {
                result.push((source, message));
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct GGRSConfig;

impl Config for GGRSConfig {
    type Input = NetInput;
    type State = u8;
    type Address = String;
}

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_LEFT: u8 = 1 << 2;
pub const INPUT_RIGHT: u8 = 1 << 3;

fn net_input(_handle: In<PlayerHandle>, keyboard_input: Res<Input<KeyCode>>) -> NetInput {
    let mut input: u8 = 0;

    if keyboard_input.pressed(KeyCode::W) {
        input |= INPUT_UP;
    }
    if keyboard_input.pressed(KeyCode::A) {
        input |= INPUT_LEFT;
    }
    if keyboard_input.pressed(KeyCode::S) {
        input |= INPUT_DOWN;
    }
    if keyboard_input.pressed(KeyCode::D) {
        input |= INPUT_RIGHT;
    }

    NetInput { keys: input }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
pub struct NetInput {
    pub keys: u8,
}
