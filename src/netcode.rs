use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier3d::{
    prelude::{RapierContext, Sleeping},
    dynamics::Velocity,
};
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
            .insert_resource(PeerFrames::default())
            .add_system(wait_for_players
                        .run_if(in_state(GameState::Matchmaking)))
            .add_system(handle_messages)
            .add_system(send_inputs.run_if(in_state(GameState::Ready)))
            .add_system(broadcast_state
                        .in_schedule(CoreSchedule::FixedUpdate)
                        .run_if(in_state(GameState::Ready)))
            .add_system(increment_frame
                        .run_if(in_state(GameState::Ready)));
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FrameNumber(u16);

impl FrameNumber {
    pub fn is_after(x: Self, y: Self) -> bool {
        (x.0 - y.0) < 1024
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct PeerFrames {
    pub frames: HashMap<Peer, FrameNumber>,
}

#[derive(
    Clone, Debug,
    PartialEq, Eq, PartialOrd, Ord, Hash,
    Component, Serialize, Deserialize
)]
pub struct Peer {
    pub id: String
}

#[derive(Resource)]
pub struct ClientInputBuffer {
    buffer: HashMap<Peer, HashMap<FrameNumber, ClientInput>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInput {
    input: crate::fps_controller::FpsControllerInput,
    frame: FrameNumber,
    // inputs: Vec<(FrameNumber, crate::fps_controller::FpsControllerInput)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerState {
    player_transforms: Vec<Transform>,
    frame: FrameNumber,
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

    if info.room_size == 1 {
        server_name.name = Some(info.id());
        server_name.is_self = true;
        println!("Singleplayer mode, server is self");
        state.set(crate::assets::GameState::Ready);
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

pub fn increment_frame(
    session: Res<Session>,
    mut frames: ResMut<PeerFrames>,
) {
    let Some(info) = &session.info else {
        return;
    };

    let new = FrameNumber(
        if let Some(old) = frames.frames.get(&info.id()) {
            old.0.wrapping_add(1)
        } else {
            0
        });
    frames.frames.insert(info.id(), new);
}

pub fn handle_messages(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut server_name: ResMut<ServerName>,
    mut fps_controller_inputs: Query<(&Peer, &mut FpsControllerInput)>,
    mut frames: ResMut<PeerFrames>,
    mut rapier_context: ResMut<RapierContext>,
    mut logical_players: Query<(Entity, &mut Transform, Option<&Peer>), With<LogicalPlayer>>,
) {
    let Some(info) = &mut session.info else {
        return;
    };

    let mut inputs = HashMap::<Peer, ClientInput>::new();
    for (peer, message) in info.receive() {
        match message {
            Message::ClientInput(input) => {
                if let Some(current_input) = inputs.get(&peer) {
                    if FrameNumber::is_after(input.frame, current_input.frame) {
                        inputs.insert(peer, input);
                    }
                } else {
                    inputs.insert(peer, input);
                }
            },
            Message::ServerState(state) => {
                if let Some(name) = &server_name.name {
                    if let Some(frame) = frames.frames.get(name) {
                        if FrameNumber::is_after(state.frame, frame.clone()) {
                            // rollback(&state, &info, &mut rapier_context,
                            //          &mut logical_players);
                        }
                    }
                }
            },
        }
    }

    for (peer, mut fps_controller_input) in fps_controller_inputs.iter_mut() {
        if let Some(input) = inputs.get(peer) {
            if FrameNumber::is_after(input.frame,
                                     frames.frames.get(&peer)
                                     .unwrap_or(&FrameNumber(0)).clone()) {
                debug!("Updating input for {:?}", peer);
                *fps_controller_input = input.input.clone();
                frames.frames.insert(peer.clone(), input.frame);
            }
        }
    }
}

pub fn rollback(
    state: &ServerState,
    session_info: &SessionInfo,
    rapier_context: &mut ResMut<RapierContext>,
    logical_players: &mut Query<(Entity, &mut Transform, Option<&Peer>), With<LogicalPlayer>>,
) {
    let mut peers = Vec::new();
    peers.push(session_info.id());
    for peer in session_info.peers() {
        peers.push(peer);
    }
    peers.sort_unstable();

    let mut peer_to_index = HashMap::new();
    for i in 0 .. peers.len() {
        peer_to_index.insert(peers[i].clone(), i);
    }

    for (entity, mut transform, optional_peer) in logical_players.iter_mut() {
        let peer = optional_peer.cloned().unwrap_or(session_info.id());
        let peer_index = peer_to_index.get(&peer).unwrap().clone();
        *transform = state.player_transforms[peer_index];
        let iso = crate::transform_to_iso(&transform, rapier_context.physics_scale());
        {
            let h = rapier_context.entity2body().get(&entity).unwrap().clone();
            rapier_context.bodies.get_mut(h).unwrap().set_position(iso, true);
        }
        {
            let h = rapier_context.entity2collider().get(&entity).unwrap().clone();
            rapier_context.colliders.get_mut(h).unwrap().set_position(iso);
        }
    }

}

pub fn send_inputs(
    mut session: ResMut<Session>,
    server_name: Res<ServerName>,
    fps_controller_input: Query<&FpsControllerInput,
                                (Without<Peer>, Changed<FpsControllerInput>)>,
    frames: Res<PeerFrames>,
) {
    let Some(info) = &mut session.info else {
        return;
    };
    let Ok(input) = fps_controller_input.get_single() else {
        return;
    };

    if let Some(frame) = frames.frames.get(&info.id()) {
        let spam_coefficient = 1;
        for _ in 0 .. spam_coefficient {
            info.broadcast(&Message::ClientInput(ClientInput {
                input: input.clone(),
                frame: frame.clone(),
            }));
        }
    }
}

pub fn broadcast_state(
    mut session: ResMut<Session>,
    server_name: Res<ServerName>,
    players: Query<(&Transform, Option<&Peer>, &LogicalPlayer)>,
    frames: Res<PeerFrames>,
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

    if let Some(frame) = frames.frames.get(&info.id()) {
        let server_state = ServerState {
            player_transforms,
            frame: frame.clone(),
        };

        if !info.peers().is_empty() {
            info!("Broadcasting current state: {:?}", server_state);
        }

        let spam_coefficient = 3;
        for _ in 0 .. spam_coefficient {
            info.broadcast(&Message::ServerState(server_state.clone()));
        }
    }
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
