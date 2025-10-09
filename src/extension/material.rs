use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;

pub type ExtendedWindAffectedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

impl<P> WindAffectable<P, StandardMaterial, ExtendedWindAffectedMaterial>
    for ExtendedWindAffectedMaterial
where
    P: ProtoType<ExtendedWindAffectedMaterial> + Asset + Clone,
{
    fn create_material(
        base: Option<StandardMaterial>,
        wind: Wind,
        noise_texture: Handle<Image>,
        aabb: Aabb,
        options: MaterialOptions,
    ) -> ExtendedWindAffectedMaterial {
        ExtendedMaterial {
            base: base.unwrap_or_default(),
            extension: WindAffectedExtension {
                noise_texture,
                wind,
                aabb,
                options,
            },
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
}
