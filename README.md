# Apothic Spawners — a Horizon port

**Apothic Spawners** is a Minecraft mod. The Spawner module of Apotheosis. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Shadows-of-Fire/Apothic-Spawners](https://github.com/Shadows-of-Fire/Apothic-Spawners) — licence **All Rights Reserved (assets; code MIT)**.

| | |
|---|---|
| Upstream mod | Apothic Spawners ([source](https://github.com/Shadows-of-Fire/Apothic-Spawners)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `apothic_spawners.hmod` (install it with the Horizon launcher) |
| Licence | **All Rights Reserved (assets; code MIT)** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What works

- Spawn egg sets the spawner's mob.
- `min_delay`/`max_delay`/`spawn_count`/`max_nearby`/`player_range`/
  `spawn_range`/`initial_health` and the `ignore_players`, `redstone_control`,
  `no_ai`, `silent`, `youthful`, `burning` stats.
- The upstream modifier items and their quartz-offhand inverses.
- Silk-touch drops keep the mob and stats; placing the dropped stack restores them.
- A vanilla spawner broken with silk touch drops the Apothic spawner with its
  mob and vanilla settings; our item used on one converts it in place.

## Not yet ported

- **Not a vanilla-spawner override.** Horizon content mods cannot replace
- **`ignore_conditions` and `ignore_light` are stored and shown but inert.**
- **`echoing` is stored but does not duplicate loot.** Extra death drops need
- **No unstable explosion.** `onBlockExploded`/detonation would need an
- **No capturing enchantment.** Enchantment registration and the drop-egg
- **No client spin/display.** The block model renders; the spinning mob in a
- **No tooltip lines.** Upstream shows stats on the item with Shift; the pin's
- **No advancements/JEI/Jade/TOP integration.** Those upstream files are

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `apothic_spawners.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Apothic Spawners was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **All Rights Reserved (assets; code MIT)**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Shadows-of-Fire/Apothic-Spawners

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run apothic_spawners` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/apothic_spawners/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
