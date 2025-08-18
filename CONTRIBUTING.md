## Contributing

Hey! Thanks for your interest in contributing. Filing issues, improving code, or adding features are all appreciated. 🌱

I've created some issues to track planned features, but feel free to open a new one if you find a bug or have a suggestion. Pull requests are also welcome!

### Naming Conventions
I try to follow a few naming conventions, but they're not super strict:

- ```cmd: Commands```
- ```q_some_entity: Query<Entity,With<SomeEntity>>```
- `er_some_event` and `ew_some_event` for `EventReader`/`EventWriter`
- using short names like `i`, `x`, `e` is okay if the scope is small, clear and it doesn't hurt readability.

### Debugging
To see detailed log messages while running the project, you can set the `RUST_LOG` environment variable:

 `RUST_LOG="warn,bevy_feronia=debug"`

### CI / GitHub Actions
There is no CI or GitHub Actions pipeline yet. I'm not opposed to adding one, I just haven't gotten around to it.
This needs to be done eventually anyway, so if you'd like to set one up, a PR would be great.

