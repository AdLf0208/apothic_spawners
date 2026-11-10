// Runs at the head of `Game::native_loot_drops` (API 75): a silk-touch break
// of our spawner gives the block back with its stats.
fn observe_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &crate::game::native_hooks::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    crate::game::apothic_spawners_state::on_loot_drops(game, context, drops);
}
