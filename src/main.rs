#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    include_aseprite,
    include_wav,
    fixnum::{Vector2D, Rect, Num, num},
    display::object::{Graphics, Object, OamManaged, Tag},
    sound::mixer::{Mixer, SoundChannel, Frequency},
    input::{Button, Tri},
    rng,
};

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
    const WIDTH: i32 = 16;
    const HEIGHT: i32 = 48;

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

    fn position(&self) -> Vector2D<i32> {
        self.start.position()
    }

    fn ball_colliding(&self, ball_rect: Rect<i32>) -> bool {
        let rect = Rect {
            position: self.position(),
            size: Vector2D::new(Self::WIDTH, Self::HEIGHT),
        };
        rect.touches(ball_rect)
    }
}

struct Ball<'obj> {
    sprite: Object<'obj>,
    position: Vector2D<Num<i32, 3>>,
    velocity: Vector2D<Num<i32, 3>>,
}

impl<'obj> Ball<'obj> {
    const MAX_X: i32 = agb::display::WIDTH - 16;
    const MAX_Y: i32 = agb::display::HEIGHT - 16;

    fn new(object: &'obj OamManaged<'_>, start_x: i32, start_y: i32, start_vel: Vector2D<Num<i32, 3>>) -> Ball<'obj> {
        let mut sprite = object.object_sprite(BALL.sprite(0));
        sprite.show();

        let mut ball = Ball {
            sprite,
            position: Vector2D::new(num!(0.0), num!(0.0)),
            velocity: start_vel,
        };
        ball.set_position(Vector2D::new(Num::from(start_x), Num::from(start_y)));

        ball
    }

    fn frame(&mut self) {
        self.set_position(self.position() + self.velocity);

        let new_pos = self.position().floor();
        if new_pos.x == 0 || new_pos.x == Self::MAX_X {
            self.velocity.x = -self.velocity.x;
        }
        if new_pos.y == 0 || new_pos.y == Self::MAX_Y {
            self.velocity.y = -self.velocity.y;
        }
    }

    fn set_position(&mut self, position: Vector2D<Num<i32, 3>>) {
        let position = Vector2D::new(
            position.x.clamp(num!(0.0), Num::from(Self::MAX_X)),
            position.y.clamp(num!(0.0), Num::from(Self::MAX_Y)));
        self.position = position;
        self.sprite.set_position(position.floor());
    }

    fn position(&self) -> Vector2D<Num<i32, 3>> {
        self.position
    }

    fn rect(&self) -> Rect<i32> {
        Rect::new(self.position().floor(), Vector2D::new(16, 16))
    }

    fn bounce_l(&mut self, mixer: &mut Mixer<'_>) {
        self.velocity.x = (self.velocity.x ).abs()+ num!(0.2);
        self.velocity.y += num!(0.2) * {if self.velocity.y > 0.into() {1} else {-1}};

        let sproing = SoundChannel::new(SPROING);
        let _ = mixer.play_sound(sproing);
    }

    fn bounce_r(&mut self, mixer: &mut Mixer<'_>) {
        self.velocity.x = -(self.velocity.x).abs() - num!(0.2);
        self.velocity.y += num!(0.2) * {if self.velocity.y > 0.into() {1} else {-1}};

        let sproing = SoundChannel::new(SPROING);
        let _ = mixer.play_sound(sproing);
    }
}


#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let object = gba.display.object.get_managed();
    let mut input = agb::input::ButtonController::new();
    let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
    mixer.enable();
    let vblank = agb::interrupt::VBlank::get();

    let mut ball = Ball::new(&object, 50, 50, Vector2D::new(1.into(), 1.into()));
    let mut paddle_l = Paddle::new(&object, 8, 8);
    let mut paddle_r = Paddle::new(&object, agb::display::WIDTH - 16 - 8, 8);

    let mut channel = SoundChannel::new(FIFTEEN_STEP);
    channel.stereo();
    let _ = mixer.play_sound(channel);

    loop {
        ball.frame();

        let l_movement = input.y_tri();
        let mut paddle_l_pos = paddle_l.position();
        paddle_l_pos.y += l_movement as i32 * 3;
        paddle_l.set_position(paddle_l_pos.x, paddle_l_pos.y);

        let r_movement = {
            if input.is_pressed(Button::A) {
                Tri::Negative
            } else if input.is_pressed(Button::B) {
                Tri::Positive
            } else {
                Tri::Zero
            }
        };
        let mut paddle_r_pos = paddle_r.position();
        paddle_r_pos.y += r_movement as i32 * 3;
        paddle_r.set_position(paddle_r_pos.x, paddle_r_pos.y);

        if paddle_l.ball_colliding(ball.rect()) {
            ball.bounce_l(&mut mixer);
        }
        if paddle_r.ball_colliding(ball.rect()) {
            ball.bounce_r(&mut mixer);
        }

        mixer.frame();
        vblank.wait_for_vblank();
        object.commit();
        input.update();
    }
}
