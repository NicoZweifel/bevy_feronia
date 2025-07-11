# bevy_feronia

Foliage/Grass Wind simulation shaders/materials that prioritize visual fidelity/artistic freedom and modularity. 

> [!CAUTION]
> This package is in very early development, api's will most likely change and be modularized. The performance isn't great for most scenarios atm and in general it is not stable.

### Roadmap

- Texture based Scattering.
- Manual GPU Instancing.
- Compute Shaders for procedural grass (covering areas and pre-calculated geometry).
- Performance Improvements (e.g. skip calculations for LODs) / Shortcuts (e.g. procedurally defined geometry instead of calculating neighbor pos)


### Examples

- Press space to scatter plants.
- The `Wind` Resource is configurable in the Inspector Window.


#### Grass

`cargo run --example cargo run --example extended_material_grass`

https://github.com/user-attachments/assets/17459f40-9dc5-4032-82b8-63ab7fa9e16a

### Foliage
`cargo run --example cargo run --example extended_material_foliage`

https://github.com/user-attachments/assets/4b71415e-63d7-4a5b-b85a-9cb4408abdab

### Foliage complex
`cargo run --example cargo run --example extended_material_foliage_complex`

https://github.com/user-attachments/assets/63d6d98a-6b5a-47a6-853c-0c336a89f3e6





