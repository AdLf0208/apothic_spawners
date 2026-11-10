// Apothic Spawners for Horizon — pure logic for the configurable spawner.
//
// Ported from Apothic Spawners 2.0.1
// (Shadows-of-Fire/Apothic-Spawners, MIT, commit
// `a5401df4a80b833b297c398f88a5718bdd47c02a`). Upstream source was read for
// behaviour only; this is a clean-room Rust rewrite.
//
// Upstream replaces vanilla's spawner block entity and item, so every
// `minecraft:spawner` gains stats and modifiers. Horizon content mods cannot
// replace a vanilla block, so this port adds `apothic_spawners:spawner`: a
// spawner with the same model and the same stats, plus a right-click bridge
// that converts a vanilla spawner (see notes.md).

pub const SPAWNER: &str = "apothic_spawners:spawner";
pub const VANILLA_SPAWNER: &str = "minecraft:spawner";

/// `BaseSpawner`'s defaults (`SpawnerBlockEntity`).
pub const DEFAULT_MIN_DELAY: i32 = 200;
pub const DEFAULT_MAX_DELAY: i32 = 800;
pub const DEFAULT_SPAWN_COUNT: i32 = 4;
pub const DEFAULT_MAX_NEARBY: i32 = 6;
pub const DEFAULT_PLAYER_RANGE: i32 = 16;
pub const DEFAULT_SPAWN_RANGE: i32 = 4;

/// The bounds upstream's modifiers clamp to (`ASRecipeProvider`).
pub const MIN_DELAY_FLOOR: i32 = 20;
pub const MIN_DELAY_CEIL: i32 = 1600;
pub const MAX_DELAY_FLOOR: i32 = 20;
pub const MAX_DELAY_CEIL: i32 = 1600;
pub const SPAWN_COUNT_FLOOR: i32 = 1;
pub const SPAWN_COUNT_CEIL: i32 = 16;
pub const MAX_NEARBY_FLOOR: i32 = 1;
pub const MAX_NEARBY_CEIL: i32 = 32;
pub const PLAYER_RANGE_FLOOR: i32 = 1;
pub const PLAYER_RANGE_CEIL: i32 = 48;
pub const SPAWN_RANGE_FLOOR: i32 = 1;
pub const SPAWN_RANGE_CEIL: i32 = 32;
pub const HEALTH_FLOOR: f32 = 0.2;
pub const HEALTH_CEIL: f32 = 1.0;
pub const ECHOING_CEIL: i32 = 3;

/// One spawner's stats, as the block entity stores them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub min_delay: i32,
    pub max_delay: i32,
    pub spawn_count: i32,
    pub max_nearby: i32,
    pub player_range: i32,
    pub spawn_range: i32,
    pub initial_health: f32,
    pub echoing: i32,
    pub ignore_players: bool,
    pub ignore_conditions: bool,
    pub redstone_control: bool,
    pub ignore_light: bool,
    pub no_ai: bool,
    pub silent: bool,
    pub youthful: bool,
    pub burning: bool,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            min_delay: DEFAULT_MIN_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            spawn_count: DEFAULT_SPAWN_COUNT,
            max_nearby: DEFAULT_MAX_NEARBY,
            player_range: DEFAULT_PLAYER_RANGE,
            spawn_range: DEFAULT_SPAWN_RANGE,
            initial_health: 1.0,
            echoing: 0,
            ignore_players: false,
            ignore_conditions: false,
            redstone_control: false,
            ignore_light: false,
            no_ai: false,
            silent: false,
            youthful: false,
            burning: false,
        }
    }
}

/// A stat change a modifier item applies (`StatModifier`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    MinDelay(i32),
    MaxDelay(i32),
    SpawnCount(i32),
    MaxNearby(i32),
    PlayerRange(i32),
    SpawnRange(i32),
    InitialHealth(f32),
    Echoing(i32),
    IgnorePlayers(bool),
    IgnoreConditions(bool),
    RedstoneControl(bool),
    IgnoreLight(bool),
    NoAi(bool),
    Silent(bool),
    Youthful(bool),
    Burning(bool),
}

