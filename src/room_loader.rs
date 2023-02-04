use anyhow::Result;
use std::io::Cursor;
use thiserror::Error;

use bevy::{
    asset::{AddAsset, AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};

pub struct TxtPlugin;

impl Plugin for TxtPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<TextFile>()
            .init_asset_loader::<TxtLoader>();
    }
}

#[derive(Default)]
struct TxtLoader;

impl AssetLoader for TxtLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { Ok(load_txt(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["txt"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
enum TxtError {
    #[error("Failed to decode UTF-8")]
    Utf8(#[from] std::str::Utf8Error),
}

#[derive(TypeUuid)]
#[uuid = "0b551801-d092-44df-a4fb-1d24ff9ef499"]
pub struct TextFile {
    pub contents: String,
}

async fn load_txt<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), TxtError> {
    let text_file = TextFile {
        contents: std::str::from_utf8(bytes)?.to_string(),
    };

    load_context.set_default_asset(LoadedAsset::new(text_file));

    Ok(())
}
