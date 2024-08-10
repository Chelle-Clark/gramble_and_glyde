#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod player;

use agb::{
    display::{
        Priority,
        tiled::{RegularBackgroundSize, TileFormat, TiledMap, InfiniteScrolledMap, PartialUpdateStatus, VRamManager},
        blend::{Blend, Layer as BlendLayerPriority, BlendMode},
    },
    fixnum::{Vector2D, Rect, Num},
    input::{Button, ButtonController},
};
use agb::sound::mixer::{Frequency, SoundChannel};
use agb_ext::{
    tiles::Tilemap,
    math::PosNum,
    camera::Camera,
};
use crate::player::{Player, GramblePipe, Controllable};

pub mod tileset {
    include!(concat!(env!("OUT_DIR"), "/tileset.rs"));
}

pub mod single_screen_demo {
    include!(concat!(env!("OUT_DIR"), "/grambles_room.rs"));
}

pub mod sounds {
    use agb::fixnum::Num;
    use agb::include_wav;
    use agb_ext::{
        sound::Music,
        math::const_num_u32,
    };

    static TITLE_DATA: &[u8] = include_wav!("sound/Title.wav");
    pub static TITLE: Music = Music::new(TITLE_DATA, const_num_u32(7,125));
}

fn move_and_collide(movement: Vector2D<PosNum>, hitbox: Rect<PosNum>, tilemap: &Tilemap, in_pipe: bool) -> Vector2D<PosNum> {
    let tile_collisions = tilemap.get_collision_seams(movement, hitbox, in_pipe);
    let (x_collision, y_collision) = tile_collisions.slide_corners();
    let mut actual = movement.clone();
    if let Some(x_collision) = x_collision {
        let desired_x = {
            let mut value = PosNum::new(x_collision);
            if movement.x > PosNum::new(0) {
                value -= hitbox.size.x;
            }
            value
        };
        actual.x = desired_x - hitbox.position.x;
    }
    if let Some(y_collision) = y_collision {
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

fn physics_process(player: &mut dyn Controllable, tilemap: &Tilemap, input: Option<&ButtonController>, in_pipe: bool) {
    let player_movement = player.propose_movement(input);
    player.move_by(move_and_collide(player_movement, player.col_rect(), tilemap, in_pipe));
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

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();
    sounds::TITLE.play(&mut mixer);

    let mut blend = gba.display.blend.get();
    blend.set_blend_mode(BlendMode::Normal);
    blend.set_background_enable(BlendLayerPriority::Bottom, primary.background(), true);
    blend.set_background_enable(BlendLayerPriority::Top, foreground.background(), true);
    blend.set_object_enable(BlendLayerPriority::Bottom, true);
    let mut opacity = opacity_num::ONE;
    apply_opacity(opacity, &mut blend);

    let mut gramble = Player::gramble(&object, (48, 96).into());
    let mut glyde = Player::glyde(&object, (16, 16).into());
    let mut gramble_pipe = GramblePipe::new(&object, (19 * 16, 32).into());
    glyde.hide_sprite();
    let mut playing_gramble = true;

    let mut camera = Camera::new();

    let tilemap: &Tilemap = &single_screen_demo::TILEMAP;
    tilemap.load_tileset_palette(&mut vram);
    tilemap.set_camera_limits(&mut camera);

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
        let gramble_input = if playing_gramble {Some(&input)} else {None};
        let glyde_input = if !playing_gramble {Some(&input)} else {None};
        physics_process(&mut gramble, &tilemap, gramble_input, false);
        physics_process(&mut glyde, &tilemap, glyde_input, false);
        physics_process(&mut gramble_pipe, &tilemap, Some(&input), true);

        if input.is_just_pressed(Button::L) {
            playing_gramble = !playing_gramble;
        }

        let player = if playing_gramble { &gramble } else { &glyde };
        camera.smoothed_center_on(player.position());

        if input.is_pressed(Button::R) {
            if opacity > opacity_num::ZERO {
                opacity -= opacity_num::MIN_INC;
            }
        } else {
            opacity += opacity_num::MIN_INC;
        }
        opacity = opacity.clamp(opacity_num::ZERO, opacity_num::ONE);
        apply_opacity(opacity, &mut blend);

        if input.is_just_pressed(Button::START) {
            break;
        }

        gramble.draw(&camera, &object, gramble_input);
        gramble_pipe.draw(&camera, &object);
        glyde.draw(&camera, &object, glyde_input);
        primary.set_pos(&mut vram, camera.position().trunc());
        foreground.set_pos(&mut vram, camera.position().trunc());

        vblank.wait_for_vblank();
        primary.commit(&mut vram);
        foreground.commit(&mut vram);
        blend.commit();
        input.update();
        mixer.frame();
        object.commit();
    }

    primary.clear(&mut vram);
    foreground.clear(&mut vram);
    loop {}
}
