use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::asset::{Assets, Handle};
use bevy::image::Image;
use bevy::pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

pub type ExtendedWindAffectedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

impl<T, P> WindAffectable<StandardMaterial, ExtendedWindAffectedMaterial, T, P>
    for ExtendedWindAffectedMaterial
where
    T: Resource + ProtoTypes<ExtendedWindAffectedMaterial, P>,
    P: ProtoType<ExtendedWindAffectedMaterial>,
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

    fn spawn(
        mut cmd: Commands,
        trigger: On<ScatterResults>,
        prototypes: &T,
        // TODO use chunks if spawned for chunk
        _q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
        _q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
    ) {
        let Some(prototype) = prototypes.choose() else {
           warn!("No prototype found!");
           return
        };

        cmd.spawn_batch(
            trigger
                .data
                .iter()
                .map(|result| {
                    (
                        Mesh3d(prototype.mesh().clone()),
                        MeshMaterial3d(prototype.material().clone()),
                        **result,
                        WindAffected,
                        WindAffectedReady,
                        ChildOf(trigger.layer),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    fn component(material: Handle<ExtendedWindAffectedMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }
}
