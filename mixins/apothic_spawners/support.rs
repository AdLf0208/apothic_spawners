// Game-side glue for Apothic Spawners: the spawner block entity's stats,
// countdown and spawn loop, the modifier items, the spawn-egg interaction and
// the silk-touch drop.
//
// Injected into `crate::game` as `apothic_spawners_state`; it needs the game,
// so it is not part of the standalone library (only `runtime.rs` is).
//
// The spawner is a content block (`apothic_spawners:spawner`) with a mod block
// entity. Upstream replaces vanilla's spawner block entity instead; a content
// mod cannot replace a vanilla block, so this port also converts a vanilla
// spawner in place when its own item is used on one (see notes.md).

use std::collections::HashMap;
use std::sync::Mutex;

use horizon_client::block_entity_item;
use horizon_client::containers::{BlockEntity, ModBlockEntity};
use horizon_client::items::ItemStack;
use horizon_client::mobs::finalize::set_baby;
use horizon_client::mobs::spawner::{SpawnDataExt, SpawnerExt};
use horizon_core::random::RandomSource;
use horizon_world::level::flags;
use horizon_world::nbt::{Compound, JavaString, Tag};

use crate::game::native_hooks::NativeLootDrops;

use crate::game::apothic_spawners::{
    DEFAULT_MAX_DELAY, DEFAULT_MAX_NEARBY, DEFAULT_MIN_DELAY, DEFAULT_PLAYER_RANGE,
    DEFAULT_SPAWN_COUNT, DEFAULT_SPAWN_RANGE, SPAWNER, Stats, VANILLA_SPAWNER, activated, apply,
    change_for, health_for, in_spawn_area, next_delay,
};

/// The block entity's own keys, kept apart from the stats so `/data get` and
/// a dropped item read them plainly.
const MOB: &str = "mob";
const DELAY: &str = "delay";

/// Block entity NBT captured at the head of a player break, for the item the
/// silk-touch drop makes. Keyed by dimension and position.
type SnapshotKey = (String, [i32; 3]);
type Snapshots = HashMap<SnapshotKey, Compound>;
static SNAPSHOTS: Mutex<Option<Snapshots>> = Mutex::new(None);

/// Stats an item carried to a spawner whose block entity was not there yet
/// when `place_block` ran; applied on the first tick.
static PENDING: Mutex<Option<Snapshots>> = Mutex::new(None);

/// The last NBT each spawner ticked with, so a drop still has its stats even
/// when the break hook did not see the block.
static LAST: Mutex<Option<Snapshots>> = Mutex::new(None);

fn snapshot_map() -> std::sync::MutexGuard<'static, Option<Snapshots>> {
    SNAPSHOTS
        .lock()
        .expect("apothic_spawners snapshot lock poisoned")
}

fn pending_map() -> std::sync::MutexGuard<'static, Option<Snapshots>> {
    PENDING
        .lock()
        .expect("apothic_spawners pending lock poisoned")
}

fn last_map() -> std::sync::MutexGuard<'static, Option<Snapshots>> {
    LAST.lock().expect("apothic_spawners last lock poisoned")
}

fn tag<'a>(nbt: &'a Compound, key: &str) -> Option<&'a Tag> {
    nbt.get(&JavaString::from(key))
}

fn int(nbt: &Compound, key: &str) -> Option<i32> {
    match tag(nbt, key) {
        Some(Tag::Int(value)) => Some(*value),
        Some(Tag::Short(value)) => Some(i32::from(*value)),
        Some(Tag::Byte(value)) => Some(i32::from(*value)),
        _ => None,
    }
}

fn float(nbt: &Compound, key: &str) -> Option<f32> {
    match tag(nbt, key) {
        Some(Tag::Float(bits)) => Some(f32::from_bits(*bits)),
        _ => None,
    }
}

fn flag(nbt: &Compound, key: &str) -> Option<bool> {
    match tag(nbt, key) {
        Some(Tag::Byte(value)) => Some(*value != 0),
        _ => None,
    }
}

fn string(nbt: &Compound, key: &str) -> Option<String> {
    match tag(nbt, key) {
        Some(Tag::String(value)) => Some(value.to_string_lossy()),
        _ => None,
    }
}

