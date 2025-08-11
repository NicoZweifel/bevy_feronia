use crate::prelude::*;
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
    ) -> ExtendedWindAffectedMaterial {
        ExtendedMaterial {
            base: base.unwrap_or_else(|| StandardMaterial::default()),
            extension: WindAffectedExtension {
                noise_texture,
                wind,
                controlled,
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
