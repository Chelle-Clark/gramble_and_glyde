#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod player;

use agb::{
    display::{
        Priority,
        object::{Graphics, OamManaged, Object, Tag},
        tiled::{RegularBackgroundSize, TileFormat, TiledMap},
    },
    fixnum::{Vector2D, Rect},
    input::{Button, ButtonController},
};
use agb_ext::{
    tiles::{Tilemap, TileSetData},
    math::PosNum,
};
use crate::player::Player;

pub mod tileset {
    include!(concat!(env!("OUT_DIR"), "/tileset.rs"));
}

pub mod single_screen_demo {
    include!(concat!(env!("OUT_DIR"), "/single_screen_demo.rs"));
}

fn move_and_collide(movement: Vector2D<PosNum>, hitbox: Rect<PosNum>, tilemap: &Tilemap) -> Vector2D<PosNum> {
    let tile_collisions = tilemap.get_collision_seams(movement, hitbox);
    let mut actual = movement.clone();
    if let Some(x_collision) = tile_collisions.x_seam {
        let desired_x = {
            let mut value = PosNum::new(x_collision);
            if movement.x > PosNum::new(0) {
                value -= hitbox.size.x;
            }
            value
        };
        actual.x = desired_x - hitbox.position.x;
    }
    if let Some(y_collision) = tile_collisions.y_seam {
        let desired_y = {
            let mut value = PosNum::new(y_collision);
            if movement.y > PosNum::new(0) {
                value -= hitbox.size.y;
            }
            value
        };
        actual.y = desired_y - hitbox.position.y;
    }

    actual
}

fn physics_process(player: &mut Player, tilemap: &Tilemap, input: Option<&ButtonController>) {
    let player_movement = player.propose_movement(input);
    player.move_by(move_and_collide(player_movement, player.col_rect(), tilemap));
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (tiled0, mut vram) = gba.display.video.tiled0();
    let mut background = tiled0.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let object = gba.display.object.get_managed();
    let mut input = agb::input::ButtonController::new();
    let vblank = agb::interrupt::VBlank::get();

    let mut gramble = Player::gramble(&object, (48, 96).into());
    let mut glyde = Player::glyde(&object, (16, 16).into());
    //glyde.hide_sprite();
    let mut playing_gramble = true;

    let tilemap = single_screen_demo::get_level();
    tilemap.draw_background(&mut background, &mut vram);
    background.commit(&mut vram);
    background.set_visible(true);
    object.commit();

    loop {
        physics_process(&mut gramble, &tilemap, if playing_gramble {Some(&input)} else {None});
        physics_process(&mut glyde, &tilemap, if !playing_gramble {Some(&input)} else {None});

        if input.is_just_pressed(Button::L) {
            playing_gramble = !playing_gramble;
        }

        gramble.draw(&object);
        glyde.draw(&object);

        vblank.wait_for_vblank();
        background.commit(&mut vram);
        input.update();
        object.commit();
    }
}
