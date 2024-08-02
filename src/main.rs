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
};
use crate::player::{Player, PosNum};

agb::include_background_gfx!(tile_test, "333333", background => deduplicate "gfx/tile_test.png");

fn move_and_collide(movement: Vector2D<PosNum>, hitbox: Rect<PosNum>) -> Vector2D<PosNum> {
    let bottom_y = hitbox.position.y + hitbox.size.y + movement.y;
    let floor = 120.into();
    let mut actual = movement.clone();
    if bottom_y > floor {
        actual.y = movement.y - (bottom_y - floor);
    }

    actual
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (tiled0, mut vram) = gba.display.video.tiled0();
    vram.set_background_palettes(tile_test::PALETTES);
    let mut background = tiled0.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let object = gba.display.object.get_managed();
    let mut input = agb::input::ButtonController::new();
    let vblank = agb::interrupt::VBlank::get();

    let mut glyde = Player::new(&object, (16, 64).into());

    let tileset = &tile_test::background.tiles;
    background.set_tile(
        &mut vram,
        (0_u16, 13_u16),
        tileset,
        tile_test::background.tile_settings[0],
    );
    background.set_tile(
        &mut vram,
        (1_u16, 13_u16),
        tileset,
        tile_test::background.tile_settings[1],
    );
    background.set_tile(
        &mut vram,
        (0_u16, 14_u16),
        tileset,
        tile_test::background.tile_settings[4],
    );
    background.set_tile(
        &mut vram,
        (1_u16, 14_u16),
        tileset,
        tile_test::background.tile_settings[5],
    );
    background.commit(&mut vram);
    background.set_visible(true);
    object.commit();

    loop {
        let player_movement = glyde.propose_movement(&input);
        glyde.move_by(move_and_collide(player_movement, glyde.col_rect()));

        vblank.wait_for_vblank();
        input.update();
        object.commit();
    }
}
