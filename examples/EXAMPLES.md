
## Examples

- Press `SPACE` to scatter.
- The `Wind` Resource is configurable in the Inspector Window.

All examples have the window and a performance overlay,
but the `Wind` resource doesn't automatically sync all the materials in all of the examples.

Most examples (except `extension`, `instanced` and `scatter`) use assets that are tracked with [Git LFS](https://git-lfs.com/).

> [!TIP]  
>
> The examples all use `TAA` by default, you can however run it with `DLAA` by using `--features dlss`.
>
> How to set up `DLSS` can be found [here](https://github.com/bevyengine/dlss_wgpu).


### Full

A complex scene with rocks, trees, foliage and grass.
This example demonstrates ordered scattering (rocks → trees/foliage → grass) and combines all three scatter plugins.

> [!IMPORTANT]  
> Doesn't work without [LFS](https://git-lfs.com/) tracked assets!

`cargo run --example full`

https://github.com/user-attachments/assets/5aa9caa8-670f-482e-8a8a-0a24994ade84

The other examples are mostly for demonstration and testing purposes:

### Scatter

Shows the basic scatter logic. This is the `Getting started`/`Setup` section as an example. 

`cargo run --example scatter`

#### Avian

Experimental example, will likely heavily change in the future.

`cargo run --example avian`

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

These demonstrate methods and options related to grass scattering.

> [!IMPORTANT]  
> These don't work without [Git LFS](https://git-lfs.com/) tracked assets!

- Demonstrates scattering non-instanced grass. Mostly for testing/reference.

`cargo run --example scatter_extended_grass`

- A basic example of high-density, wind-affected grass.
 
`cargo run --example scatter_instanced_grass`

- Scatters high-density instanced grass in chunks on a large landscape.
 
`cargo run --example scatter_instanced_chunks`

- Shows how to control instanced grass placement using a DensityMap .

`cargo run --example scatter_instanced_density_map`


### Foliage

Demonstrates scattering wind-affected foliage.

> [!IMPORTANT]  
> Doesn't work without [Git LFS](https://git-lfs.com/) tracked assets!

`cargo run --example scatter_extended_foliage`

### Trees

Demonstrates scattering wind-affected trees.

> [!IMPORTANT]  
> Doesn't work without [Git LFS](https://git-lfs.com/) tracked assets!

`cargo run --example scatter_extended_trees`




