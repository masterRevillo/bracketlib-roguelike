use bracket_lib::color::RGB;
use bracket_lib::prelude::{console, FontCharType, Point};
use specs::{Join, ReadStorage, System, WriteStorage};
use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB
}

#[derive(Component)]
pub struct LeftMover {}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct Name {
    pub name: String
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool
}
#[derive(Component)]
pub struct Player {}

pub struct LeftWalker{}
impl<'a> System<'a> for LeftWalker {
    type SystemData = (
        ReadStorage<'a, LeftMover>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for(_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79}
        }
    }
}

#[derive(Component, Debug)]
pub struct BlocksTile {}


#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}
