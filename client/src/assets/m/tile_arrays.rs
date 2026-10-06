//! The ground-tile texture arrays the splat shader samples (see
//! `assets::tile_layers` for why arrays, and how each tile became a layer).
//!
//! Built exactly once, as soon as every tile `tile2d.ifo` defines has been
//! loaded (its layer is in [`TerrainTileLayers`]) or has failed to load:
//! four `texture_2d_array`s of up to [`TILE_ARRAY_LAYERS`] layers, layer
//! `id % 256` of array `id / 256` holding tile `id`. Ids without a usable
//! texture get a white layer, as their fallback image was white before. The
//! layer table is drained afterwards; it only held the bytes until now.

use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use super::block_splat_material::TerrainTileAtlas;
use crate::assets::tile_layers::{
    tile_layer_len, white_tile_layer, TerrainTileLayers, TILE_ARRAY_COUNT, TILE_ARRAY_LAYERS,
    TILE_LAYER_MIPS, TILE_LAYER_SIZE,
};

/// The four ground-tile arrays, indexed by `tile_id / 256`.
#[derive(Resource, Clone, ExtractResource)]
pub struct TerrainTileArrays(pub [Handle<Image>; TILE_ARRAY_COUNT]);

pub(super) fn build_tile_arrays(
    mut commands: Commands,
    atlas: Option<Res<TerrainTileAtlas>>,
    layers: Res<TerrainTileLayers>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(atlas) = atlas else {
        return;
    };
    let table = layers.0.read().unwrap();
    // wait until every defined tile has either reported its layer or failed
    for handle in atlas.slots.iter().flatten() {
        let reported = handle.path().is_some_and(|path| table.contains_key(path));
        let settled = matches!(
            asset_server.get_load_state(handle),
            Some(LoadState::Loaded | LoadState::Failed(_))
        );
        if !reported && !settled {
            return;
        }
    }

    let white = white_tile_layer();
    let mut missing = 0usize;
    let arrays = std::array::from_fn(|array| {
        let first = array * TILE_ARRAY_LAYERS as usize;
        let ids = &atlas.slots[first..first + TILE_ARRAY_LAYERS as usize];
        // as many layers as the highest defined id in this range needs
        let count = ids.iter().rposition(Option::is_some).map_or(1, |i| i + 1);
        let mut data = Vec::with_capacity(count * tile_layer_len());
        for slot in &ids[..count] {
            let layer = slot
                .as_ref()
                .and_then(Handle::path)
                .and_then(|path| table.get(path));
            match layer {
                Some(layer) => data.extend_from_slice(layer),
                None => {
                    missing += slot.is_some() as usize;
                    data.extend_from_slice(&white);
                }
            }
        }
        images.add(tile_array_image(count as u32, data))
    });
    drop(table);
    if missing > 0 {
        warn!("{missing} ground tiles have no texture-array layer and render white");
    }
    info!("ground tile arrays built");
    layers.0.write().unwrap().clear();
    commands.insert_resource(TerrainTileArrays(arrays));
}

fn tile_array_image(layers: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new_uninit(
        Extent3d {
            width: TILE_LAYER_SIZE,
            height: TILE_LAYER_SIZE,
            depth_or_array_layers: layers,
        },
        TextureDimension::D2,
        TextureFormat::Bc1RgbaUnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = TILE_LAYER_MIPS;
    // a single-layer array must still be viewed as an array
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    image.data = Some(data);
    image
}