/// The stats a block entity stores, with vanilla's defaults for absent keys.
fn read_stats(nbt: &Compound) -> Stats {
    Stats {
        min_delay: int(nbt, "min_delay").unwrap_or(DEFAULT_MIN_DELAY),
        max_delay: int(nbt, "max_delay").unwrap_or(DEFAULT_MAX_DELAY),
        spawn_count: int(nbt, "spawn_count").unwrap_or(DEFAULT_SPAWN_COUNT),
        max_nearby: int(nbt, "max_nearby").unwrap_or(DEFAULT_MAX_NEARBY),
        player_range: int(nbt, "player_range").unwrap_or(DEFAULT_PLAYER_RANGE),
        spawn_range: int(nbt, "spawn_range").unwrap_or(DEFAULT_SPAWN_RANGE),
        initial_health: float(nbt, "initial_health").unwrap_or(1.0),
        echoing: int(nbt, "echoing").unwrap_or(0),
        ignore_players: flag(nbt, "ignore_players").unwrap_or(false),
        ignore_conditions: flag(nbt, "ignore_conditions").unwrap_or(false),
        redstone_control: flag(nbt, "redstone_control").unwrap_or(false),
        ignore_light: flag(nbt, "ignore_light").unwrap_or(false),
        no_ai: flag(nbt, "no_ai").unwrap_or(false),
        silent: flag(nbt, "silent").unwrap_or(false),
        youthful: flag(nbt, "youthful").unwrap_or(false),
        burning: flag(nbt, "burning").unwrap_or(false),
    }
}

/// Write every stat, so `/data get block` and a dropped item read them all.
fn write_stats(nbt: &mut Compound, stats: &Stats) {
    for (key, value) in [
        ("min_delay", stats.min_delay),
        ("max_delay", stats.max_delay),
        ("spawn_count", stats.spawn_count),
        ("max_nearby", stats.max_nearby),
        ("player_range", stats.player_range),
        ("spawn_range", stats.spawn_range),
        ("echoing", stats.echoing),
    ] {
        nbt.insert(JavaString::from(key), Tag::Int(value));
    }
    nbt.insert(
        JavaString::from("initial_health"),
        Tag::Float(stats.initial_health.to_bits()),
    );
    for (key, value) in [
        ("ignore_players", stats.ignore_players),
        ("ignore_conditions", stats.ignore_conditions),
        ("redstone_control", stats.redstone_control),
        ("ignore_light", stats.ignore_light),
        ("no_ai", stats.no_ai),
        ("silent", stats.silent),
        ("youthful", stats.youthful),
        ("burning", stats.burning),
    ] {
        nbt.insert(JavaString::from(key), Tag::Byte(i8::from(value)));
    }
}

/// A fresh spawner's block entity.
pub fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if block != SPAWNER || entity.is_some() {
        return;
    }
    let mut nbt = Compound::new();
    write_stats(&mut nbt, &Stats::default());
    nbt.insert(JavaString::from(DELAY), Tag::Int(20));
    *entity = Some((SPAWNER.to_owned(), nbt));
}

/// Our block entity ticks.
pub fn on_block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == SPAWNER {
        *ticks = true;
    }
}

/// The spawner's own tick: count down and add the configured mobs.
#[allow(private_interfaces)]
pub fn on_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
) {
    if entity.id != SPAWNER {
        return;
    }
    last_map()
        .get_or_insert_with(HashMap::new)
        .insert((game.dim.id().to_owned(), pos), entity.nbt.clone());
    if let Some(data) = pending_map()
        .get_or_insert_with(HashMap::new)
        .remove(&(game.dim.id().to_owned(), pos))
    {
        let stats = read_stats(&data);
        write_stats(&mut entity.nbt, &stats);
        if let Some(delay) = int(&data, DELAY) {
            entity.nbt.insert(JavaString::from(DELAY), Tag::Int(delay));
        }
        if let Some(mob) = string(&data, MOB) {
            entity
                .nbt
                .insert(JavaString::from(MOB), Tag::String(mob.as_str().into()));
        }
    }
    let Some(mob) = string(&entity.nbt, MOB) else {
        return;
    };
    let stats = read_stats(&entity.nbt);
    let dimension = game.dim.id().to_owned();
    let center = [
        f64::from(pos[0]) + 0.5,
        f64::from(pos[1]) + 0.5,
        f64::from(pos[2]) + 0.5,
    ];
    let player_near = game.alive_player_near(
        center[0],
        center[1],
        center[2],
        f64::from(stats.player_range),
    );
    let powered = game.native_neighbor_signal(&dimension, pos) > 0;
    if !activated(
        stats.ignore_players,
        player_near,
        stats.redstone_control,
        powered,
    ) {
        return;
    }
    let delay = int(&entity.nbt, DELAY).unwrap_or(20);
    let span = (stats.max_delay - stats.min_delay).max(1);
    let roll = game.world.random.next_int_bound(span).unwrap_or(0);
    let (delay, due) = next_delay(delay, stats.min_delay, stats.max_delay, roll);
    entity.nbt.insert(JavaString::from(DELAY), Tag::Int(delay));
    if !due {
        return;
    }
    let nearby = game
        .native_mobs(&dimension)
        .into_iter()
        .filter(|m| m.kind == mob.as_str())
        .filter(|m| {
            in_spawn_area(
                pos,
                [
                    m.pos[0].floor() as i32,
                    m.pos[1].floor() as i32,
                    m.pos[2].floor() as i32,
                ],
                stats.spawn_range,
            )
        })
        .count();
    if nearby >= usize::try_from(stats.max_nearby.max(1)).unwrap_or(1) {
        return;
    }
    let height = game
        .res
        .mob_types()
        .get(&mob)
        .map_or(1.0_f64, |kind| kind.height)
        .ceil() as i32;
    let cells = height.clamp(1, 3);
    let mut spawned = 0;
    for _ in 0..stats.spawn_count.max(1) {
        let span = 2 * stats.spawn_range.max(0) + 1;
        let dx = game.world.random.next_int_bound(span).unwrap_or(0) - stats.spawn_range;
        let dy = game.world.random.next_int_bound(3).unwrap_or(0) - 1;
        let dz = game.world.random.next_int_bound(span).unwrap_or(0) - stats.spawn_range;
        let target = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
        if !game.world.loaded(target[0], target[2]) {
            continue;
        }
        if (0..cells).any(|up| {
            let state = game.world.block(target[0], target[1] + up, target[2]);
            !game.res.facts().is_air(state)
        }) {
            continue;
        }
        let options = crate::game::native_mobs::NativeSpawn {
            no_ai: stats.no_ai,
            silent: stats.silent,
            ..Default::default()
        };
        let at = [
            f64::from(target[0]) + 0.5,
            f64::from(target[1]),
            f64::from(target[2]) + 0.5,
        ];
        let Some(id) = game.native_spawn_mob(&dimension, &mob, at, &options) else {
            continue;
        };
        if let Some(spawned_mob) = game.mobs.iter_mut().find(|m| m.id == id) {
            if stats.youthful {
                set_baby(spawned_mob, true);
            }
            if stats.burning && !spawned_mob.kind.flags.fire_immune {
                spawned_mob.fire_ticks = 32_000;
            }
            if stats.initial_health < 1.0 {
                spawned_mob.health = health_for(spawned_mob.attrs.max_health, stats.initial_health);
            }
        }
        spawned += 1;
    }
    if spawned > 0 {
        game.native_block_entity_sync(pos);
    }
}

