use crate::prelude::*;

/// This systems performs end of turns checks, in order to set the next state in the state machine.
#[system]
#[read_component(Health)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(AmuletOfYala)]
pub fn end_turn(ecs: &SubWorld, #[resource] turn_state: &mut TurnState, #[resource] map: &Map) {
    let mut player_hp = <(&Health, &Point)>::query().filter(component::<Player>());
    let mut amulet = <&Point>::query().filter(component::<AmuletOfYala>());

    let amulet_default = Point::new(-1, -1);
    let amulet_pos = amulet.iter(ecs).next().unwrap_or(&amulet_default);

    let current_state = *turn_state;
    let mut new_state = match current_state {
        TurnState::AwaitingInput => return,
        TurnState::PlayerTurn => TurnState::MonsterTurn,
        TurnState::MonsterTurn => TurnState::AwaitingInput,
        _ => current_state,
    };

    player_hp.iter(ecs).for_each(|(hp, pos)| {
        if hp.current < 1 {
            new_state = TurnState::GameOver;
        }
        if pos == amulet_pos {
            new_state = TurnState::Victory;
        }
        let idx = map.point2d_to_index(*pos);
        if map.tiles[idx] == TileType::Exit {
            new_state = TurnState::NextLevel;
        }
    });

    *turn_state = new_state;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awaiting_input_stays() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let map = Map::new();
        let player_pos = Point::new(10, 10);
        resources.insert(map);
        resources.insert(TurnState::AwaitingInput);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 10,
                max: 10,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 1);
        assert_eq!(
            *resources.get::<TurnState>().unwrap(),
            TurnState::AwaitingInput
        );
    }

    #[test]
    fn monster_turn_after_player_turn() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let map = Map::new();
        let player_pos = Point::new(10, 10);
        resources.insert(map);
        resources.insert(TurnState::PlayerTurn);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 10,
                max: 10,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 1);
        assert_eq!(
            *resources.get::<TurnState>().unwrap(),
            TurnState::MonsterTurn
        );
    }

    #[test]
    fn awaiting_input_after_monster_turn() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let map = Map::new();
        let player_pos = Point::new(10, 10);
        resources.insert(map);
        resources.insert(TurnState::MonsterTurn);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 10,
                max: 10,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 1);
        assert_eq!(
            *resources.get::<TurnState>().unwrap(),
            TurnState::AwaitingInput
        );
    }

    #[test]
    fn game_over_if_player_zero_health() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let map = Map::new();
        let player_pos = Point::new(10, 10);
        resources.insert(map);
        resources.insert(TurnState::PlayerTurn);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 0,
                max: 10,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 1);
        assert_eq!(*resources.get::<TurnState>().unwrap(), TurnState::GameOver);
    }

    #[test]
    fn game_win_if_player_found_amulet() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let map = Map::new();
        let player_pos = Point::new(10, 10);
        resources.insert(map);
        resources.insert(TurnState::PlayerTurn);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 10,
                max: 10,
            },
        ));
        ecs.push((player_pos, AmuletOfYala {}));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 2);
        assert_eq!(*resources.get::<TurnState>().unwrap(), TurnState::Victory);
    }

    #[test]
    fn next_level_if_player_found_stairs() {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut sched = Schedule::builder()
            .add_system(super::end_turn_system())
            .build();

        let mut map = Map::new();
        let player_pos = Point::new(10, 10);
        let idx = map.point2d_to_index(player_pos);
        map.tiles[idx] = TileType::Exit;
        resources.insert(map);
        resources.insert(TurnState::PlayerTurn);

        ecs.push((
            player_pos,
            Player { map_level: 0 },
            Health {
                current: 10,
                max: 10,
            },
        ));

        sched.execute(&mut ecs, &mut resources);
        assert_eq!(ecs.len(), 1);
        assert_eq!(*resources.get::<TurnState>().unwrap(), TurnState::NextLevel);
    }
}
