# bevy_feronia

Foliage/Grass Wind simulation shaders/materials that prioritize visual fidelity/artistic freedom and modularity.

> [!CAUTION]
> This package is in very early development, api's will most likely change and be modularized.
>
> The performance isn't great for most scenarios atm and in general it is not stable.
>
> The dev branch points at bevy's main branch for the time being because the manual instancing example uses new API's.
> The main branch will stay on 0.16.x until 0.17.0 releases.

### Roadmap

- Chunking of landscape/meshes
- Texture-based Scattering on top of landscape/meshes
- Performance Improvements (e.g., LODs) / Shortcuts (e.g., more efficient calculations for procedural geometry)
- Make physics-based entities impact the displacement
- Make use of compute shaders (Allow scattering/sampling of foliage on CPU and GPU to get scattering of high-quality
  assets and procedural geometry)

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

### Credits/Inspirations

- [Graswald](https://gscatter.com/gallery) for their amazing assets.
- Sucker Punch Productions for their Procedural Grass and Wind simulation in 'Ghost of Tsushima'
  and [GDC Talks](https://www.youtube.com/watch?v=Ibe1JBF5i5Y).



