use crate::asset::resources::ScatterAssetManager;
use crate::prelude::*;

use bevy_asset::{AssetEvent, Assets};
use bevy_ecs::prelude::*;
use bevy_pbr::StandardMaterial;

#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;

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
    mut asset_manager: ResMut<ScatterAssetManager<T>>,
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
            .iter()
            .map(|part| {
                part.clone().into_scatter_material_part(
                    &materials_in,
                    &mut materials_out,
                    &wind_noise_texture,
                )
            })
            .collect::<Vec<_>>();

        create_asset(
            &mut cmd,
            entity,
            *layer,
            parts,
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
            &mut scatter_assets,
            &mut asset_manager,
        );
    }
}

/// System that processes [`ScatterAssetCreationRequest<T>`] where the in and out materials are a [`StandardMaterial`].
///
/// Clones the handle and re-uses the material.
/// Typically used to scatter assets like rocks or other static entities that don't need a new material.
pub fn process_standard_requests(
    mut cmd: Commands,
    requests: Query<(Entity, &ScatterAssetCreationRequest)>,
    mut scatter_assets: ResMut<Assets<ScatterAsset>>,
    mut asset_manager: ResMut<ScatterAssetManager<StandardMaterial>>,
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
        create_asset(
            &mut cmd,
            entity,
            *layer,
            parts.clone(),
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
            &mut scatter_assets,
            &mut asset_manager,
        );
    }
}

/// Helper to deduplicate the final steps of creating and registering a ScatterAsset.
fn create_asset<T: ScatterMaterial>(
    cmd: &mut Commands,
    asset_entity: Entity,
    layer: Entity,
    parts: Vec<ScatterAssetPartEntity<T>>,
    properties: ScatterAssetProperties,
    #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    scatter_assets: &mut Assets<ScatterAsset<T>>,
    asset_manager: &mut ScatterAssetManager<T>,
) {
    let asset = ScatterAsset::new(
        parts.iter().map(|x| x.part.clone()).collect(),
        properties,
        #[cfg(feature = "avian")]
        rigid_body,
    );

    let h_scatter_asset = scatter_assets.add(asset);

    asset_manager
        .asset_to_layer
        .insert(h_scatter_asset.id(), layer);

    asset_manager.asset_to_entity.insert(
        h_scatter_asset.id(),
        (
            asset_entity,
            parts
                .iter()
                .map(|ScatterAssetPartEntity { entity, .. }| entity.clone())
                .collect(),
        ),
    );

    cmd.entity(asset_entity)
        .remove::<ScatterAssetCreationRequest<T>>();

    for ScatterAssetPartEntity { part, entity } in parts {
        cmd.entity(entity).insert((
            ScatterItem,
            ScatterItemAsset::<T>(h_scatter_asset.clone()),
            part.properties.lod,
            ScatterItemOf(layer),
            ScatterLayerChildProcessed,
        ));
    }
}

/// Listens for Asset Removed events to clean up the manager.
pub fn manage_asset_lifecycle<T>(
    mut asset_events: MessageReader<AssetEvent<ScatterAsset<T>>>,
    mut asset_manager: ResMut<ScatterAssetManager<T>>,
) where
    T: ScatterMaterialAsset,
{
    for event in asset_events.read() {
        if let AssetEvent::Removed { id } = event {
            asset_manager.asset_to_layer.remove(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy_asset::AssetEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            MaterialPlugin::<StandardMaterial>::default(),
        ))
        .init_asset::<ScatterAsset<StandardMaterial>>()
        .init_resource::<ScatterAssetManager<StandardMaterial>>();

        app
    }

    #[test]
    fn test_manage_asset_lifecycle_should_remove_entry() {
        // Arrange
        let mut app = setup_app();
        app.add_systems(Update, manage_asset_lifecycle::<StandardMaterial>);

        let layer_entity = app.world_mut().spawn_empty().id();
        let asset_handle = app
            .world_mut()
            .resource_mut::<Assets<ScatterAsset>>()
            .add(ScatterAsset::default());
        let asset_id = asset_handle.id();

        app.world_mut()
            .resource_mut::<ScatterAssetManager<StandardMaterial>>()
            .asset_to_layer
            .insert(asset_id, layer_entity);

        // Act
        app.world_mut()
            .write_message(AssetEvent::Removed { id: asset_id });
        app.update();

        // Assert
        let manager = app
            .world()
            .resource::<ScatterAssetManager<StandardMaterial>>();
        
        assert!(
            !manager.asset_to_layer.contains_key(&asset_id),
            "The asset ID should be removed from the manager after the AssetEvent::Removed"
        );
    }
}
