// Runs at the head of `Game::native_place_block` (API 14): a placed spawner
// takes the stats its dropped item kept.
fn observe_place_block(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    crate::game::apothic_spawners_state::on_place_block(game, stack, pos, state);
}
