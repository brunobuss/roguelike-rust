use crate::prelude::*;
use std::collections::HashSet;

#[system]
#[read_component(Player)]
#[read_component(Enemy)]
#[read_component(FieldOfView)]
#[read_component(WantsToMove)]
#[write_component(Point)]
pub fn movement(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
) {
    let mut occupied_spaces = HashSet::new();
    <&Point>::query()
        .filter(component::<Enemy>())
        .iter(ecs)
        .for_each(|p| {
            occupied_spaces.insert(*p);
        });
    <&Point>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .for_each(|p| {
            occupied_spaces.insert(*p);
        });

    let mut movers = <(Entity, &WantsToMove)>::query();
    movers.iter(ecs).for_each(|(e, want_move)| {
        if map.can_enter_tile(want_move.destination)
            && !occupied_spaces.contains(&want_move.destination)
        {
            occupied_spaces.remove(&want_move.from);
            occupied_spaces.insert(want_move.destination);
            commands.add_component(want_move.entity, want_move.destination);

            if let Ok(entry) = ecs.entry_ref(want_move.entity) {
                if let Ok(fov) = entry.get_component::<FieldOfView>() {
                    commands.add_component(want_move.entity, fov.clone_dirty());

                    if entry.get_component::<Player>().is_ok() {
                        camera.on_player_move(want_move.destination);
                        fov.visible_tiles
                            .iter()
                            .for_each(|pos| map.revealed_tiles[map_idx(pos.x, pos.y)] = true);
                    }
                }
            }
        }
        commands.remove(*e)
    });
}
