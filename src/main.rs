#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

mod player;
mod object;
mod world;

use alloc::vec::Vec;
use agb::{
  display::{
    Priority,
    tiled::{RegularBackgroundSize, TileFormat, TiledMap, InfiniteScrolledMap, PartialUpdateStatus, VRamManager},
    blend::{Blend, Layer as BlendLayerPriority, BlendMode},
  },
  fixnum::{Vector2D, Rect, Num},
  input::{Button, ButtonController},
};
use agb::fixnum::num;
use agb::sound::mixer::{Frequency, SoundChannel};
use agb_ext::{
  tiles::Tilemap,
  math::PosNum,
  camera::Camera,
  collision::{ControllableEntity, Entity, Pos, Vel, Acc},
  ecs::{MutEntityAccessor, HasEntity}
};
use agb_ext::blend::ManagedBlend;
use crate::player::{gramble};
use crate::world::{World};

pub mod tileset {
  include!(concat!(env!("OUT_DIR"), "/tileset.rs"));
}

pub mod grambles_room {
  include!(concat!(env!("OUT_DIR"), "/grambles_room.rs"));
}

pub mod slope_test {
  include!(concat!(env!("OUT_DIR"), "/slope_test.rs"));
}

pub mod sounds {
  use agb::fixnum::Num;
  use agb::include_wav;
  use agb_ext::{
    sound::Music,
    math::const_num_u32,
  };

  static TITLE_DATA: &[u8] = include_wav!("sound/Title.wav");
  pub static TITLE: Music = Music::new(TITLE_DATA, const_num_u32(7, 125));
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
  let (tiled0, mut vram) = gba.display.video.tiled0();
  let primary = tiled0.background(
    Priority::P2,
    RegularBackgroundSize::Background32x32,
    TileFormat::FourBpp,
  );
  let foreground = tiled0.background(
    Priority::P1,
    RegularBackgroundSize::Background32x32,
    TileFormat::FourBpp,
  );

  let tilemap: &Tilemap = &grambles_room::TILEMAP;
  tilemap.load_tileset_palette(&mut vram);

  let mut input = ButtonController::new();
  let object = gba.display.object.get_managed();
  let mut camera = Camera::new();
  let mut collide_tilemap = tilemap.clone().into();

  let mut world = World::new();
  tilemap.set_camera_limits(&mut camera);

  let vblank = agb::interrupt::VBlank::get();

  let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
  mixer.enable();
  sounds::TITLE.play(&mut mixer);

  let mut blend = gba.display.blend.get();
  blend.set_blend_mode(BlendMode::Normal);
  blend.set_background_enable(BlendLayerPriority::Bottom, primary.background(), true);
  blend.set_background_enable(BlendLayerPriority::Top, foreground.background(), true);
  blend.set_object_enable(BlendLayerPriority::Bottom, true);
  let mut blend = ManagedBlend::new(blend);

  let mut gramble = gramble(&mut world, &object, (48, 96).into());
  //let mut glyde = Player::glyde(&object, (80, 80).into());
  //let mut gramble_pipe = GramblePipe::new(&object, (19 * 16, 32).into());

  grambles_room::load_objects(&mut world);

  let mut primary = InfiniteScrolledMap::new(primary, tilemap.primary_tile_fn());
  primary.init(&mut vram, (0, 0).into(), &mut || {});
  primary.commit(&mut vram);
  primary.set_visible(true);
  let mut foreground = InfiniteScrolledMap::new(foreground, tilemap.foreground_tile_fn());
  foreground.init(&mut vram, (0, 0).into(), &mut || {});
  foreground.commit(&mut vram);
  foreground.set_visible(true);
  object.commit();

  loop {
    primary.set_pos(&mut vram, camera.position().trunc());
    foreground.set_pos(&mut vram, camera.position().trunc());
    world.frame(&input, &object, &mut camera, &collide_tilemap, &mut blend);

    vblank.wait_for_vblank();
    primary.commit(&mut vram);
    foreground.commit(&mut vram);
    blend.commit();
    mixer.frame();
    object.commit();
    input.update();
  }

  primary.clear(&mut vram);
  foreground.clear(&mut vram);
  loop {}
}
