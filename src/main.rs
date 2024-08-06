#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod player;

use agb::{
    display::{
        Priority,
        tiled::{RegularBackgroundSize, TileFormat, TiledMap},
        blend::{Blend, Layer as BlendLayerPriority, BlendMode},
    },
    fixnum::{Vector2D, Rect, Num},
    input::{Button, ButtonController},
};
use agb_ext::{
    tiles::Tilemap,
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

type OpacityNum = Num<u8, 4>;
mod opacity_num {
    use crate::OpacityNum;

    pub const ZERO: OpacityNum = OpacityNum::from_raw(0);
    pub const ONE: OpacityNum = OpacityNum::from_raw(1 << 4);
    pub const MIN_INC: OpacityNum = OpacityNum::from_raw(1);
}

fn apply_opacity(opacity: OpacityNum, blend: &mut Blend) {
    blend.set_blend_weight(BlendLayerPriority::Bottom, opacity_num::ONE - opacity);
    blend.set_blend_weight(BlendLayerPriority::Top, opacity);
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (tiled0, mut vram) = gba.display.video.tiled0();
    let mut primary = tiled0.background(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut foreground = tiled0.background(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let object = gba.display.object.get_managed();
    let mut input = agb::input::ButtonController::new();
    let vblank = agb::interrupt::VBlank::get();

    let mut blend = gba.display.blend.get();
    blend.set_blend_mode(BlendMode::Normal);
    blend.set_background_enable(BlendLayerPriority::Bottom, primary.background(), true);
    blend.set_background_enable(BlendLayerPriority::Top, foreground.background(), true);
    let mut opacity = opacity_num::ONE;
    apply_opacity(opacity, &mut blend);

    let mut gramble = Player::gramble(&object, (48, 96).into());
    let mut glyde = Player::glyde(&object, (16, 16).into());
    //glyde.hide_sprite();
    let mut playing_gramble = true;

    let tilemap = single_screen_demo::get_level();
    tilemap.draw_primary(&mut primary, &mut vram);
    primary.commit(&mut vram);
    primary.set_visible(true);
    tilemap.draw_foreground(&mut foreground, &mut vram);
    foreground.commit(&mut vram);
    foreground.set_visible(true);
    object.commit();

    loop {
        physics_process(&mut gramble, &tilemap, if playing_gramble {Some(&input)} else {None});
        physics_process(&mut glyde, &tilemap, if !playing_gramble {Some(&input)} else {None});

        if input.is_just_pressed(Button::L) {
            playing_gramble = !playing_gramble;
        }

        if input.is_pressed(Button::R) {
            if opacity > opacity_num::ZERO {
                opacity -= opacity_num::MIN_INC;
            }
        } else {
            opacity += opacity_num::MIN_INC;
        }
        opacity = opacity.clamp(opacity_num::ZERO, opacity_num::ONE);
        apply_opacity(opacity, &mut blend);

        gramble.draw(&object);
        glyde.draw(&object);

        vblank.wait_for_vblank();
        primary.commit(&mut vram);
        foreground.commit(&mut vram);
        blend.commit();
        input.update();
        object.commit();
    }
}
