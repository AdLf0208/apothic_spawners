// Runs at the head of `Game::native_block_entity_ticks` (API 15): the
// spawner's block entity ticks.
fn observe_block_entity_ticks(block: &str, ticks: &mut bool) {
    crate::game::apothic_spawners_state::on_block_entity_ticks(block, ticks);
}
