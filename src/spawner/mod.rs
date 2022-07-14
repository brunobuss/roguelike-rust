mod template;

use crate::prelude::*;

use self::template::Templates;

pub fn spawn_player(ecs: &mut World, pos: Point) {
    ecs.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('@'),
        },
        Health {
            current: 10,
            max: 10,
        },
        FieldOfView::new(8),
        Damage(1),
    ));
}

pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item,
        AmuletOfYala,
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('|'),
        },
        Name("Amulet of Yala".to_string()),
    ));
}

pub fn spawn_level(
    ecs: &mut World,
    resources: &mut Resources,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_points: &[Point],
) {
    let template = Templates::load();
    template.spawn_entities(ecs, resources, rng, level, spawn_points);
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_spawn_player() {
        let mut ecs = World::default();
        let pos = Point::new(10, 10);

        spawn_player(&mut ecs, pos);
        assert_eq!(ecs.len(), 1);

        let player_entity = <Entity>::query().iter(&ecs).next().unwrap();
        let player_entry = ecs.entry(*player_entity).unwrap();
        assert!(player_entry.get_component::<Player>().is_ok());
        assert!(player_entry.get_component::<Point>().is_ok());
        assert_eq!(*player_entry.get_component::<Point>().unwrap(), pos);
        assert!(player_entry.get_component::<Render>().is_ok());
        assert!(player_entry.get_component::<Health>().is_ok());
        assert!(player_entry.get_component::<FieldOfView>().is_ok());
        assert!(player_entry.get_component::<Damage>().is_ok());
    }

    #[test]
    fn test_spawn_amulet_of_yala() {
        let mut ecs = World::default();
        let pos = Point::new(10, 10);

        spawn_amulet_of_yala(&mut ecs, pos);
        assert_eq!(ecs.len(), 1);

        let amulet_entity = <Entity>::query().iter(&ecs).next().unwrap();
        let amulet_entry = ecs.entry(*amulet_entity).unwrap();
        assert!(amulet_entry.get_component::<Item>().is_ok());
        assert!(amulet_entry.get_component::<Point>().is_ok());
        assert_eq!(*amulet_entry.get_component::<Point>().unwrap(), pos);
        assert!(amulet_entry.get_component::<Render>().is_ok());
        assert!(amulet_entry.get_component::<AmuletOfYala>().is_ok());
        assert!(amulet_entry.get_component::<Name>().is_ok());
    }
}
