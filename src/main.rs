use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike tutorial")
        .build()?;
    let mut gs = State {
      ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    // gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    let (rooms, map) = new_map_rooms_and_corridors2();
    gs.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position{ x: player_x, y: player_y})
        .with(Renderable {
          glyph: rltk::to_cp437('@'),
          fg: RGB::named(rltk::YELLOW),
          bg: RGB::named(rltk::BLACK)
        })
        .with(Player{})
        .build();
    rltk::main_loop(context, gs)
}

struct State {
  ecs: World
}
// impl State {
//     fn run_systems(&mut self) {
//       let mut lw = LeftWalker{};
//       lw.run_now(&self.ecs);
//       self.ecs.maintain();
//     }
// }
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
      ctx.cls();

      player_input(self, ctx);
      // self.run_systems();

      let map = self.ecs.fetch::<Vec<TileType>>();
      draw_map(&map, ctx);
      
      let position = self.ecs.read_storage::<Position>();
      let renderable = self.ecs.read_storage::<Renderable>();

      for (pos, render) in (&position, &renderable).join() {
        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
      }
    }
}







