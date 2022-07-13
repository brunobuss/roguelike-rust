use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn entity_render(ecs: &SubWorld, #[resource] camera: &Camera) {
    let mut renderables = <(&Point, &Render)>::query();
    let mut player = <(&FieldOfView, &Point, &Render)>::query().filter(component::<Player>());
    let (player_fov, player_pos, player_render) = player.iter(ecs).next().unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);

    // Render everything but the player first.
    renderables
        .iter(ecs)
        .filter(|(pos, _)| player_fov.visible_tiles.contains(pos) && *pos != player_pos)
        .for_each(|(pos, render)| {
            draw_batch.set(*pos - offset, render.color, render.glyph);
        });

    // Then draw the player, to avoid being "under" items in the screen.
    draw_batch.set(
        *player_pos - offset,
        player_render.color,
        player_render.glyph,
    );

    draw_batch.submit(5000).expect("Batch error");
}
