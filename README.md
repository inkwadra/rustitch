# rustitch

> [!IMPORTANT]
> `rustitch` is still in an early design and development stage. The API is not stable yet, and breaking changes should be expected while the project structure and public surface continue to evolve.

`rustitch` is an early-stage Rust workspace for building Twitch integrations with a unified, type-safe API. The project is organized around the main Twitch domains: authentication, Helix, EventSub, chat, and shared core primitives that can be reused across the workspace.

The public entry point is the `rustitch` facade crate. Behind it, the workspace is split into focused crates so each domain can evolve with clear boundaries instead of collapsing into a single monolithic library.

## Roadmap

- [ ] Complete the initial public API surface.
- [ ] Expand Helix and EventSub coverage.
- [ ] Improve chat support and transport integration.
- [ ] Add stronger tests for core workflows and protocol edge cases.
- [ ] Improve examples and usage documentation.
- [ ] Prepare the project for an initial stable release process.