fn clamp_int(value: i32, floor: i32, ceiling: i32) -> i32 {
    value.clamp(floor, ceiling)
}

fn clamp_float(value: f32, floor: f32, ceiling: f32) -> f32 {
    value.clamp(floor, ceiling)
}

/// Apply one modifier, clamped as upstream clamps it; true when the value
/// changed. A `SET`-mode boolean that already holds its value changes nothing.
pub fn apply(stats: &mut Stats, change: Change) -> bool {
    let before = *stats;
    match change {
        Change::MinDelay(value) => {
            stats.min_delay = clamp_int(
                stats.min_delay.saturating_add(value),
                MIN_DELAY_FLOOR,
                MIN_DELAY_CEIL,
            );
        }
        Change::MaxDelay(value) => {
            stats.max_delay = clamp_int(
                stats.max_delay.saturating_add(value),
                MAX_DELAY_FLOOR,
                MAX_DELAY_CEIL,
            );
        }
        Change::SpawnCount(value) => {
            stats.spawn_count = clamp_int(
                stats.spawn_count.saturating_add(value),
                SPAWN_COUNT_FLOOR,
                SPAWN_COUNT_CEIL,
            );
        }
        Change::MaxNearby(value) => {
            stats.max_nearby = clamp_int(
                stats.max_nearby.saturating_add(value),
                MAX_NEARBY_FLOOR,
                MAX_NEARBY_CEIL,
            );
        }
        Change::PlayerRange(value) => {
            stats.player_range = clamp_int(
                stats.player_range.saturating_add(value),
                PLAYER_RANGE_FLOOR,
                PLAYER_RANGE_CEIL,
            );
        }
        Change::SpawnRange(value) => {
            stats.spawn_range = clamp_int(
                stats.spawn_range.saturating_add(value),
                SPAWN_RANGE_FLOOR,
                SPAWN_RANGE_CEIL,
            );
        }
        Change::InitialHealth(value) => {
            stats.initial_health =
                clamp_float(stats.initial_health + value, HEALTH_FLOOR, HEALTH_CEIL);
        }
        Change::Echoing(value) => {
            stats.echoing = clamp_int(stats.echoing.saturating_add(value), 0, ECHOING_CEIL);
        }
        Change::IgnorePlayers(value) => stats.ignore_players = value,
        Change::IgnoreConditions(value) => stats.ignore_conditions = value,
        Change::RedstoneControl(value) => stats.redstone_control = value,
        Change::IgnoreLight(value) => stats.ignore_light = value,
        Change::NoAi(value) => stats.no_ai = value,
        Change::Silent(value) => stats.silent = value,
        Change::Youthful(value) => stats.youthful = value,
        Change::Burning(value) => stats.burning = value,
    }
    *stats != before
}

/// Whether the offhand is the inverse modifier's catalyst: a quartz held in
/// the offhand turns every forward modifier into its inverse
/// (`ASRecipeProvider.addInverse`).
pub fn is_inverse(offhand: Option<&str>) -> bool {
    offhand == Some("minecraft:quartz")
}

/// Whether `item` is wool (`ItemTags.WOOL`): the silent modifier's main hand.
fn is_wool(item: &str) -> bool {
    item.strip_prefix("minecraft:")
        .is_some_and(|name| name.ends_with("_wool"))
}

