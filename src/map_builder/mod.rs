mod automata;
mod drunkard;
mod empty;
mod prefab;
mod rooms;
mod themes;

use crate::prelude::*;
use automata::CellularAutomataArchitect;
use drunkard::DrunkardWalkArchitect;
use rooms::RoomsArchitect;

use self::{
    prefab::apply_prefab,
    themes::{DungeonTheme, ForestTheme},
};

pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
}

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub monster_spawns: Vec<Point>,
    pub player_start: Point,
    pub amulet_start: Point,
    pub theme: Box<dyn MapTheme>,
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut architect: Box<dyn MapArchitect> = match rng.range(0, 3) {
            0 => Box::new(DrunkardWalkArchitect {}),
            1 => Box::new(RoomsArchitect {}),
            _ => Box::new(CellularAutomataArchitect {}),
        };
        let mut mb = architect.new(rng);
        apply_prefab(&mut mb, rng);

        mb.theme = match rng.range(0, 2) {
            0 => DungeonTheme::new(),
            _ => ForestTheme::new(),
        };

        mb
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn find_most_distance(&self) -> Point {
        let dijkstra_map = self.build_player_distance_map();

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dijkstra_map
                .map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn build_player_distance_map(&self) -> DijkstraMap {
        DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &[self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        )
    }

    /// Should be called only after self.player_start is set.
    fn spawn_monster(&self, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        const UNREACHABLE: f32 = f32::MAX;
        let dijkstra_map = self.build_player_distance_map();
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            // Only spawn in Floors and in places that are reachable by the player
            // but not too close to the start position.
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && dijkstra_map.map[*idx] >= 10.0
                    && dijkstra_map.map[*idx] < UNREACHABLE
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        // TODO: We could shuffle spawnable_tiles and grab the first NUM_MONSTERS entries.
        for _ in 0..NUM_MONSTERS {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index]);
            spawnable_tiles.remove(target_index);
        }
        spawns
    }
}
