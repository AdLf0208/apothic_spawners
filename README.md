# apothic_spawners (Horizon port)

A Horizon port of [apothic_spawners](Shadows-of-Fire/Apothic-Spawners) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- see notes.md in the port (this page is generated)

## Not yet ported

- Not a vanilla-spawner override.** Horizon content mods cannot replace
- `minecraft:spawner`; the port adds a block and provides the silk-touch drop
- and in-place conversion bridge. A world's existing vanilla spawners stay
- vanilla until converted.
- `ignore_conditions` and `ignore_light` are stored and shown but inert.**
- The pin's spawn helpers (`native_spawn_mob`, `native_summon`) do not run
- vanilla `SpawnPlacements`/custom-rule checks, so there is no condition check
- to bypass. The port always requires air for the mob's height, as vanilla's

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: Shadows-of-Fire/Apothic-Spawners. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
