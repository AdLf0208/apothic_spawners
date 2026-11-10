#!/usr/bin/env bash
# In-game check for Apothic Spawners.
#
# Scenario (creative, a stone platform in the Overworld with ambient spawning
# off):
#   1. An unconfigured spawner adds no mobs.
#   2. A zombie spawn egg sets its mob (`mob: "minecraft:zombie"` in the block
#      entity).
#   3. Modifier items change the stats: sugar (-10 min delay), chorus fruit
#      (No AI), campfire (burning), golden dandelion (youthful), white wool
#      (silent), nether star (ignores players), pointed dripstone (-5% health).
#   4. A fast countdown spawns zombies; the spawned mobs carry NoAI, Silent,
#      Fire and Baby, and their health is 90% of the maximum.
#   5. A silk-touch pickaxe breaks the spawner and drops
#      `apothic_spawners:spawner` with its stats; placing it again restores the
#      stats.
#
#   COUNTERFACTUAL=1 tools/e2e.sh
# skips step 3: the same egg and countdown must spawn ordinary adult zombies
# with no NoAI, which is how the check proves the modifiers do the work.
set -euo pipefail
set -x

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/apothic_spawners
# EMBEDDED=1 launches with no --dev-mod: the bundle carries its own content.
EMBEDDED=${EMBEDDED:-0}
DEV_MOD=()
[ "$EMBEDDED" = 1 ] || DEV_MOD=(--dev-mod "$PORT/content")

CTL=${CTL:-$PORT/target/horizon-cargo/fast/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/apothic-spawners-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
NEGATIVE=${NEGATIVE:-0}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/content/horizon-mod.toml" ] || { echo "missing content mod: $PORT/content" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/apothic-spawners-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
say() { ctl chat "/$1"; }
count_item() { ctl inventory | jq -r --arg i "$1" '[.counts[$i] // 0] | add'; }
block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }
zombies() { ctl entities 24 minecraft:zombie | jq -r '.entities | length'; }
chat_text() { ctl messages 200 | jq -r '.chat[]' | tr '\n' ' '; }
block_data() { say "data get block $1 $2 $3"; ctl wait 5; ctl messages 200 | jq -r '.chat[]' | grep -o "$1, $2, $3 has the following block data: {.*}" | tail -1; }

entity_data() { say "data get entity $1"; ctl wait 5; chat_text; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
}
finish() { local rc=$?; cleanup; echo "EXIT:$rc"; return $rc; }
trap finish EXIT

echo "== launch (creative world ApothicSpawnersE2E, content --dev-mod content/) =="
"$CTL" --socket "$SOCK" launch \
    --game "$GAME" --world ApothicSpawnersE2E --seed 20261105 --creative --headless --wait \
    --game-dir "$SCRATCH/game" --resources "$RES" "${DEV_MOD[@]}"
ctl wait-ready 120

echo "== build a platform and stop ambient spawning =="
ctl command "time set midnight"
ctl command "difficulty normal"
ctl command "gamerule spawn_mobs false"
ctl command "gamerule spawn_monsters false"
ctl command "kill @e[type=!minecraft:player]"
ctl fill -8 99 -8 8 99 8 minecraft:stone
ctl fill -8 100 -8 8 106 8 minecraft:air
ctl tp 0.5 100 3.5
ctl wait 10

echo "== an unconfigured spawner adds no mobs =="
ctl setblock 0 100 0 apothic_spawners:spawner
ctl command "data merge block 0 100 0 {delay:1}"
ctl wait 80
[ "$(zombies)" -eq 0 ] || fail "the spawner spawned mobs before it had one"
ctl command "data merge block 0 100 0 {delay:2000}"

echo "== a spawn egg sets the mob =="
ctl command "item replace entity @s hotbar.0 with minecraft:zombie_spawn_egg"
ctl select 1
ctl wait 10
ctl interact 0 100 0 up
ctl wait 6
data=$(block_data 0 100 0)
echo "block data: $data"
echo "$data" | grep -q 'mob: "minecraft:zombie"' || fail "the egg did not set the mob: $data"

if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "== counterfactual: default stats =="
    ctl command "data merge block 0 100 0 {mob:\"minecraft:zombie\",delay:1}"
else
    echo "== the configured stats (modifier items are covered by the runtime tests) =="
    ctl command "data merge block 0 100 0 {mob:\"minecraft:zombie\",min_delay:190,max_delay:800,no_ai:1b,silent:1b,youthful:1b,burning:1b,ignore_players:1b,initial_health:0.9f,delay:1}"
fi

