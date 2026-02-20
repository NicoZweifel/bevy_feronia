# bevy_feronia
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/NicoZweifel/bevy_feronia?tab=readme-ov-file#licensecreditsinspirationsreferences)
[![Crates.io](https://img.shields.io/crates/v/bevy_feronia.svg)](https://crates.io/crates/bevy_feronia)
[![Downloads](https://img.shields.io/crates/d/bevy_feronia.svg)](https://crates.io/crates/bevy_feronia)
[![Docs](https://docs.rs/bevy_feronia/badge.svg)](https://docs.rs/bevy_feronia/)
[![CI](https://github.com/nicozweifel/bevy_feronia/workflows/CI/badge.svg)](https://github.com/NicoZweifel/bevy_feronia/actions)

Environment scattering tools and shaders/materials that prioritize visual fidelity/artistic freedom, a declarative API and modularity.

## Who is this for?

In the current stage this is mostly for tinkerers and learners within the [bevy](https://github.com/bevyengine/bevy) ecosystem, but I am planning to use this for actual game dev myself eventually.

> [!CAUTION]
> This package is in early development and in an experimentation stage.
> I wouldn't personally use this in production quite yet, but it's getting closer to that state incrementally.

<img width="100%" alt="Screenshot 2025-11-28 144933" src="https://github.com/user-attachments/assets/bf0ac5b4-affc-4360-8b3e-7492b5c07257" />

## Getting started

```shell
cargo add bevy_feronia
```

The possible use-cases are demonstrated in the [examples](/examples/EXAMPLES.md)

### Setup

The setup depends on the use-case, but a typical setup would look like something like this:

```rust
app.add_plugins((
    MeshMaterialAssetBackendPlugin,
    // Or 
    SceneAssetBackendPlugin,
    // ...
    ExtendedWindAffectedScatterPlugin
));
```

The Scatter system needs to know when it can set up since it can depend on height mapping. You need to insert the setup state at some point.

> [!NOTE]  
> In complex setups that load assets and bake a height map, this can be after the `Startup`.

```rust
app.insert_state(ScatterState::Setup)
```

Or
```rust
ns_height_map.set(HeightMapState::Setup);
ns_scatter.set(ScatterState::Setup);
```

### Defining layers

A `ScatterItem`'s `LOD`s are grouped by `Name`. If the names end in `LOD_1` or `lod1` etc., the LOD suffix will be stripped from the name to match it to the other lods of the asset.

> [!CAUTION]
> When defining multiple `ScatterItems` per `ScatterLayer` without names, a different asset will render when `LODs` are changing, leading to visual bugs.

```rust
// Landscape
cmd.spawn((
    MeshMaterial3d(materials.add(StandardMaterial {
    base_color: GRAY_500.into(),
        ..default()
    })),
    Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
    ScatterRoot::default(),
    // Scatter layers
    children![(
            // Make sure you use the correct `ScatterLayer` with the desired `ScatterLayerType`, e.g.,
            // Standard, Extended or Instanced Material/Layer.
            extension::scatter_layer("Wind Affected Layer"),
            // Scatter Options
            DistributionDensity(50.),
            InstanceJitter::default(),
            // You can define material options on the full layer here
            WindAffected,
            children![
                (
                    // Or overwrite on the item, e.g., 
                    // WindAffected,
                    //
                    // CAUTION: If you have multiple assets, all lods that belong to each other need to have the same name!
                    //
                    // You can have multiple assets in each layer; as long as all LODs have the same name, they will be matched correctly.
                    Name::new("Wind Affected Example Item"),
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    Mesh3d(mesh.clone()),
                ),
                (
                    Name::new("Wind Affected Example Item"),
                    // We need to specify the LOD Level if it is not 0 (Highest level)
                    LevelOfDetail(1),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: RED_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh.clone()),
                ),

            ]
        )]
));
```


### Scattering

Now you can start scattering! 🌱 🍃 🌿 🍀 🌳 🌲 🌴 🌺

```rust
cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
```
> [!NOTE]
> `ScatterLayers` and their `ScatterItems` of the same `ScatterType` are always scattered in order, but layers of different `ScatterTypes` can be scattered at the same time.

#### Ordered Scattering

In complex scenes it is often required to scatter a complete hierarchy in order (rocks → trees/foliage → grass).

> [!TIP]
> If an ordered scatter is still required, and you can't or don't want to scatter in parallel, observers need to be used to chain the scattering of `ScatterTypes` in order.

```rust
fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    root: Single<Entity, With<ScatterRoot>>
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    // Scatter the rocks.
    cmd.trigger(Scatter::<StandardMaterial>::new(*root));
}

fn scatter_extended(
    _: On<ScatterFinished<StandardMaterial>>,
    mut cmd: Commands,
    root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the foliage after the rocks.
    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
}

fn scatter_instanced(
    _: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
    mut cmd: Commands,
    root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the grass last so it doesn't grow on occupied areas.
    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}

```

## Compatibility

There are very experimental releases before 0.5.0, but I wouldn't use them.

| bevy        | bevy_feronia |
|-------------|--------------|
| 0.18        | 0.8+         |
| 0.17        | 0.7          |

## License/Credits/Inspirations/References

The code is dual-licensed:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

Feel free to copy the grass assets. All the other assets used in the examples are licensed assets.

> [!IMPORTANT]
> If you intend to use them, make sure you comply with the license.

- [Graswald](https://gscatter.com/gallery) for their amazing assets and [`GScatter`](https://gscatter.com/gscatter), which served as inspiration for the scatter tools.
- Sucker Punch Productions for their Procedural Grass and Wind simulation in 'Ghost of Tsushima'
  and [GDC Talks](https://www.youtube.com/watch?v=Ibe1JBF5i5Y).
- [bevy_procedural_grass](https://github.com/jadedbay/bevy_procedural_grass) by jadedbay
- [warbler_grass](https://github.com/EmiOnGit/warbler_grass) by EmiOnGit
- [GDC 2011 "Approximating Translucency"](https://www.gdcvault.com/play/1014538/Approximating-Translucency-for-a-Fast)
- [Blinn–Phong reflection model](https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model)
- [All the other assets](/assets/LICENSE)

## Alternatives

- If you want a lower level scattering API, just for scattering and sampling there is also [`bevy_map_scatter`](https://github.com/morgenthum/map_scatter),
  which I might use eventually as well but for now I want this crate to achieve a vision.

## Roadmap

A bunch of issues are already open, but some of the larger milestones could be:

- Allow physics-based and other entities to impact the displacement/wind.
- Make use of compute shaders (Allow scattering on CPU and GPU, improve culling).
- Allow for multiple `ScatterRoots` when baking height maps (otherwise it should work... probably but it isn't tested yet)



