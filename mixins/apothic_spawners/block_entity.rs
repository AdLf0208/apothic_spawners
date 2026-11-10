// Runs at the head of `Game::native_block_entity` (API 15): our spawner gets
// its block entity, which keeps the mob, the countdown and every stat.
fn observe_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::apothic_spawners_state::on_block_entity(block, entity);
}
