
## Examples

- Press `SPACE` to scatter.
- The `Wind` Resource is configurable in the Inspector Window.

All examples have the window and a performance overlay. 

> [!TIP]  
>
> The examples all use `TAA` and `SSAO` by default, you can however run it with `DLSS` by using `--features dlss`.
>
> How to set up `DLSS` can be found [here](https://github.com/bevyengine/dlss_wgpu).


### Full

A complex scene with rocks, trees, foliage and grass.
This example demonstrates ordered scattering (rocks → trees/foliage → grass) and combines all three scatter plugins.

`cargo run --example full`

https://github.com/user-attachments/assets/970e69d9-6a05-4897-9cff-754c845814fe

The other examples are mostly for demonstration and testing purposes:

### Scatter

Shows the basic scatter logic. This is the `Getting started`/`Setup` section as an example. 

`cargo run --example scatter`

- A minimal example showing how to scatter cuboids.

### Materials (Standalone)

These show how to use the wind-affected materials directly without any scattering plugins.

> [!NOTE]
> Useful if you want to apply wind effects to individual, non-scattered meshes, i.e., manually placed entities.

`cargo run --example extension`

- Demonstrates applying wind effects to a single cuboid.

`cargo run --example instanced`

- Demonstrates applying wind effects to a set of instanced cuboids.

### Grass

`cargo run --example scatter_extended_grass`

- Demonstrates scattering non-instanced grass. Mostly for testing/reference. 

`cargo run --example scatter_instanced_grass`

- A basic example of high-density, wind-affected grass.
 
`cargo run --example scatter_instanced_chunks`

- Scatters high-density instanced grass in chunks on a large landscape.

`cargo run --example scatter_instanced_density_map`

- Shows how to control instanced grass placement using a DensityMap .

### Foliage

`cargo run --example scatter_extended_foliage`

- Demonstrates scattering wind-affected foliage.

### Trees
`cargo run --example scatter_extended_trees`

- Demonstrates scattering wind-affected trees.




