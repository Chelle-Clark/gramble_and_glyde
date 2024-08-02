use agb::{
  display::object::{Object, OamManaged, Graphics, Tag},
  fixnum::{Vector2D, Num, num},
  input::{ButtonController, Button, Tri},
};

type PosNum = Num<i32, 8>;

static GLYDE: &Graphics = agb::include_aseprite!("gfx/glyde.aseprite");
static GLYDE_IDLE: &Tag = GLYDE.tags().get("Idle");

pub struct Player<'obj> {
  sprite: Object<'obj>,
  position: Vector2D<PosNum>,
  velocity: Vector2D<PosNum>,
}

impl<'obj> Player<'obj> {
  pub fn new(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Player<'obj> {
    let mut sprite = object.object_sprite(GLYDE_IDLE.sprite(0));
    sprite.show();

    let mut player = Player {
      sprite,
      position: (0, 0).into(),
      velocity: (0, 0).into(),
    };
    player.set_position(position);

    player
  }

  pub fn frame(&mut self, input: &ButtonController) {
    let acceleration: PosNum = num!(0.2);
    let max_velocity: PosNum = num!(2.5);
    let gravity: PosNum = num!(0.3);
    let jump_impulse: PosNum = {
      let max_height: PosNum = 19.into();
      (PosNum::new(2) * gravity * max_height).sqrt()
    };

    let tri = input.x_tri();
    let desired_x_vel = PosNum::new(tri as i32) * max_velocity;
    self.velocity = {
      let mut vel = self.velocity.clone();
      if vel.x > desired_x_vel {
        vel.x -= acceleration;
        vel.x = vel.x.clamp(desired_x_vel, max_velocity);
      } else {
        vel.x += acceleration;
        vel.x = vel.x.clamp(-max_velocity, desired_x_vel);
      }
      if vel.x.abs() < 1.into() {
        match tri {
          Tri::Zero => vel.x = 0.into(),
          Tri::Positive => vel.x = 1.into(),
          Tri::Negative => vel.x = (-1).into(),
        }
      }

      vel.y += gravity;
      if input.is_just_pressed(Button::A) {
        vel.y = -jump_impulse;
      }

      vel
    };

    self.move_by(self.velocity);
  }

  pub fn move_by(&mut self, offset: Vector2D<PosNum>) {
    self.set_position(self.position + offset);
  }

  pub fn set_position(&mut self, position: Vector2D<PosNum>) {
    self.position = position;
    self.position.y = self.position.y.clamp(0.into(), (120 - 32).into());
    self.sprite.set_position(self.position.trunc());
  }
}