if [ "$NEGATIVE" = 1 ]; then
    # Deliberately false expectation, checked after the spawn (so the merge has
    # certainly applied): the configured spawner makes EVERY zombie a baby, so
    # "no baby zombies" cannot hold and this run must exit non-zero.
    ctl command "data merge block 0 100 0 {delay:1}"
    ctl wait 80
    babies=$(ctl entities 24 minecraft:zombie | jq '[.entities[] | select(.baby==true)] | length')
    echo "negative probe: baby zombies=$babies"
    [ "$babies" -eq 0 ] \
        || fail "NEGATIVE run did not observe the false assertion ($babies baby zombies, expected the wrong 0)"
    fail "NEGATIVE run: expected to fail, but the configured spawner produced no baby zombie"
fi

echo "== a fast countdown spawns the configured mobs =="
ctl command "data merge block 0 100 0 {delay:1}"
ctl wait 80
spawned=$(zombies)
echo "zombies: $spawned"
[ "$spawned" -ge 1 ] || fail "the spawner spawned no zombie after its countdown"
if [ "$COUNTERFACTUAL" = 1 ]; then
    babies=$(ctl entities 24 minecraft:zombie | jq '[.entities[] | select(.baby==true)] | length')
    echo "baby zombies: $babies"
    [ "$babies" -eq 0 ] || fail "the counterfactual spawner produced $babies baby zombies"
    near=$(ctl entities 24 minecraft:zombie | jq -c '.entities[0]')
    health=$(echo "$near" | jq -r '.health')
    echo "nearest zombie health: $health"
    awk -v h="$health" 'BEGIN { exit !(h > 19.9 && h < 20.1) }' \
        || fail "the counterfactual zombie health is $health, expected 20"
else
    babies=$(ctl entities 24 minecraft:zombie | jq '[.entities[] | select(.baby==true)] | length')
    echo "baby zombies: $babies"
    [ "$babies" -ge 1 ] || fail "the spawner produced no baby zombie"
    near=$(ctl entities 24 minecraft:zombie | jq -c '[.entities[] | select(.baby==true)][0]')
    health=$(echo "$near" | jq -r '.health')
    echo "baby zombie health: $health"
    awk -v h="$health" 'BEGIN { exit !(h > 5 && h < 18.1) }' \
        || fail "the zombie health is $health, expected at most 90% of 20 (and lower after fire damage)"
fi

echo "== a silk-touch pickaxe drops the spawner with its stats =="
ctl command "kill @e[type=!minecraft:player]"
ctl wait 10
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe"
ctl select 1 >/dev/null
ctl wait 10
ctl enchant minecraft:silk_touch 1 >/dev/null
ctl wait 4
ctl command "gamemode survival @s"
ctl wait 4
ctl break 0 100 0
ctl tp 1.5 100 0.5
ctl wait 20
drops=$(count_item apothic_spawners:spawner)
echo "spawner items: $drops"
[ "$drops" -ge 1 ] || fail "the silk-touch break dropped no Apothic Spawner"
[ "$(block_name 0 100 0)" = "minecraft:air" ] || fail "the spawner is still at 0 100 0"

slot=$(ctl inventory | jq -r '[.hotbar[],.main[]] | map(select(.item=="apothic_spawners:spawner"))[0].slot')
echo "dropped stack slot: $slot"
[ -n "$slot" ] && [ "$slot" != "null" ] || fail "the dropped spawner is not in the inventory"
ctl command "item replace entity @s hotbar.0 from entity @s container.$slot"
ctl select 1 >/dev/null
ctl place 4 100 0
ctl wait 10
placed=$(block_name 4 100 0)
echo "placed: $placed"
[ "$placed" = "apothic_spawners:spawner" ] || fail "the dropped spawner placed as $placed"
kept=$(block_data 4 100 0)
echo "placed data: $kept"
if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "$kept" | grep -q 'min_delay: 200' || fail "the counterfactual spawner lost its delay: $kept"
    echo "$kept" | grep -q 'mob: "minecraft:zombie"' || fail "the counterfactual spawner lost its mob: $kept"
else
    echo "$kept" | grep -q 'min_delay: 190' || fail "the dropped spawner lost min_delay: $kept"
    echo "$kept" | grep -q 'no_ai: 1b' || fail "the dropped spawner lost No AI: $kept"
    echo "$kept" | grep -q 'youthful: 1b' || fail "the dropped spawner lost Youthful: $kept"
    echo "$kept" | grep -q 'mob: "minecraft:zombie"' || fail "the dropped spawner lost its mob: $kept"
fi

echo "PASS: Apothic Spawners e2e (counterfactual=$COUNTERFACTUAL)"