/// A right click on our spawner: a spawn egg sets its mob, a modifier item
/// changes one stat (quartz in the offhand inverts it). Our item used on a
/// vanilla spawner converts it in place, keeping its mob and stats.
#[allow(private_interfaces)]
pub fn on_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    _face: [i32; 3],
    claimed: &mut bool,
) {
    let key = (pos[0], pos[1], pos[2]);
    let clicked = game.world.block(pos[0], pos[1], pos[2]);
    let clicked_name = game.res.facts().get(clicked).name.clone();

    if clicked_name.as_ref() == SPAWNER {
        if stack.item.ends_with("_spawn_egg") {
            let (kind, _) =
                block_entity_item::spawn_egg_on_spawner(stack, game.acting_player_is_op());
            if !game.res.mob_types().contains_key(&kind) {
                return;
            }
            let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get_mut(&key) else {
                return;
            };
            if entity.id != SPAWNER {
                return;
            }
            entity
                .nbt
                .insert(JavaString::from(MOB), Tag::String(kind.as_str().into()));
            *claimed = true;
            game.consume_held();
            game.native_block_entity_sync(pos);
            return;
        }
        let change = change_for(&stack.item, game.offhand.as_ref().map(|s| s.item.as_ref()));
        let Some(change) = change else {
            return;
        };
        let changed = {
            let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get_mut(&key) else {
                return;
            };
            if entity.id != SPAWNER {
                return;
            }
            let mut stats = read_stats(&entity.nbt);
            if !apply(&mut stats, change) {
                false
            } else {
                write_stats(&mut entity.nbt, &stats);
                true
            }
        };
        if changed {
            *claimed = true;
            game.consume_held();
            game.native_block_entity_sync(pos);
        }
        return;
    }

    if clicked_name.as_ref() == VANILLA_SPAWNER
        && stack.item.as_ref() == SPAWNER
        && convert_vanilla(game, pos)
    {
        *claimed = true;
        game.consume_held();
        game.native_block_entity_sync(pos);
    }
}

