// Runs at the head of `Game::native_destroy_block` (API 14): keep the
// spawner's data for the silk-touch drop.
fn observe_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    _canceled: &mut bool,
) {
    crate::game::apothic_spawners_state::on_destroy_block(game, pos, state);
}
