#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        Priority,
        object::{Graphics, OamManaged, Object, Tag},
        tiled::{RegularBackgroundSize, TileFormat},
    },
};
use agb::display::tiled::TiledMap;

agb::include_background_gfx!(tile_test, "333333", background => deduplicate "gfx/tile_test.png");
static GLYDE: &Graphics = agb::include_aseprite!("gfx/glyde.aseprite");
static GLYDE_IDLE: &Tag = GLYDE.tags().get("Idle");

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
    let mut glyde = object.object_sprite(GLYDE_IDLE.sprite(0));
    glyde.set_position((16, 64)).show();

    let tileset = &tile_test::background.tiles;
    background.set_tile(
        &mut vram,
        (0_u16, 8_u16),
        tileset,
        tile_test::background.tile_settings[0],
    );
    background.set_tile(
        &mut vram,
        (1_u16, 8_u16),
        tileset,
        tile_test::background.tile_settings[1],
    );
    background.set_tile(
        &mut vram,
        (0_u16, 9_u16),
        tileset,
        tile_test::background.tile_settings[4],
    );
    background.set_tile(
        &mut vram,
        (1_u16, 9_u16),
        tileset,
        tile_test::background.tile_settings[5],
    );
    background.commit(&mut vram);
    background.set_visible(true);
    object.commit();

    loop {

    }
}