/// The stat change `main` applies, or the inverse when the offhand is quartz.
/// Mirrors `ASRecipeProvider`'s recipe table. `None` when the item is not a
/// modifier.
pub fn change_for(main: &str, offhand: Option<&str>) -> Option<Change> {
    let inverse = is_inverse(offhand);
    let change = match main {
        "minecraft:sugar" => Change::MinDelay(if inverse { 10 } else { -10 }),
        "minecraft:clock" => Change::MaxDelay(if inverse { 20 } else { -20 }),
        "minecraft:fermented_spider_eye" => Change::SpawnCount(if inverse { -2 } else { 2 }),
        "minecraft:ghast_tear" => Change::MaxNearby(if inverse { -2 } else { 2 }),
        "minecraft:prismarine_crystals" => Change::PlayerRange(if inverse { -4 } else { 4 }),
        "minecraft:piston" => Change::SpawnRange(if inverse { -2 } else { 2 }),
        "minecraft:pointed_dripstone" => Change::InitialHealth(if inverse { 0.05 } else { -0.05 }),
        "minecraft:echo_shard" => Change::Echoing(if inverse { -1 } else { 1 }),
        "minecraft:nether_star" => Change::IgnorePlayers(!inverse),
        "minecraft:conduit" => Change::IgnoreConditions(!inverse),
        "minecraft:comparator" => Change::RedstoneControl(!inverse),
        "minecraft:soul_lantern" => Change::IgnoreLight(!inverse),
        "minecraft:chorus_fruit" => Change::NoAi(!inverse),
        "minecraft:golden_dandelion" => Change::Youthful(!inverse),
        "minecraft:campfire" => Change::Burning(!inverse),
        _ if is_wool(main) => Change::Silent(!inverse),
        _ => return None,
    };
    Some(change)
}

/// The delay a fresh spawner draws: `min + rand[0, max - min)` as
/// `BaseSpawner.delay` does. When the bounds are equal the minimum is used.
pub fn spawn_delay(min: i32, max: i32, roll: i32) -> i32 {
    if max <= min {
        min
    } else {
        min.saturating_add(roll.rem_euclid(max - min))
    }
}

/// One tick of the countdown. `roll` is only drawn when a new delay is
/// needed. Returns the delay to store and whether a spawn is due now.
pub fn next_delay(delay: i32, min: i32, max: i32, roll: i32) -> (i32, bool) {
    if delay < 0 {
        return (spawn_delay(min, max, roll), false);
    }
    if delay > 0 {
        return (delay - 1, false);
    }
    (spawn_delay(min, max, roll), true)
}

/// Whether the spawner may run this tick: a player in range unless it
/// ignores players, and redstone power when it requires it.
pub fn activated(
    ignore_players: bool,
    player_near: bool,
    redstone_control: bool,
    powered: bool,
) -> bool {
    let players = ignore_players || player_near;
    players && (!redstone_control || powered)
}

/// Whether `pos` lies in the spawn box: `spawn_range` blocks out on each
/// horizontal axis and one block down/up from the spawner.
pub fn in_spawn_area(center: [i32; 3], pos: [i32; 3], range: i32) -> bool {
    let range = range.max(0);
    (pos[0] - center[0]).abs() <= range
        && (pos[2] - center[2]).abs() <= range
        && (pos[1] - center[1]).abs() <= 1
}

