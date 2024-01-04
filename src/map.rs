use rltk::{ RGB, Rltk, RandomNumberGenerator, Tile, Algorithm2D, BaseMap, Point };
use specs::{World, WorldExt, Join};
use crate::{Viewshed, Player};

use super::{Rect};
use std::cmp::{max, min};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
  Wall, Floor
}
#[derive(Default)]
pub struct  Map {
  pub tiles: Vec<TileType>,
  pub rooms: Vec<Rect>,
  pub width: i32,
  pub height: i32,
  pub revealed_tiles: Vec<bool>,
  pub visible_tiles: Vec<bool>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
      (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
      for y in room.y1 + 1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
          let idx = self.xy_idx(x, y);
          self.tiles[idx] = TileType::Floor;
        }
      }
    }

    fn apply_horizontal_tunner(&mut self, x1: i32, x2: i32, y: i32) {
      for x in min(x1, x2) ..= max(x1, x2) {
        let idx = self.xy_idx(x, y);
        if idx > 0 && idx < self.width as usize * self.height as usize {
          self.tiles[idx as usize] = TileType::Floor;
        }
      }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
      for y in min(y1, y2) ..= max(y1, y2) {
        let idx = self.xy_idx(x, y);
        if idx > 0 && idx < self.width as usize * self.height as usize {
          self.tiles[idx as usize] = TileType::Floor;
        }
      }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
      let mut map = Map {
        tiles: vec![TileType::Wall; 80 * 50],
        rooms: Vec::new(),
        width: 80,
        height: 50,
        revealed_tiles: vec![false; 80 * 50],
        visible_tiles : vec![false; 80*50],
      };

      const MAX_ROOMS: i32 = 30;
      const MIN_SIZE: i32 = 6;
      const MAX_SIZE: i32 = 10;

      let mut rng = RandomNumberGenerator::new();

      for i in 0..MAX_ROOMS {
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
            let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
            if rng.range(0, 2) == 1 {
              map.apply_horizontal_tunner(prev_x, new_x, prev_y);
              map.apply_vertical_tunnel(prev_y, new_y, new_x);
            } else {
              map.apply_horizontal_tunner(prev_y, new_y, prev_x);
              map.apply_horizontal_tunner(prev_x, new_x, new_y);
            }
          }
          map.rooms.push(new_room);
        }
      }

      map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn xy_idx(x: i32, y: i32) -> usize {
  (y as usize * 80) + x as usize
}

pub fn new_map_test() -> Vec<TileType> {
  let mut map = vec![TileType::Floor; 80*50];

  //Make the boundaries walls
  for x in 0..80 {
    map[xy_idx(x, 0)] = TileType::Wall;
    map[xy_idx(x, 49)] = TileType::Wall;
  }

  for y in 0..50 {
    map[xy_idx(0, y)] = TileType::Wall;
    map[xy_idx(79, y)] = TileType::Wall;
  }

  //Now we'll randomly splat a bunch of walls/ It won't be pretty, but it's a decent illustration.
  //First, obtain the thread-local RNG:
  let mut rng = rltk::RandomNumberGenerator::new();

  for _i in 0..400 {
    let x = rng.roll_dice(1, 79);
    let y = rng.roll_dice(1, 49);
    let idx = xy_idx(x, y);
    if idx != xy_idx(40, 25){
      map[idx] = TileType::Wall;
    }
  }

  map
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
  let map = ecs.fetch::<Map>();
  let mut y = 0;
  let mut x = 0;
  
  for (idx, tile) in map.tiles.iter().enumerate() {
    if map.revealed_tiles[idx] {
      let glyph;
      let mut fg;
      match tile {
          TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.0, 0.5, 0.5);
          }
          TileType::Wall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::from_f32(0., 1.0, 0.);
          }
      }
      if !map.visible_tiles[idx] {fg = fg.to_greyscale()}
      ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
    }

    x += 1;
    if x > 79 {
      x = 0;
      y += 1;
    }
  }

}

pub fn draw_map_old(map:  &[TileType], ctx: &mut Rltk) {
  let mut y = 0;
  let mut x = 0;
  for tile in map.iter() {
    match tile {
        TileType::Floor => {
          ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
        }
        TileType::Wall => {
          ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'))
        }
    }
    x += 1;
    if x > 79 {
      x = 0;
      y += 1;
    }
  }
}

pub fn new_map_rooms_and_corridors() -> Vec<TileType> {
  let mut map = vec![TileType::Wall; 80 * 50];

  let room1 = Rect::new(20, 15, 10, 15);
  let room2 = Rect::new(35, 15, 10, 15);

  apply_room_to_map(&room1, &mut map);
  apply_room_to_map(&room2, &mut map);
  apply_horizontal_tunnel(&mut map, 25, 40, 23);
  map
}



fn apply_room_to_map(room: &Rect, map: &mut [TileType]){
  for y in room.y1 + 1 ..= room.y2 {
    for x in room.x1 + 1 ..= room.x2 {
      map[xy_idx(x, y)] = TileType::Floor
    }
  }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
  for x in min(x1, x2) ..= max(x1, x2) {
    let idx = xy_idx(x, y);
    if idx > 0 && idx < 80 * 50 {
      map[idx as usize] = TileType::Floor;
    }
  }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
  for y in min(y1, y2) ..= max(y1, y2) {
    let idx = xy_idx(x, y);
    if idx > 0 && idx < 80 * 50 {
      map[idx as usize] = TileType::Floor;
    }
  }
}

pub fn new_map_rooms_and_corridors2() -> (Vec<Rect>, Vec<TileType>) {
  let mut map = vec![TileType::Wall; 80*50];

  let mut rooms : Vec<Rect> = Vec::new();
  const MAX_ROOMS : i32 = 30;
  const MIN_SIZE : i32 = 6;
  const MAX_SIZE : i32 = 10;

  let mut rng = RandomNumberGenerator::new();

  for _ in 0..MAX_ROOMS {
      let w = rng.range(MIN_SIZE, MAX_SIZE);
      let h = rng.range(MIN_SIZE, MAX_SIZE);
      let x = rng.roll_dice(1, 80 - w - 1) - 1;
      let y = rng.roll_dice(1, 50 - h - 1) - 1;
      let new_room = Rect::new(x, y, w, h);
      let mut ok = true;
      for other_room in rooms.iter() {
          if new_room.intersect(other_room) { ok = false }
      }
      if ok {
          apply_room_to_map(&new_room, &mut map);        
          
          if !rooms.is_empty() {
            let (new_x, new_y) = new_room.center();
            let (prev_x, prev_y) = rooms[rooms.len()-1].center();
            if rng.range(0, 2) == 1 {
              apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
              apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
            } else {
              apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
              apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
            }
          }

          rooms.push(new_room);
      }
  }

  (rooms, map)
}