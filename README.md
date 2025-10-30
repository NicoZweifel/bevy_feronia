# bevy_feronia

Foliage/grass scattering tools and wind simulation shaders/materials that prioritize visual fidelity/artistic freedom, a declarative api and modularity.

> [!CAUTION]
> This package is in very early development and in an experimentation stage.
>



### Examples

- Press `SPACE` to scatter plants.
- The `Wind` Resource is configurable in the Inspector Window.

#### Grass

`cargo run --example extended_material_grass`

https://github.com/user-attachments/assets/b6adb502-aa99-412f-8c6c-67418d59aa3a

### Foliage

`cargo run --example extended_material_foliage`

https://github.com/user-attachments/assets/4b71415e-63d7-4a5b-b85a-9cb4408abdab

### Foliage complex

`cargo run --example extended_material_foliage_complex`

https://github.com/user-attachments/assets/63d6d98a-6b5a-47a6-853c-0c336a89f3e6

### Manually instanced grass

`cargo run --example instanced_material_grass`

https://github.com/user-attachments/assets/486b3df2-3669-4ac5-850d-0826d1f881a1

### Manually instanced grass split into chunks

`cargo run --example instanced_material_chunks`

https://github.com/user-attachments/assets/6c1e64ad-004a-4a38-8034-eb5c25ce7f8a

### Manually instanced grass with a cpu-sampled density map and chunks

`cargo run --example instanced_density_map`

https://github.com/user-attachments/assets/3141e4ac-24ff-4519-8ba5-afadf8f6a2ad

### Full example WIP

`cargo run --example full`

https://github.com/user-attachments/assets/2b81a6d2-cd6a-4baa-85c3-1dc814114a37

### Credits/Inspirations

- [Graswald](https://gscatter.com/gallery) for their amazing assets.
- Sucker Punch Productions for their Procedural Grass and Wind simulation in 'Ghost of Tsushima'
  and [GDC Talks](https://www.youtube.com/watch?v=Ibe1JBF5i5Y).
- [bevy_procedural_grass](https://github.com/jadedbay/bevy_procedural_grass) by jadedbay



### Roadmap

There are a bunch of issues already open, but some of the larger milestones left would be:

- Allow physics-based and other entities to impact the displacement/wind.
- Make use of compute shaders (Allow scattering on CPU and GPU, improve culling).



