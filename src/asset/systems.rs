use crate::prelude::*;

use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_pbr::StandardMaterial;

/// System that processes [`ScatterAssetCreationRequest<T>`] where the out material uses a standard material as a base.
///
/// Creates a *new* `T` material for each asset part.
pub fn process_requests<T>(
    mut cmd: Commands,
    q_requests: Query<(Entity, &ScatterAssetCreationRequest<T>)>,
    materials_in: Res<Assets<StandardMaterial>>,
    mut materials_out: ResMut<Assets<T>>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<T>>>,
    wind_noise_texture: Res<WindTexture>,
) where
    T: ScatterMaterial,
{
    for (
        entity,
        ScatterAssetCreationRequest {
            parts,
            properties,
            #[cfg(feature = "avian")]
            o_rigid_body,
            layer,
            ..
        },
    ) in &q_requests
    {
        let parts = parts
            .into_iter()
            .map(|part| {
                part.clone().into_scatter_material_part::<T>(
                    &materials_in,
                    &mut materials_out,
                    &wind_noise_texture,
                )
            })
            .collect::<Vec<_>>();

        let asset = ScatterAsset::new(
            parts.clone(),
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
        );
        let h_scatter_asset = scatter_assets.add(asset);

        cmd.entity(entity)
            .remove::<ScatterAssetCreationRequest<T>>();

        for part in parts {
            part.insert_bundle(&mut cmd, entity, h_scatter_asset.clone(), *layer);
        }
    }
}

/// System that processes [`ScatterAssetCreationRequest<T>`] where the in and out materials are a [`StandardMaterial`].
///
/// Clones the handle and re-uses the material.
/// Typically used to scatter assets like rocks or other static entities that don't need a new material.
///
/// NOTE: Does not apply the global wind or other shader-related modifiers, it uses the material in its original state.
pub fn process_standard_requests(
    mut cmd: Commands,
    requests: Query<(Entity, &ScatterAssetCreationRequest)>,
    mut scatter_assets: ResMut<Assets<ScatterAsset>>,
) {
    for (
        entity,
        ScatterAssetCreationRequest {
            parts,
            properties,
            layer,
            #[cfg(feature = "avian")]
            o_rigid_body,
            ..
        },
    ) in &requests
    {
        let asset = ScatterAsset::new(
            parts.clone(),
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
        );

        let h_scatter_asset = scatter_assets.add(asset);

        cmd.entity(entity).remove::<ScatterAssetCreationRequest>();

        for part in parts {
            part.insert_bundle(&mut cmd, entity, h_scatter_asset.clone(), *layer);
        }
    }
}
