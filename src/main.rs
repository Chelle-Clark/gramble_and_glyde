#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    include_aseprite,
    include_wav,
    display::object::{Graphics, Object, OamManaged, Tag},
    sound::mixer::Frequency,
    input::Button,
};
use agb::sound::mixer::SoundChannel;

static GRAPHICS: &Graphics = include_aseprite!("gfx/sprites.aseprite");
static FIFTEEN_STEP: &[u8] = include_wav!("sound/15StepCrust.wav");
static SPROING: &[u8] = include_wav!("sound/sproing.wav");

static PADDLE_END: &Tag = GRAPHICS.tags().get("Paddle End");
static PADDLE_MID: &Tag = GRAPHICS.tags().get("Paddle Mid");
static BALL: &Tag = GRAPHICS.tags().get("Ball");

struct Paddle<'obj> {
    start: Object<'obj>,
    mid: Object<'obj>,
    end: Object<'obj>,
}

impl<'obj> Paddle<'obj> {
    fn new(object: &'obj OamManaged<'_>, start_x: i32, start_y: i32) -> Self {
        let mut paddle_start = object.object_sprite(PADDLE_END.sprite(0));
        let mut paddle_mid = object.object_sprite(PADDLE_MID.sprite(0));
        let mut paddle_end = object.object_sprite(PADDLE_END.sprite(0));
        paddle_start.show();
        paddle_mid.show();
        paddle_end.set_vflip(true).show();

        let mut paddle = Self {
            start: paddle_start,
            mid: paddle_mid,
            end: paddle_end,
        };

        paddle.set_position(start_x, start_y);
        paddle
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.start.set_position((x, y));
        self.mid.set_position((x, y + 16));
        self.end.set_position((x, y + 32));
    }
}


#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let object = gba.display.object.get_managed();
    let mut input = agb::input::ButtonController::new();
    let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
    mixer.enable();
    let vblank = agb::interrupt::VBlank::get();

    let mut ball = object.object_sprite(BALL.sprite(0));
    let mut paddle_l = Paddle::new(&object, 8, 8);
    let mut paddle_r = Paddle::new(&object, agb::display::WIDTH - 16 - 8, 8);

    let mut channel = SoundChannel::new(FIFTEEN_STEP);
    channel.stereo();
    let _ = mixer.play_sound(channel);

    let mut ball_x = 50;
    let mut ball_y = 50;
    let mut x_velocity = 0;
    let mut y_velocity = 0;
    ball.set_x(ball_x as u16).set_y(ball_y as u16).show();

    loop {
        ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
        ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

        x_velocity = input.x_tri() as i32;
        y_velocity = input.y_tri() as i32;
        if input.is_pressed(Button::A) {
            x_velocity *= 2;
            y_velocity *= 2;
            if input.is_just_pressed(Button::A) {
                let mut channel = SoundChannel::new(SPROING);
                let _ = mixer.play_sound(channel);
            }
        }

        ball.set_x(ball_x as u16).set_y(ball_y as u16);

        mixer.frame();
        vblank.wait_for_vblank();
        object.commit();
        input.update();
    }
}