/// The health a spawned mob starts with, as a fraction of its maximum.
pub fn health_for(max_health: f32, percent: f32) -> f32 {
    (max_health * clamp_float(percent, HEALTH_FLOOR, HEALTH_CEIL)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_vanilla_spawners() {
        let stats = Stats::default();
        assert_eq!(stats.min_delay, 200);
        assert_eq!(stats.max_delay, 800);
        assert_eq!(stats.spawn_count, 4);
        assert_eq!(stats.max_nearby, 6);
        assert_eq!(stats.player_range, 16);
        assert_eq!(stats.spawn_range, 4);
        assert_eq!(stats.initial_health, 1.0);
        assert_eq!(stats.echoing, 0);
        assert!(!stats.no_ai);
    }

    #[test]
    fn forward_modifiers_match_the_recipe_table() {
        let mut stats = Stats::default();
        assert!(apply(
            &mut stats,
            change_for("minecraft:sugar", None).unwrap()
        ));
        assert_eq!(stats.min_delay, 190);
        assert!(apply(
            &mut stats,
            change_for("minecraft:clock", None).unwrap()
        ));
        assert_eq!(stats.max_delay, 780);
        assert!(apply(
            &mut stats,
            change_for("minecraft:fermented_spider_eye", None).unwrap()
        ));
        assert_eq!(stats.spawn_count, 6);
        assert!(apply(
            &mut stats,
            change_for("minecraft:nether_star", None).unwrap()
        ));
        assert!(stats.ignore_players);
        assert!(!apply(
            &mut stats,
            change_for("minecraft:nether_star", None).unwrap()
        ));
    }

    #[test]
    fn quartz_turns_a_modifier_into_its_inverse() {
        assert_eq!(
            change_for("minecraft:sugar", Some("minecraft:quartz")),
            Some(Change::MinDelay(10))
        );
        assert_eq!(
            change_for("minecraft:campfire", Some("minecraft:quartz")),
            Some(Change::Burning(false))
        );
        assert_eq!(
            change_for("minecraft:stick", Some("minecraft:quartz")),
            None
        );
    }

    #[test]
    fn modifiers_clamp_at_upstreams_bounds() {
        let mut stats = Stats {
            min_delay: 25,
            spawn_count: 3,
            initial_health: 0.22,
            echoing: 3,
            ..Default::default()
        };
        assert!(apply(&mut stats, Change::MinDelay(-10)));
        assert_eq!(stats.min_delay, 20);
        assert!(!apply(&mut stats, Change::MinDelay(-10)));
        assert!(apply(&mut stats, Change::SpawnCount(-2)));
        assert_eq!(stats.spawn_count, 1);
        assert!(apply(&mut stats, Change::InitialHealth(-0.05)));
        assert!((stats.initial_health - 0.2).abs() < 1e-6);
        assert!(!apply(&mut stats, Change::Echoing(1)));
        assert_eq!(stats.echoing, 3);
    }

    #[test]
    fn wool_is_the_silent_modifier() {
        assert_eq!(
            change_for("minecraft:white_wool", None),
            Some(Change::Silent(true))
        );
        assert_eq!(
            change_for("minecraft:red_wool", Some("minecraft:quartz")),
            Some(Change::Silent(false))
        );
        assert_eq!(change_for("minecraft:wool", None), None);
    }

    #[test]
    fn delay_draws_stay_inside_the_bounds() {
        assert_eq!(spawn_delay(200, 800, 0), 200);
        assert_eq!(spawn_delay(200, 800, 599), 799);
        assert_eq!(spawn_delay(200, 800, 600), 200);
        assert_eq!(spawn_delay(40, 40, 17), 40);
        assert_eq!(next_delay(1, 20, 40, 0), (0, false));
        assert_eq!(next_delay(0, 20, 40, 0), (20, true));
        assert_eq!(next_delay(-1, 20, 40, 0), (20, false));
    }

    #[test]
    fn activation_needs_players_and_power_only_when_asked() {
        assert!(activated(false, true, false, false));
        assert!(!activated(false, false, false, true));
        assert!(activated(true, false, false, false));
        assert!(!activated(true, true, true, false));
        assert!(activated(true, false, true, true));
    }

    #[test]
    fn spawn_area_is_a_flat_box_around_the_spawner() {
        let center = [10, 64, -4];
        assert!(in_spawn_area(center, [10, 64, -4], 4));
        assert!(in_spawn_area(center, [14, 65, 0], 4));
        assert!(!in_spawn_area(center, [15, 64, -4], 4));
        assert!(!in_spawn_area(center, [10, 66, -4], 4));
    }

    #[test]
    fn health_is_a_clamped_fraction_of_max() {
        assert_eq!(health_for(20.0, 1.0), 20.0);
        assert_eq!(health_for(20.0, 0.5), 10.0);
        assert_eq!(health_for(20.0, 0.0), 4.0);
        assert_eq!(health_for(20.0, 5.0), 20.0);
    }
}
