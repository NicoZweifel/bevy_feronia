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
        controlled: bool,
        aabb: Aabb,
        debug_color: Color,
        debug: bool,
    ) -> ExtendedWindAffectedMaterial {
        ExtendedMaterial {
            base: base.unwrap_or_default(),
            extension: WindAffectedExtension {
                noise_texture,
                wind,
                controlled,
                aabb,
                debug_color,
                debug,
            },
        }
    }

    fn update_material(mut materials: ResMut<Assets<ExtendedWindAffectedMaterial>>, wind: Wind) {
        for (_, material) in materials
            .iter_mut()
            .filter(|(_, x)| !x.extension.controlled)
        {
            let ext = &mut material.extension;
            ext.wind = wind.clone();
        }
    }

    fn component(material: Handle<ExtendedWindAffectedMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }
}
