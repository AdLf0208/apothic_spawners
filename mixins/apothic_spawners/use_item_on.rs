// Runs at the head of `Game::native_use_item_on` (API 13): a spawn egg sets
// the spawner's mob, a modifier item changes a stat, and our item converts a
// vanilla spawner in place.
fn observe_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::apothic_spawners_state::on_use_on(game, stack, pos, face, claimed);
}
