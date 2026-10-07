# Patched dependencies

Crates carried here instead of taken from crates.io, each with a fix the
published version lacks. They are wired in with `[patch.crates-io]` in the
workspace `Cargo.toml`, and excluded from the workspace members so `make ci`
does not lint or test them as our own code. Each one is the published crate
with as small a change as possible, marked `OPENROAD PATCH` in the source.
Drop the patch as soon as an upstream release carries the fix.

## `bevy_pbr` 0.19.1

**Why:** GPUs without storage buffers fail to render any mesh. That is
WebGL2, and wgpu's GL 3.3 backend on DX10-class cards (Radeon HD 2000–6000,
GeForce 8–200), the oldest tier the client targets (`docs/RUNNING.md` §8,
ADR 0011).

**The bug:** in `src/render/mesh_view_bindings.rs`, mesh-view binding 14
(`visibility_ranges`) falls back to a uniform buffer when storage buffers
are unavailable. The shader then declares it as `array<vec4<f32>, 64>`
(1024 bytes), but the bind-group layout keeps the storage path's
`min_binding_size` of one `Vec4` (16 bytes). wgpu validates layouts on every
backend, so `pbr_opaque_mesh_pipeline` fails to build and Bevy's render error
policy quits the app. Reproduced on Vulkan with `OPENROAD_GPU_BASELINE=gl33`
(`docs/perf-remote.md`, "The GL 3.3 floor").

**The fix:** use the whole array's size as the minimum for the uniform
fallback. The storage path is unchanged.

**Upstream:** not reported yet. It is a candidate for a Bevy issue, filed by
a person since the project does not accept AI-generated contributions.

Licence: MIT OR Apache-2.0, © the Bevy contributors. The texts are in
`bevy_pbr/LICENSE-MIT` and `bevy_pbr/LICENSE-APACHE`.
