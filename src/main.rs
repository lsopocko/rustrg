use rltk::{Rltk, GameState, RGB, Point};
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
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod gui;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 50;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

pub struct State {
    pub ecs: World,
    pub runstate: RunState
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
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }

        // gui::draw_ui(&self.ecs, ctx);
    }
}

rltk::embedded_resource!(TILE_FONT, "../resources/dungeon_tiles_16x16.png");

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    rltk::link_resource!(TILE_FONT, "resources/dungeon_tiles_16x16.png");
    let context = RltkBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("Roguelike Tutorial")
        .with_tile_dimensions(16, 16)
        .with_font("dungeon_tiles_16x16.png", 16, 16)
        .with_simple_console(WIDTH, HEIGHT, "dungeon_tiles_16x16.png")
        .with_sparse_console_no_bg(WIDTH, HEIGHT, "dungeon_tiles_16x16.png")
        // .with_fullscreen(true)
        .build()?;

    let mut gs = State{
        ecs: World::new(),
        runstate: RunState::Running
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: 132,
            fg: RGB::from_f32(1.0, 1.0, 1.0),
            bg: RGB::from_f32(0., 0., 0.),
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true })
        .build();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();
        gs.ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{
                glyph: 160,
                fg: RGB::from_f32(1.0, 1.0, 1.0),
                bg: RGB::from_f32(0., 0., 0.),
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .with(Name{ name: format!("{} #{}", "Goblin", i) })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    rltk::main_loop(context, gs)
}