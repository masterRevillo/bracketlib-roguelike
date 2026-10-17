use bracket_lib::color::{BLACK, ORANGE, RGB};
use bracket_lib::prelude::to_cp437;
use bracket_lib::terminal::{DistanceAlg, Point};
use specs::prelude::*;

use crate::components::WeaponAttribute::Might;
use crate::components::{LightSource, Position, Viewshed};
use crate::gamelog::GameLog;
use crate::gamesystem::skill_bonus;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;

pub struct LightingSystem {}

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightSource>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut map, viewshed, position, light_source) = data;

        if map.outdoors {
            return;
        }

        let black = RGB::named(BLACK);
        for x in map.light.iter_mut() {
            for l in x {
                *l = black;
            }
        }

        for (viewshed, pos, light) in (&viewshed, &position, &light_source).join() {
            let light_point = Point::new(pos.x, pos.y);
            let range_f = light.range as f32;
            for t in viewshed.visible_tiles.iter() {
                if t.x > 0 && t.x < map.width && t.y > 0 && t.y < map.height {
                    let distance = DistanceAlg::Pythagoras.distance2d(light_point, *t);
                    let intensity = (range_f - distance) / range_f;

                    map.light[t.x as usize][t.y as usize] =
                        map.light[t.x as usize][t.y as usize] + (light.color * intensity);
                }
            }
        }
    }
}
