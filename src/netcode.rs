use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier3d::{prelude::Sleeping, dynamics::Velocity};
use bevy::tasks::IoTaskPool;
use matchbox_socket::WebRtcSocket;
use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};

use crate::assets::GameState;
use crate::fps_controller::{FpsControllerInput, LogicalPlayer};

pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Session::default())
            .insert_resource(ServerName::default())
            .insert_resource(NetcodeIdProvider::default())
            .add_system(wait_for_players
                        .run_if(in_state(GameState::Matchmaking)))
            .add_system(handle_messages)
            .add_system(send_inputs.run_if(in_state(GameState::Ready)))
            .add_system(broadcast_state
                        .in_schedule(CoreSchedule::FixedUpdate)
                        .run_if(in_state(GameState::Ready)));
    }
}

#[derive(
    Clone, Debug,
    PartialEq, Eq, PartialOrd, Ord, Hash,
    Component, Serialize, Deserialize
)]
pub struct Peer {
    pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInput {
    input: crate::fps_controller::FpsControllerInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerState {
    player_transforms: Vec<Transform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    ClientInput(ClientInput),
    ServerState(ServerState),
}

pub struct SessionInfo {
    room_id: String,
    room_size: usize,
    socket: WebRtcSocket,
}

impl SessionInfo {
    pub fn room_id(&self) -> &str {
        &self.room_id
    }

    pub fn room_size(&self) -> usize {
        self.room_size
    }

    pub fn id(&self) -> Peer {
        Peer { id: self.socket.id().clone() }
    }

    pub fn send(&mut self, peer: &Peer, message: &Message) {
        debug!("To {} sent {:?}", peer.id, message);
        let buf = bincode::serialize(&message).unwrap();
        self.socket.send(buf.into_boxed_slice(), peer.id.clone());
    }

    pub fn receive(&mut self) -> Vec<(Peer, Message)> {
        let mut result = Vec::new();
        for (source, bytes) in self.socket.receive() {
            if let Ok(message) = bincode::deserialize(&bytes) {
                debug!("From {} received {:?}", source, message);
                result.push((Peer { id: source }, message));
            } else {
                warn!("Failed to parse message from socket");
            }
        }
        result
    }

    pub fn peers(&self) -> Vec<Peer> {
        self.socket.connected_peers()
            .iter().map(|s| Peer { id: s.clone() }).collect()
    }

    pub fn broadcast(&mut self, message: &Message) {
        for peer in &self.peers() {
            self.send(peer, message);
        }
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct ServerName {
    name: Option<Peer>,
    is_self: bool,
}

#[derive(Default, Resource)]
pub struct Session {
    pub info: Option<SessionInfo>
}

pub fn connect(room_id: &str, room_size: usize) -> Session {
    let room_url = format!("ws://deeper.quest:3536/{}?next={}",
                           room_id, room_size);
    info!("Connecting to matchbox server: {:?}", room_url);
    let (mut socket, message_loop) = WebRtcSocket::new(room_url);
    IoTaskPool::get().spawn(message_loop).detach();
    socket.accept_new_connections();
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
    mut server_name: ResMut<ServerName>,
    mut state: ResMut<NextState<crate::assets::GameState>>,
    time: Res<Time>,
) {
    let Some(info) = &mut session.info else {
        return;
    };

    {
        info.socket.accept_new_connections();
        let players = info.socket.connected_peers();

        if players.len() < info.room_size - 1 {
            return;
        }
    }

    let mut peers = info.peers();
    peers.sort_unstable();
    if let Some(peer) = peers.first() {
        server_name.name = Some(std::cmp::min(info.id(), peer.clone()));
        server_name.is_self = info.id() < peer.clone();
        println!("Server name is {:?}, my name is {:?}",
                 server_name, info.id());
        state.set(crate::assets::GameState::Ready);
    }
}

pub fn handle_messages(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut server_name: ResMut<ServerName>,
    mut fps_controller_inputs: Query<(&Peer, &mut FpsControllerInput)>,
) {
    let Some(info) = &mut session.info else {
        return;
    };

    let mut inputs = HashMap::<Peer, ClientInput>::new();
    for (peer, message) in info.receive() {
        match message {
            Message::ClientInput(input) => {
                inputs.insert(peer, input);
            },
            Message::ServerState(state) => {},
        }
    }

    for (peer, mut fps_controller_input) in fps_controller_inputs.iter_mut() {
        if let Some(input) = inputs.get(peer) {
            debug!("Updating input for {:?}", peer);
            *fps_controller_input = input.input.clone();
        }
    }
}

pub fn send_inputs(
    mut session: ResMut<Session>,
    server_name: Res<ServerName>,
    fps_controller_input: Query<&FpsControllerInput,
                                (Without<Peer>, Changed<FpsControllerInput>)>,
) {
    let Some(info) = &mut session.info else {
        return;
    };
    if server_name.is_self {
        return;
    }
    let Some(server) = &server_name.name else {
        return;
    };
    let Ok(input) = fps_controller_input.get_single() else {
        return;
    };

    info.broadcast(&Message::ClientInput(ClientInput {
        input: input.clone(),
    }));
}

pub fn broadcast_state(
    mut session: ResMut<Session>,
    server_name: Res<ServerName>,
    players: Query<(&Transform, Option<&Peer>, &LogicalPlayer)>,
) {
    let Some(info) = &mut session.info else {
        return;
    };
    if !server_name.is_self || server_name.name.is_none() {
        return;
    }

    let mut peers = Vec::new();
    peers.push(info.id());
    for peer in info.peers() {
        peers.push(peer);
    }
    peers.sort_unstable();

    let mut peer_to_index = HashMap::new();
    for i in 0 .. peers.len() {
        peer_to_index.insert(peers[i].clone(), i);
    }

    let mut player_transforms = Vec::new();
    player_transforms.resize(peers.len(), Transform::default());
    for (transform, optional_peer, logical_player) in players.iter() {
        if optional_peer.is_none() {
            assert!(logical_player.0 == 0);
        }
        let me = info.id();
        let peer = optional_peer.unwrap_or(&me);
        let index = peer_to_index.get(peer).unwrap().clone();
        player_transforms[index] = transform.clone();
    }

    let server_state = ServerState {
        player_transforms,
    };

    info!("Broadcasting current state: {:?}", server_state);

    info.broadcast(&Message::ServerState(server_state));
}

#[derive(Component)]
pub struct NetcodeId {
    id: u32,
}

impl NetcodeId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub const fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Resource, Default)]
pub struct NetcodeIdProvider {
    next_id: u32,
}

impl NetcodeIdProvider {
    pub fn next_id(&mut self) -> u32 {
        if self.next_id == u32::MAX {
            panic!("NetcodeIdProvider: u32::MAX has been reached.");
        }
        let ret = self.next_id;
        self.next_id += 1;
        ret
    }

    pub fn next(&mut self) -> NetcodeId {
        NetcodeId::new(self.next_id())
    }
}
