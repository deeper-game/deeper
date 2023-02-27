use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use crate::room_loader::TextFile;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Ready)
                    .with_collection::<ImageAssets>()
                    .with_collection::<RoomAssets>()
                    .with_collection::<FontAssets>())
            .add_state(GameState::Loading);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GameState { Loading, Ready }

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "level.png")]
    pub level: Handle<Image>,
    #[asset(path = "empty.png")]
    pub empty: Handle<Image>,
    #[asset(path = "crosshair.png")]
    pub crosshair: Handle<Image>,
    #[asset(path = "coin.png")]
    pub coin: Handle<Image>,
    #[asset(path = "block-debug.png")]
    pub block_debug: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct RoomAssets {
    #[asset(path = "rooms/room1.txt")]
    pub room1: Handle<TextFile>,
    #[asset(path = "rooms/room2.txt")]
    pub room2: Handle<TextFile>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "DejaVuSans.ttf")]
    pub dejavu_sans: Handle<Font>,
}
