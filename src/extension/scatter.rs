use crate::prelude::*;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use std::borrow::Cow;

pub fn scatter_layer(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<ExtendedWindAffectedMaterial>::default(),
    )
}

impl ScatterMaterial for ExtendedWindAffectedMaterial {
    fn create_material(
        base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> ExtendedWindAffectedMaterial {
        ExtendedMaterial {
            base: base.unwrap_or_default(),
            extension: WindAffectedExtension::new(properties, noise_texture),
        }
    }

    fn update_material(
        material: &mut ExtendedWindAffectedMaterial,
        wind: Wind,
        options: MaterialOptions,
    ) {
        let ext = &mut material.extension;
        ext.wind = wind;
        ext.options = options;
    }

    fn component(material: Handle<ExtendedWindAffectedMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<ExtendedWindAffectedMaterial>) {
        cmd.spawn_batch(
            request
                .spawn_batch_iter()
                .map(|x| (x, WindAffected))
                .collect::<Vec<_>>(),
        );
    }
}
