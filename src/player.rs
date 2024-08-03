use agb::{
  display::object::{Object, OamManaged, Graphics, Tag},
  fixnum::{Vector2D, Rect, Num, num},
  input::{ButtonController, Button, Tri},
};

pub type PosNum = Num<i32, 8>;

const ZERO: PosNum = PosNum::from_raw(0);

static GRAMBLE: &Graphics = agb::include_aseprite!("gfx/gramble.aseprite");
static GRAMBLE_IDLE: &Tag = GRAMBLE.tags().get("Idle");

static GLYDE: &Graphics = agb::include_aseprite!("gfx/glyde.aseprite");
static GLYDE_IDLE: &Tag = GLYDE.tags().get("Idle");

enum PlayerType {
  Gramble,
  Glyde,
}

pub struct Player<'obj> {
  sprite: Object<'obj>,
  position: Vector2D<PosNum>,
  velocity: Vector2D<PosNum>,

  col_rect: Rect<PosNum>,
  player_type: PlayerType,
  on_ground: bool,
}

impl<'obj> Player<'obj> {
  pub fn gramble(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Player<'obj> {
    Self::new(object, position, GRAMBLE_IDLE, Rect::new((1, 4).into(), (14, 28).into()), PlayerType::Gramble)
  }

  pub fn glyde(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Player<'obj> {
    Self::new(object, position, GLYDE_IDLE, Rect::new((4, 4).into(), (24, 28).into()), PlayerType::Glyde)
  }

  fn new(object: &'obj OamManaged, position: Vector2D<PosNum>, tag: &Tag, col_rect: Rect<PosNum>, player_type: PlayerType) -> Player<'obj> {
    let mut sprite = object.object_sprite(tag.sprite(0));
    sprite.show();

    let mut player = Player {
      sprite,
      position: (0, 0).into(),
      velocity: (0, 0).into(),
      col_rect,
      player_type,
      on_ground: false,
    };
    player.set_position(position);

    player
  }

  pub fn propose_movement(&mut self, input: &ButtonController) -> Vector2D<PosNum> {
    let acceleration: PosNum = num!(0.2);
    let max_velocity: PosNum = match self.player_type {
      PlayerType::Gramble => num!(3.0),
      PlayerType::Glyde => num!(2.5),
    };
    let gravity: PosNum = num!(0.1);
    let jump_impulse: PosNum = {
      let max_height: PosNum = match self.player_type {
        PlayerType::Gramble => 35,
        PlayerType::Glyde => 19,
      }.into();
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
      if vel.x.abs() < num!(0.5) {
        match tri {
          Tri::Zero => vel.x = 0.into(),
          Tri::Positive => vel.x = num!(0.5),
          Tri::Negative => vel.x = num!(-0.5),
        }
      }

      vel.y += gravity;
      if input.is_just_pressed(Button::B) && self.on_ground {
        vel.y = -jump_impulse;
      } else if input.is_released(Button::B) {
        vel.y += gravity * num!(0.75);
      }

      let tile_size = PosNum::new(16);
      vel.x = vel.x.clamp(-tile_size, tile_size);
      vel.y = vel.y.clamp(-tile_size, tile_size);
      vel
    };

    match tri {
      Tri::Negative => { self.sprite.set_hflip(true); },
      Tri::Positive => { self.sprite.set_hflip(false); },
      _ => {}
    };

    self.velocity
  }

  pub fn move_by(&mut self, offset: Vector2D<PosNum>) {
    self.on_ground = self.velocity.y > ZERO && offset.y == ZERO;
    self.velocity = offset;
    self.set_position(self.position + offset);
  }

  pub fn set_position(&mut self, position: Vector2D<PosNum>) {
    self.position = position;
    self.sprite.set_position(self.position.trunc());
  }

  pub fn col_rect(&self) -> Rect<PosNum> {
    Rect::new(self.position + self.col_rect.position, self.col_rect.size)
  }
}