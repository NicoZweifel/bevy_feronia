# bevy_feronia ![crates.io](https://img.shields.io/crates/v/bevy_feronia.svg)

Foliage/grass scattering tools and wind simulation shaders/materials that prioritize visual fidelity/artistic freedom, a declarative api and modularity.

> [!CAUTION]
> This package is in early development and in an experimentation stage.
>

<img width="3440" height="1392" alt="Screenshot 2025-10-30 180213" src="https://github.com/user-attachments/assets/b00a0f73-f3ea-471c-b688-6aa2a478014e" />


### Getting started

```shell
cargo add bevy_feronia
```

There are a couple of use cases, you should be able to find most of them here: 

- [Examples](/examples/EXAMPLES.md)


### Credits/Inspirations/References

- [Graswald](https://gscatter.com/gallery) for their amazing assets.
- [Other Assets](/assets/LICENSE)
- Sucker Punch Productions for their Procedural Grass and Wind simulation in 'Ghost of Tsushima'
  and [GDC Talks](https://www.youtube.com/watch?v=Ibe1JBF5i5Y).
- [bevy_procedural_grass](https://github.com/jadedbay/bevy_procedural_grass) by jadedbay
- [warbler_grass](https://github.com/EmiOnGit/warbler_grass) by EmiOnGit
- [GDC 2011 "Approximating Translucency"](https://www.gdcvault.com/play/1014538/Approximating-Translucency-for-a-Fast)
- [Blinn–Phong reflection model](https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model)

### Roadmap

There are a bunch of issues already open, but some of the larger milestones left would be:

- Allow physics-based and other entities to impact the displacement/wind.
- Make use of compute shaders (Allow scattering on CPU and GPU, improve culling).



