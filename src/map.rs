use rltk::{ RGB, RandomNumberGenerator, Algorithm2D, Point, BaseMap, DrawBatch, ColorPair };
use object_pool::{Reusable};
use super::{Rect};
use std::cmp::{max, min};
use specs::prelude::*;
use rltk::{console};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub layers: Vec<Layer>
}

const MAPWIDTH: i32 = 80;
const MAPHEIGHT: i32 = 50;
const MAPCOUNT: usize = MAPHEIGHT as usize * MAPWIDTH as usize;

#[derive(Serialize, Deserialize)]
pub struct Layer {
    name: String,
    data: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonMap {
    compressionlevel: i32,
    height: i32,
    infinite: bool,
    layers: Vec<Layer>,
}

rltk::embedded_resource!(RAW_FILE, "../resources/map.json");


impl Map {

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 +1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }


    fn load_map(&mut self) {
        rltk::link_resource!(RAW_FILE, "../resources/map.json");

        // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
        let raw_data = rltk::embedding::EMBED
            .lock()
            .get_resource("../resources/map.json".to_string())
            .unwrap();
        let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
        let decoder: JsonMap = serde_json::from_str(&raw_string).expect("Unable to parse JSON");

        self.layers = decoder.layers;
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.

    pub fn load() -> Map {
        let mut map = Map{
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            layers: Vec::new(),
            width: MAPWIDTH,
            height: MAPHEIGHT,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT]
        };

        map.load_map();

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;

            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }

            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

pub fn draw_map(ecs: &World, draw_batch: &mut Reusable<'_, DrawBatch>) {
    let map = ecs.fetch::<Map>();

    draw_batch.target(0);
    draw_batch.cls();

    let mut y = 0;
    let mut x = 0;
    for glyph in &map.layers[0].data {

        draw_batch.set(Point::new(x, y), ColorPair::new(RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0., 0., 0.)), *glyph);

        println!("gluph {}", glyph);

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}