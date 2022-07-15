use crate::prelude::*;

/// This system updates all FieldOfViews that are marked as dirty.
#[system]
#[read_component(Point)]
#[write_component(FieldOfView)]
pub fn fov(ecs: &mut SubWorld, #[resource] map: &Map) {
    let mut views = <(&Point, &mut FieldOfView)>::query();
    views
        .iter_mut(ecs)
        .filter(|(_, fov)| fov.is_dirty)
        .for_each(|(pos, fov)| {
            fov.visible_tiles = field_of_view_set(*pos, fov.radius, map);
            fov.is_dirty = false;
        });
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn clean_fov_is_not_updated() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder().add_system(super::fov_system()).build();

        let map = Map::new();
        resources.insert(map);

        let e = ecs.push((
            Point::new(10, 10),
            FieldOfView {
                visible_tiles: HashSet::new(),
                radius: 1,
                is_dirty: false,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        let entry = ecs.entry(e).unwrap();
        let fov = entry.get_component::<FieldOfView>().unwrap();
        assert_eq!(fov.is_dirty, false);
        assert_eq!(fov.visible_tiles.is_empty(), true);
    }

    #[test]
    fn dirty_fov_is_updated() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder().add_system(super::fov_system()).build();

        let map = Map::new();
        let center = Point::new(10, 10);
        let mut expected_fov = HashSet::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                let delta = Point::new(dx, dy);
                let p = center + delta;
                expected_fov.insert(p);
            }
        }

        resources.insert(map);
        let e = ecs.push((
            center,
            FieldOfView {
                visible_tiles: HashSet::new(),
                radius: 1,
                is_dirty: true,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        let entry = ecs.entry(e).unwrap();
        let fov = entry.get_component::<FieldOfView>().unwrap();
        assert_eq!(fov.is_dirty, false);
        assert_eq!(fov.visible_tiles, expected_fov);
    }
}
