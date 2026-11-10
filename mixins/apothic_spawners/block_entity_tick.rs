// Runs at the head of `Game::native_block_entity_tick` (API 17): the spawner
// counts down and adds its mobs.
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::apothic_spawners_state::on_block_entity_tick(game, pos, entity);
}
