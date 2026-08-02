# lava-arch

Composition layer for the lava suite.

`lava-arch` provides the `deflava-architecture` form and its Rust builders.
Architectures consume and return typed `ResourceRef` chains, so a downstream
architecture slots into an upstream architecture's outputs **at composition
time** rather than at apply time — a wiring mistake is a build error, not a
failed apply.

It is the lava analog of Pangea's `Architecture` / `ResourceBuilder` pair.

## Install

```toml
[dependencies]
lava-arch = "0.1"
```

## Where it sits

```
lava-core ──► lava-arch ──► lava-architectures
```

`lava-arch` depends only on [`lava-core`](https://github.com/pleme-io/lava-core)
(the typed resource primitives). The concrete architectures built on top of it
live in [`lava-architectures`](https://github.com/pleme-io/lava-architectures).

## License

MIT
