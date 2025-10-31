# bevy_feronia ![crates.io](https://img.shields.io/crates/v/bevy_feronia.svg)

Foliage/grass scattering tools and wind simulation shaders/materials that prioritize visual fidelity/artistic freedom, a declarative API and modularity.

> [!CAUTION]
> This package is in early development and in an experimentation stage. 
>

<img width="3440" height="1392" alt="Screenshot 2025-10-30 180213" src="https://github.com/user-attachments/assets/b00a0f73-f3ea-471c-b688-6aa2a478014e" />


### Getting started

```shell
cargo add bevy_feronia
```

The possible use-cases are demonstrated in the [examples](/examples/EXAMPLES.md)

#### Setup

The setup depends on the use-case, but a typical setup would look like something like this:

```rust
app.add_plugins((
    ExtendedWindAffectedScatterPlugin
));
```

The Scatter system needs to know when it can setup since it can depend on height mapping. You need to insert the setup state at some point.

> [!NOTE]  
> In complex setups that load assets and bake a height map this can be after the `Startup`.

```rust
app.insert_state(ScatterState::Setup)
```

or
```rust
ns_height_map.set(HeightMapState::Setup);
ns_scatter.set(ScatterState::Setup);
```

### Defining layers

A `ScatterItem`'s `LOD`'s are grouped by `Name`.

> [!CAUTION]
> When defining multiple assets per layer without names, a different Asset will render when `LODs` are changing, leading to visual bugs.

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
            scatter_layer("Wind Affected Layer"),
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
                    // You can have multiple types in each layer; as long as all LODs have the same name, they will be matched correctly.
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
cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*q_root));
```

A `ScatterLayer` is always scattered in hierarchical order, but layers of different types can run in parallel.

> [!TIP]
> If a hierarchical scatter is still required, observers need to be used to chain the scattering of types in order.

```rust
fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Single<Entity, With<ScatterRoot>>
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    // Scatter the rocks.
    cmd.trigger(Scatter::<StandardMaterial>::new(*q_root));
}

fn scatter_extended(
    _: On<ScatterFinished<StandardMaterial>>,
    mut cmd: Commands,
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the foliage after the rocks.
    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*q_root));
}

fn scatter_instanced(
    _: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
    mut cmd: Commands,
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the grass last so it doesn't grow on occupied areas.
    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*q_root));
}

```


### Credits/Inspirations/References

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

### Roadmap

A bunch of issues are already open, but some of the larger milestones could be:

- Allow physics-based and other entities to impact the displacement/wind.
- Make use of compute shaders (Allow scattering on CPU and GPU, improve culling).



