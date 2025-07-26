use crate::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::image::Image;
use bevy::pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;

pub type WindAffectedExtendedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

impl WindAffectable<StandardMaterial, WindAffectedExtendedMaterial>
for WindAffectedExtendedMaterial
{
    fn create_material(
        base: StandardMaterial,
        wind: Wind,
        noise_texture: Handle<Image>,
        controlled: bool,
    ) -> WindAffectedExtendedMaterial {
        ExtendedMaterial {
            base,
            extension: WindAffectedExtension {
                noise_texture,
                wind,
                controlled,
            },
        }
    }

    fn update_material(mut materials: ResMut<Assets<WindAffectedExtendedMaterial>>, wind: Wind) {
        for (_, material) in materials
            .iter_mut()
            .filter(|(_, x)| !x.extension.controlled)
        {
            let ext = &mut material.extension;
            ext.wind = wind.clone();
        }
    }

    fn component(material: Handle<WindAffectedExtendedMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }
}
