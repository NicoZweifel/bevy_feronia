## Contributing

Hey! Thanks for your interest in contributing. Filing issues, improving code, or adding features are all appreciated. 🌱

I've created some issues to track planned features, but feel free to open a new one if you find a bug or have a suggestion. Pull requests are also welcome!

### Naming Conventions
I try to follow a few naming conventions, but they're not super strict:

- ```cmd: Commands```
- ```q_some_entity: Query<Entity,With<SomeEntity>>```
- `mr_some_event` and `mw_some_event` for `MessageReader`/`MessageWriter`
- `o_some_component` for `Option<SomeComponent>` - not super strict and consistent, but it really helps with readability in some locations when using `.map` a lot.
- using short names like `i`, `x`, `e` is okay if the scope is small, clear, and it doesn't hurt readability.

### Log Debugging
To see detailed log messages while running the project, you can enable the `trace` feature and set the `RUST_LOG` environment variable:

 `RUST_LOG="warn,bevy_feronia=debug"`


