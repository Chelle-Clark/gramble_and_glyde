use agb::{
  display::{
    object::{Object, OamManaged},
    Priority,
  },
  fixnum::{Vector2D, Rect, num},
  input::{ButtonController, Button, Tri},
};
use agb_ext::{
  math::{PosNum, ZERO, const_num_i32},
  anim::AnimPlayer,
  camera::Camera,
};

#[derive(Copy, Clone)]
enum AnimEnum {
  Idle,
}

mod gramble_sprites {
  use agb::{
    display::object::{Graphics, Tag},
  };
  use agb_ext::{
    anim::Anim,
    new_anim,
  };
  use super::AnimEnum;

  static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/gramble.aseprite");
  static IDLE: &Tag = GRAPHICS.tags().get("Idle");

  pub fn get_next_anim(anim_enum: AnimEnum) -> Anim<AnimEnum> {
    match anim_enum {
      AnimEnum::Idle => new_anim!(IDLE, Some(AnimEnum::Idle), (0, 60)),
    }
  }
}

mod glyde_sprites {
  use agb::{
    display::object::{Graphics, Tag},
  };
  use agb_ext::{
    anim::Anim,
    new_anim,
  };
  use super::AnimEnum;

  static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/glyde.aseprite");
  static IDLE: &Tag = GRAPHICS.tags().get("Idle");

  pub fn get_next_anim(anim_enum: AnimEnum) -> Anim<AnimEnum> {
    match anim_enum {
      AnimEnum::Idle => new_anim!(IDLE, Some(AnimEnum::Idle), (0, 90), (1, 6), (2, 6), (3, 6))
    }
  }
}

enum PlayerType {
  Gramble,
  Glyde,
}

pub struct Player<'obj> {
  anim: AnimPlayer<'obj, AnimEnum>,
  position: Vector2D<PosNum>,
  velocity: Vector2D<PosNum>,

  col_rect: Rect<PosNum>,
  player_type: PlayerType,
  on_ground: bool,
}

impl<'obj> Player<'obj> {
  const ACCELERATION: PosNum = const_num_i32(0,2);
  const GRAVITY: PosNum = const_num_i32(0,1);
  const GRAMBLE_MAX_VEL: PosNum = const_num_i32(3,0);
  const GLYDE_MAX_VEL: PosNum = const_num_i32(2,5);
  const GRAMBLE_MAX_HEIGHT: PosNum = const_num_i32(35,0);
  const GLYDE_MAX_HEIGHT: PosNum = const_num_i32(19,5);

  fn jump_impulse(max_height: PosNum) -> PosNum {
    (PosNum::new(2) * Self::GRAVITY * max_height).sqrt()
  }

  pub fn gramble(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Player<'obj> {
    Self::new(
      AnimPlayer::new(object, gramble_sprites::get_next_anim, AnimEnum::Idle),
      position,
      Rect::new((1, 4).into(), (14, 28).into()),
      PlayerType::Gramble)
  }

  pub fn glyde(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Player<'obj> {
    Self::new(
      AnimPlayer::new(object, glyde_sprites::get_next_anim, AnimEnum::Idle),
      position,
      Rect::new((4, 4).into(), (24, 28).into()),
      PlayerType::Glyde)
  }

  fn new(anim: AnimPlayer<'obj, AnimEnum>, position: Vector2D<PosNum>, col_rect: Rect<PosNum>, player_type: PlayerType) -> Player<'obj> {
    let mut player = Player {
      anim,
      position: (0, 0).into(),
      velocity: (0, 0).into(),
      col_rect,
      player_type,
      on_ground: false,
    };
    player.set_position(position);
    player.anim.sprite_mut().set_priority(Priority::P2);

    player
  }

  pub fn hide_sprite(&mut self) {
    self.sprite().hide();
  }

  pub fn propose_movement(&mut self, input: Option<&ButtonController>) -> Vector2D<PosNum> {
    let (max_velocity, jump_impulse) = match self.player_type {
      PlayerType::Gramble => (Self::GRAMBLE_MAX_VEL, Self::jump_impulse(Self::GRAMBLE_MAX_HEIGHT)),
      PlayerType::Glyde => (Self::GLYDE_MAX_VEL, Self::jump_impulse(Self::GLYDE_MAX_HEIGHT)),
    };

    let tri = {
      if let Some(input) = input {
        input.x_tri()
      } else {
        Tri::Zero
      }
    };
    let desired_x_vel = PosNum::new(tri as i32) * max_velocity;
    self.velocity = {
      let mut vel = self.velocity.clone();
      if vel.x > desired_x_vel {
        vel.x -= Self::ACCELERATION;
        vel.x = vel.x.clamp(desired_x_vel, max_velocity);
      } else {
        vel.x += Self::ACCELERATION;
        vel.x = vel.x.clamp(-max_velocity, desired_x_vel);
      }
      if vel.x.abs() < num!(0.5) {
        match tri {
          Tri::Zero => vel.x = 0.into(),
          Tri::Positive => vel.x = num!(0.5),
          Tri::Negative => vel.x = num!(-0.5),
        }
      }

      vel.y += Self::GRAVITY;
      if let Some(input) = input {
        if input.is_just_pressed(Button::B) && self.on_ground {
          vel.y = -jump_impulse;
        } else if input.is_released(Button::B) {
          vel.y += Self::GRAVITY * num!(0.75);
        }
      }

      let tile_size = PosNum::new(16);
      vel.x = vel.x.clamp(-tile_size, tile_size);
      vel.y = vel.y.clamp(-tile_size, tile_size);
      vel
    };

    match tri {
      Tri::Negative => { self.sprite().set_hflip(true); },
      Tri::Positive => { self.sprite().set_hflip(false); },
      _ => {}
    };

    self.velocity
  }

  pub fn draw(&mut self, camera: &Camera, object: &'obj OamManaged) {
    let sprite_position = (self.position - camera.position()).trunc();
    self.sprite().set_position(sprite_position);
    self.anim.draw(object);
  }

  pub fn move_by(&mut self, offset: Vector2D<PosNum>) {
    self.on_ground = self.velocity.y > ZERO && offset.y == ZERO;
    self.velocity = offset;
    self.set_position(self.position + offset);
  }

  pub fn set_position(&mut self, position: Vector2D<PosNum>) {
    self.position = position;
  }

  pub fn position(&self) -> Vector2D<PosNum> {
    self.position
  }

  pub fn col_rect(&self) -> Rect<PosNum> {
    Rect::new(self.position + self.col_rect.position, self.col_rect.size)
  }

  fn sprite(&mut self) -> &mut Object<'obj> {
    self.anim.sprite_mut()
  }
}