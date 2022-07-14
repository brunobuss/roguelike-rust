use crate::prelude::*;

use legion::systems::CommandBuffer;
use ron::de::from_reader;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;

#[derive(Clone, Deserialize, Debug)]
pub struct Template {
    pub entity_type: EntityType,
    pub levels: HashSet<usize>,
    pub frequency: i32,
    pub name: String,
    pub glyph: char,
    pub provides: Option<Vec<(String, i32)>>,
    pub hp: Option<i32>,
    pub base_damage: Option<i32>,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum EntityType {
    Enemy,
    Item,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Templates {
    pub entities: Vec<Template>,
}

impl Templates {
    pub fn load() -> Self {
        let file = File::open("resources/template.ron").expect("Failed opening file");
        from_reader(file).expect("Unable to load templates")
    }

    pub fn spawn_entities(
        &self,
        ecs: &mut World,
        resources: &mut Resources,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_points: &[Point],
    ) {
        let mut available_entities = Vec::new();
        self.entities
            .iter()
            .filter(|e| e.levels.contains(&level))
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_entities.push(t);
                }
            });

        let mut commands = CommandBuffer::new(ecs);
        spawn_points.iter().for_each(|pt| {
            if let Some(entity) = rng.random_slice_entry(&available_entities) {
                self.spawn_entity(pt, entity, &mut commands);
            }
        });
        commands.flush(ecs, resources);
    }

    fn spawn_entity(&self, pt: &Point, template: &Template, commands: &mut CommandBuffer) {
        let entity = commands.push((
            *pt,
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
        ));

        match template.entity_type {
            EntityType::Item => commands.add_component(entity, Item {}),
            EntityType::Enemy => {
                commands.add_component(entity, Enemy {});
                commands.add_component(entity, FieldOfView::new(6));
                commands.add_component(entity, ChasingPlayer {});
                commands.add_component(
                    entity,
                    Health {
                        current: template.hp.unwrap(),
                        max: template.hp.unwrap(),
                    },
                );
            }
        }

        if let Some(effects) = &template.provides {
            effects
                .iter()
                .for_each(|(provides, n)| match provides.as_str() {
                    "Healing" => commands.add_component(entity, ProvidesHealing { amount: *n }),
                    "MagicMap" => commands.add_component(entity, ProvidesDungeonMap {}),
                    _ => println!("Warning: Unknown provider for {}", provides),
                });
        }
        if let Some(damage) = &template.base_damage {
            commands.add_component(entity, Damage(*damage));
            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon {});
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_item(frequency: i32) -> Template {
        let mut levels = HashSet::new();
        levels.insert(2 as usize);
        levels.insert(3 as usize);

        Template {
            entity_type: EntityType::Item,
            levels,
            frequency,
            name: "Test Item".to_string(),
            glyph: '!',
            provides: None,
            hp: None,
            base_damage: None,
        }
    }

    fn build_test_enemy(frequency: i32) -> Template {
        let mut levels = HashSet::new();
        levels.insert(2 as usize);
        levels.insert(3 as usize);

        Template {
            entity_type: EntityType::Enemy,
            levels,
            frequency,
            name: "Test Enemy".to_string(),
            glyph: 'O',
            provides: None,
            hp: Some(5),
            base_damage: Some(2),
        }
    }

    #[test]
    fn spawn_nothing() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();

        let templates = Templates {
            entities: vec![build_test_item(1)],
        };
        let spawn = [];

        templates.spawn_entities(&mut ecs, &mut resources, &mut rng, 2 as usize, &spawn);
        assert_eq!(ecs.len(), 0);
    }

    #[test]
    fn spawn_one_entity_wrong_level() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();

        let templates = Templates {
            entities: vec![build_test_item(1)],
        };
        let spawn = [Point::new(10, 10)];

        templates.spawn_entities(&mut ecs, &mut resources, &mut rng, 1 as usize, &spawn);
        assert_eq!(ecs.len(), 0);
    }

    #[test]
    fn spawn_one_entity_no_frequency() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();

        let templates = Templates {
            entities: vec![build_test_item(0)],
        };
        let spawn = [Point::new(10, 10)];

        templates.spawn_entities(&mut ecs, &mut resources, &mut rng, 2 as usize, &spawn);
        assert_eq!(ecs.len(), 0);
    }

    #[test]
    fn spawn_one_consumable() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();

        let templates = Templates {
            entities: vec![build_test_item(1)],
        };
        let spawn = [Point::new(10, 10)];

        templates.spawn_entities(&mut ecs, &mut resources, &mut rng, 2 as usize, &spawn);
        assert_eq!(ecs.len(), 1);
    }

    #[test]
    fn spawn_one_enemy() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();

        let templates = Templates {
            entities: vec![build_test_enemy(1)],
        };
        let spawn = [Point::new(10, 10)];

        templates.spawn_entities(&mut ecs, &mut resources, &mut rng, 2 as usize, &spawn);
        assert_eq!(ecs.len(), 1);
    }
}