/// Replace a vanilla spawner with ours, keeping its mob and its vanilla
/// delays/counts/ranges. Returns whether the block changed.
fn convert_vanilla(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> bool {
    let key = (pos[0], pos[1], pos[2]);
    let converted = match game.world.block_entities.get(&key) {
        Some(BlockEntity::Spawner(spawner)) => Some((
            spawner.next_data().and_then(|data| data.id()),
            Stats {
                min_delay: spawner.min_spawn_delay,
                max_delay: spawner.max_spawn_delay,
                spawn_count: spawner.spawn_count,
                max_nearby: spawner.max_nearby_entities,
                player_range: spawner.required_player_range,
                spawn_range: spawner.spawn_range,
                ..Stats::default()
            },
            spawner.spawn_delay,
        )),
        _ => None,
    };
    let Some((mob, stats, delay)) = converted else {
        return false;
    };
    let Some(state) = game.pack.assets.registry.default_state(SPAWNER) else {
        return false;
    };
    let dimension = game.dim.id().to_owned();
    if !game.native_set_block(&dimension, pos, state, flags::UPDATE_ALL) {
        return false;
    }
    if let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get_mut(&key) {
        write_stats(&mut entity.nbt, &stats);
        entity.nbt.insert(JavaString::from(DELAY), Tag::Int(delay));
        if let Some(mob) = mob {
            entity
                .nbt
                .insert(JavaString::from(MOB), Tag::String(mob.as_str().into()));
        }
    }
    true
}

/// A player placed our block: carry the stats a dropped item kept in
/// `minecraft:custom_data`.
#[allow(private_interfaces)]
pub fn on_place_block(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    if game.res.facts().get(state).name.as_ref() != SPAWNER {
        return;
    }
    let Some(Tag::Compound(custom)) = stack
        .data()
        .and_then(|data| data.other("minecraft:custom_data"))
    else {
        return;
    };
    if !matches!(
        game.world.block_entities.get(&(pos[0], pos[1], pos[2])),
        Some(BlockEntity::Mod(_))
    ) {
        pending_map()
            .get_or_insert_with(HashMap::new)
            .insert((game.dim.id().to_owned(), pos), custom.clone());
        return;
    }
    let Some(BlockEntity::Mod(entity)) =
        game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
    else {
        return;
    };
    let stats = read_stats(custom);
    write_stats(&mut entity.nbt, &stats);
    if let Some(delay) = int(custom, DELAY) {
        entity.nbt.insert(JavaString::from(DELAY), Tag::Int(delay));
    }
    if let Some(mob) = string(custom, MOB) {
        entity
            .nbt
            .insert(JavaString::from(MOB), Tag::String(mob.as_str().into()));
    }
}

/// Before a player breaks a spawner, keep its data for the silk-touch drop.
/// A vanilla spawner is read into our stats, so breaking one with silk touch
/// gives an Apothic Spawner that keeps its mob and vanilla settings.
#[allow(private_interfaces)]
pub fn on_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    let name = game.res.facts().get(state).name.clone();
    let key = (pos[0], pos[1], pos[2]);
    let data = if name.as_ref() == SPAWNER {
        match game.world.block_entities.get(&key) {
            Some(BlockEntity::Mod(entity)) if entity.id == SPAWNER => entity.nbt.clone(),
            _ => return,
        }
    } else if name.as_ref() == VANILLA_SPAWNER {
        let Some(BlockEntity::Spawner(spawner)) = game.world.block_entities.get(&key) else {
            return;
        };
        let stats = Stats {
            min_delay: spawner.min_spawn_delay,
            max_delay: spawner.max_spawn_delay,
            spawn_count: spawner.spawn_count,
            max_nearby: spawner.max_nearby_entities,
            player_range: spawner.required_player_range,
            spawn_range: spawner.spawn_range,
            ..Stats::default()
        };
        let mut data = Compound::new();
        write_stats(&mut data, &stats);
        data.insert(JavaString::from(DELAY), Tag::Int(spawner.spawn_delay));
        if let Some(mob) = spawner.next_data().and_then(|next| next.id()) {
            data.insert(JavaString::from(MOB), Tag::String(mob.as_str().into()));
        }
        data
    } else {
        return;
    };
    let dimension = game.dim.id().to_owned();
    snapshot_map()
        .get_or_insert_with(HashMap::new)
        .insert((dimension, pos), data);
}

/// A block's drops are rolled: a silk-touch break of our spawner gives the
/// block back with its stats in `minecraft:custom_data`, as upstream's
/// `getDrops` does.
#[allow(private_interfaces)]
pub fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    let Some((pos, state)) = context.block else {
        return;
    };
    let name = game.res.facts().get(state).name.clone();
    if name.as_ref() != SPAWNER && name.as_ref() != VANILLA_SPAWNER {
        return;
    }
    if context
        .tool
        .is_none_or(|tool| tool.enchantment_level("minecraft:silk_touch") <= 0)
    {
        return;
    }
    let mut stack = ItemStack::new(SPAWNER, 1);
    let key = (context.dimension.to_owned(), pos);
    let snapshot = snapshot_map()
        .get_or_insert_with(HashMap::new)
        .remove(&key)
        .or_else(|| last_map().get_or_insert_with(HashMap::new).remove(&key));
    if let Some(data) = snapshot {
        let mut extra = horizon_client::item_data::StackData::default();
        extra.set_other("minecraft:custom_data", Some(Tag::Compound(data)));
        stack.extra = Some(Box::new(extra));
    }
    drops.push(stack);
}
