use super::MapArchitect;
use crate::prelude::*;

const STAGGER_DISTANCE: usize = 400;
const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;
const DESIRED_FLOOR: usize = NUM_TILES / 3;

pub struct DrunkardWalkArchitect {}

impl DrunkardWalkArchitect {
    fn drunkard(&mut self, start: &Point, rng: &mut RandomNumberGenerator, map: &mut Map) {
        let mut drunkard_pos = *start;
        let mut distance_staggered = 0;

        loop {
            let drunk_idx = map.point2d_to_index(drunkard_pos);
            // Break if arrived in a border (or is fully out of bounds)
            if !map.in_bounds(drunkard_pos)
                || drunkard_pos.x == 0
                || drunkard_pos.x == SCREEN_WIDTH - 1
                || drunkard_pos.y == 0
                || drunkard_pos.y == SCREEN_HEIGHT - 1
            {
                break;
            }

            map.tiles[drunk_idx] = TileType::Floor;
            match rng.range(0, 4) {
                0 => drunkard_pos.x -= 1,
                1 => drunkard_pos.x += 1,
                2 => drunkard_pos.y -= 1,
                _ => drunkard_pos.y += 1,
            }

            distance_staggered += 1;
            if distance_staggered > STAGGER_DISTANCE {
                break;
            }
        }
    }
}

impl MapArchitect for DrunkardWalkArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };

        mb.fill(TileType::Wall);
        let center = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
        self.drunkard(&center, rng, &mut mb.map);

        while mb
            .map
            .tiles
            .iter()
            .filter(|t| **t == TileType::Floor)
            .count()
            < DESIRED_FLOOR
        {
            self.drunkard(
                &Point::new(
                    rng.range(1, SCREEN_WIDTH - 1),
                    rng.range(1, SCREEN_HEIGHT - 1),
                ),
                rng,
                &mut mb.map,
            );

            let dijkstra_map = DijkstraMap::new(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                &[mb.map.point2d_to_index(center)],
                &mb.map,
                1024.0,
            );
            dijkstra_map
                .map
                .iter()
                .enumerate()
                .filter(|(_, distance)| *distance > &2000.0)
                .for_each(|(idx, _)| mb.map.tiles[idx] = TileType::Wall);
        }

        mb.player_start = center;
        mb.amulet_start = mb.find_most_distance();
        mb.monster_spawns = mb.spawn_monster(rng);

        mb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_properties() {
        let mut rng = RandomNumberGenerator::new();

        let mut architect = DrunkardWalkArchitect {};
        let mb = architect.new(&mut rng);

        // Check that both player start and amulet/exit points are floors.
        let player_start_idx = mb.map.point2d_to_index(mb.player_start);
        assert_eq!(mb.map.tiles[player_start_idx], TileType::Floor);
        let exit_idx = mb.map.point2d_to_index(mb.amulet_start);
        assert_eq!(mb.map.tiles[exit_idx], TileType::Floor);

        // Check that there is a path from player to exit.
        let dmap = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &[player_start_idx],
            &mb.map,
            1024.0,
        );
        assert_ne!(dmap.map[exit_idx], f32::MAX);
    }
}
