use rltk::{GameState, Rltk, RGB, Point};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::*;
mod monster_ai_system;
use monster_ai_system::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike tutorial")
        .build()?;
    let mut gs = State {
      ecs: World::new(),
      runstate: RunState::Running,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    // gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate(){
      let (x,y) = room.center();

      let glyph: rltk::FontCharType;
      let name: String;
      let roll = rng.roll_dice(1, 2);
      match roll {
          1 => {glyph = rltk::to_cp437('g'); name = String::from("Goblin")}
          _ => {glyph = rltk::to_cp437('o'); name = String::from("Orc")}
      }
      gs.ecs.create_entity()
            .with(Position{x, y})
            .with(Renderable{
              glyph: glyph,
              fg: RGB::named(rltk::RED),
              bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{visible_tiles: Vec::new(), range: 8, dirty: true})
            .with(Monster{})
            .with(Name{name: format!("{} #{}", &name, i)})
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));

    gs.ecs
        .create_entity()
        .with(Position{ x: player_x, y: player_y})
        .with(Renderable {
          glyph: rltk::to_cp437('@'),
          fg: RGB::named(rltk::YELLOW),
          bg: RGB::named(rltk::BLACK)
        })
        .with(Player{})
        .with(Viewshed{visible_tiles: Vec::new(), range: 8, dirty: true})
        .with(Name{name: String::from("Player")})
        .build();
    rltk::main_loop(context, gs)
}

struct State {
  ecs: World,
  runstate: RunState
}
impl State {
    fn run_systems(&mut self) {
      let mut vis = VisibilitySystem{};
      vis.run_now(&self.ecs);
      let mut mob = MonsterAI{};
      mob.run_now(&self.ecs);
      self.ecs.maintain();
    }
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
      ctx.cls();

      if self.runstate == RunState::Running {
        self.run_systems();
        self.runstate = RunState::Paused;
      } else {
        self.runstate = player_input(self, ctx);
      }

      // let map = self.ecs.fetch::<Vec<TileType>>();
      draw_map(&self.ecs, ctx);
      
      let position = self.ecs.read_storage::<Position>();
      let renderable = self.ecs.read_storage::<Renderable>();
      let map = self.ecs.fetch::<Map>();

      for (pos, render) in (&position, &renderable).join() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] {
          ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
        }
      }
    }
}


#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
  Paused,
  Running
}




