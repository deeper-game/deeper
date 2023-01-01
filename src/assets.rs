use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Ready)
                    .with_collection::<ImageAssets>())
            .add_state(GameState::Loading);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GameState { Loading, Ready }

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "empty.png")]
    pub empty: Handle<Image>,
    #[asset(path = "crosshair.png")]
    pub crosshair: Handle<Image>,
    #[asset(path = "coin.png")]
    pub coin: Handle<Image>,
}
