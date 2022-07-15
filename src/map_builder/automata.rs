use super::MapArchitect;
use crate::prelude::*;

pub struct CellularAutomataArchitect {}

impl CellularAutomataArchitect {
    fn random_noise_map(&mut self, rng: &mut RandomNumberGenerator, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        new_tiles.iter_mut().enumerate().for_each(|(idx, t)| {
            let pt = map.index_to_point2d(idx);
            if pt.x == 0 || pt.x == SCREEN_WIDTH - 1 || pt.y == 0 || pt.y == SCREEN_HEIGHT - 1 {
                // Always make the borders be Walls.
                *t = TileType::Wall
            } else {
                let roll = rng.range(0, 100);
                *t = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                };
            }
        });
        map.tiles = new_tiles;
    }

    /// Count number of adjacent (incl diagonals) Wall tiles.
    fn count_neighbors(&self, x: i32, y: i32, map: &Map) -> usize {
        let mut neighbors = 0;
        for iy in -1..=1 {
            for ix in -1..=1 {
                if !(ix == 0 && iy == 0) && map.tiles[map_idx(x + ix, y + iy)] == TileType::Wall {
                    neighbors += 1;
                }
            }
        }
        neighbors
    }

    fn iteration(&mut self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        for y in 1..SCREEN_HEIGHT - 1 {
            for x in 1..SCREEN_WIDTH - 1 {
                let neighbors = self.count_neighbors(x, y, map);
                let idx = map_idx(x, y);
                new_tiles[idx] = if neighbors > 4 || neighbors == 0 {
                    TileType::Wall
                } else {
                    TileType::Floor
                };
            }
        }
        map.tiles = new_tiles;
    }

    fn find_start(&self, map: &Map) -> Point {
        let center = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
        let closest_point = map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(idx, _)| {
                (
                    idx,
                    DistanceAlg::Pythagoras.distance2d(center, map.index_to_point2d(idx)),
                )
            })
            .min_by(|(_, distance), (_, distance2)| distance.partial_cmp(distance2).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();
        map.index_to_point2d(closest_point)
    }
}

impl MapArchitect for CellularAutomataArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };

        self.random_noise_map(rng, &mut mb.map);
        for _ in 0..10 {
            self.iteration(&mut mb.map)
        }
        let start = self.find_start(&mb.map);

        mb.player_start = start;
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

        let mut architect = CellularAutomataArchitect {};
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
