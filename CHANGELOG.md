# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0-rc.4](https://github.com/NicoZweifel/bevy_feronia/compare/v0.7.0-rc.3...v0.7.0-rc.4) - 2026-02-04

### Added

- feat/scatter occupancy map ([#63](https://github.com/NicoZweifel/bevy_feronia/pull/63))
- edge correction, format, docs
- allow material creation with only mesh  ([#29](https://github.com/NicoZweifel/bevy_feronia/pull/29))
- ci ([#26](https://github.com/NicoZweifel/bevy_feronia/pull/26))
- cleanup, add observers automatically
- feat/full example trees+rocks ([#22](https://github.com/NicoZweifel/bevy_feronia/pull/22))
- fixes, api improvements/refactors
- feat/scatter types shader debug ([#17](https://github.com/NicoZweifel/bevy_feronia/pull/17))
- feat/complex foliage lod ([#14](https://github.com/NicoZweifel/bevy_feronia/pull/14))

### Fixed

- examples
- height mapping with movement/dynamic transform
- flicker when using DLSS/TAA
- wind noise texture resolution/interpolation, noise scale leaking into other calculations, compensate for stretching of the mesh
- quality settings adjustments, respawn lights, remove warning spam when tabbed
- instancing example, uv/normals
- naming, coordinate mixups in numerical normal calc
- instanced material lighting/normal/tangent
- bill boarding, fixes, cleanup, docs  fast/fallback normal fix.
- lighting bugs analytical/numerical, coordinate mixups
- bill boarding
- point lights option not correctly toggling
- foliage complex example
- fix clippy warnings ([#32](https://github.com/NicoZweifel/bevy_feronia/pull/32))
- no need to flip normal with translucent lighting model. tweak constants, naming
- format
- format error
- naming in examples ("q_root" for Single), readme
- grass/shader optimizations/tweaks
- fade/band, max lod not fading out
- low_quality setting, inspector imports
- round_exponent -> curve_factor naming
- trigger targets
- example event/trigger rearchitecture migration
- examples events/triggers
- clamp noise to reduce precision artifacts
- add todo, disable twist if billboarding is enabled
- edge correction not respecting twist, tweak twist
- naming/cleanup
- remove highest detail for now on complex foliage
- fix opacity in foliage example, tweak lod 1 in complex foliage
- resolve chores/warnings ([#12](https://github.com/NicoZweifel/bevy_feronia/pull/12))
- fix examples
- fix readme
- fix typo

### Other

- 0.7.0-rc.1 ([#84](https://github.com/NicoZweifel/bevy_feronia/pull/84))
- Update example description
- Add alternatives section to README
- Add alternatives section to docs
- release v0.5.12 ([#75](https://github.com/NicoZweifel/bevy_feronia/pull/75))
- *(deps)* bump the dependencies group with 5 updates ([#76](https://github.com/NicoZweifel/bevy_feronia/pull/76))
- Fix CI badge link in README.md
- update contributing (trigger some workflows)
- format
- v0.5.11
- v0.5.10
- add clear root on scatter
- Add support for many formats of image for density map ([#68](https://github.com/NicoZweifel/bevy_feronia/pull/68))
- format/move out of loop
- add LodConfig::none(), improve lod 0 grass uvs
- v0.5.9
- update docs with UV notes/blocks, flip uv's in example.
- Update license link in README.md
- update readme / license / badges
- update readme
- comment cpu height sampler
- naming, cleanup, format
- v0.5.8
- v0.5.7
- debug types options in example
- cleanup
- improve debug settings/tools
- improve debug settings/tools use in examples
- readme
- format
- cleanup
- v0.5.6
- format
- v0.5.5
- format/log
- add todo
- v0.5.4
- v0.5.3
- add loading state to example (was flickering occasionally)
- debug/reflect trait impl
- format landscape on add
- use required components and component hooks in full example, add comments/docs
- remove ci section from contributing.md
- add todo
- Fix immediate children skipped in setup_root_aabb ([#59](https://github.com/NicoZweifel/bevy_feronia/pull/59))
- v0.5.1
- docs
- fix accidental rename in docs
- fix docs and move to components to correct module
- 0.5.0 ([#52](https://github.com/NicoZweifel/bevy_feronia/pull/52))
- readme wording
- readme/format/remove tip
- update LFS warnings with links
- v0.4.10
- v0.4.8
- format
- v0.4.7
- normal methods docs
- docs
- docs
- v0.4.6
- format
- format/cleanup extended example
- v0.4.5
- comment billboard fn
- example docs/comments
- v0.4.4
- point lights, tweak constants, lighting fixes (wip) ([#31](https://github.com/NicoZweifel/bevy_feronia/pull/31))
- update config template ([#35](https://github.com/NicoZweifel/bevy_feronia/pull/35))
- bill boarding / edge correction factor component docs
- cleanup shader directives/conditions, format.
- format
- update examples.md with more information
- format, naming, iterator pattern in `queue_material_creation_requests`
- cleanup generic/trait definitions
- v0.4.3
- format
- format, fix naming
- improve README
- `SpawnTrigger` docs
- update trigger field documentation
- docs
- update comment
- fix dlss tip
- remove redundant information
- fix header
- change header
- fix casing
- update description
- examples.md
- Merge branch 'dev' of https://github.com/NicoZweifel/bevy_feronia into dev
- v0.4.2
- readme
- rename Debugging section
- update EXAMPLES.md
- v0.4.0 ([#25](https://github.com/NicoZweifel/bevy_feronia/pull/25))
- tweak full example
- format
- v0.2.1
- v0.2.0
- add repository to cargo manifest
- tweak full example fog volume
- v0.1.0
- Squashed commit of the following:
- format
- v0.1.0-rc.2
- format
- format
- Update Cargo.toml
- Update Cargo.toml
- v0.1.0-rc.1
- re-add inspector gui to examples
- format
- Deterministic scattering ([#20](https://github.com/NicoZweifel/bevy_feronia/pull/20))
- simplify example keypress event handlers
- bevy 0.17
- format
- format
- 0.17 / Event rearchitecture migration ([#21](https://github.com/NicoZweifel/bevy_feronia/pull/21))
- format / improve readiblity in noise.wgsl
- move billboarding to function
- Update README.md
- format
- use string interpolation in warn statement
- replace println with debug
- cleanup cpu height sampler ([#18](https://github.com/NicoZweifel/bevy_feronia/pull/18))
- add todo, reduce sunlight exposure
- add full example section to README.md
- Update CONTRIBUTING.md
- Update CONTRIBUTING.md
- Create CONTRIBUTING.md
- lod blending in complex foliage example ([#13](https://github.com/NicoZweifel/bevy_feronia/pull/13))
- add height map / refactor / full example / 0.0.1 ([#5](https://github.com/NicoZweifel/bevy_feronia/pull/5))
- format, simplify examples. recursively add meshes/materials of windaffected entities and their children
- tweaks, fix fade margin
- Update split.rs
- Update README.md
- Chunking ([#4](https://github.com/NicoZweifel/bevy_feronia/pull/4))
- Merge branch 'master' into dev
- Update README.md
- Update README.md
- Update wind_prepass.wgsl
- refactor shaders / tweaks
- updates/tweaks ([#2](https://github.com/NicoZweifel/bevy_feronia/pull/2))
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Inspector editing ([#1](https://github.com/NicoZweifel/bevy_feronia/pull/1))
- cleanup / use lod fade for bob/s-curve
- licenses, fix cmts, add foliage_complex example
- support bindless
- Update .gitignore
- create repository

## [0.5.12](https://github.com/NicoZweifel/bevy_feronia/compare/v0.5.11...v0.5.12) - 2026-01-06

### Other

- *(deps)* bump the dependencies group with 5 updates ([#76](https://github.com/NicoZweifel/bevy_feronia/pull/76))
- Fix CI badge link in README.md
- update contributing (trigger some workflows)
- format
