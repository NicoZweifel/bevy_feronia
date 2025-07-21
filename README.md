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

- Chunking
- Texture-based Scattering.
- Performance Improvements (e.g., LODs) / Shortcuts (e.g., procedurally defined geometry instead of calculating neighbor pos), use chunks and shader flags.
- Make physics-based entities impact the displacement.
- Make use of compute shaders

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






