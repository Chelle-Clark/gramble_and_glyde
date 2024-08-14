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
  collision::{Entity, ControllableEntity, CollisionLayer},
};

#[derive(Copy, Clone, PartialEq)]
enum AnimEnum {
  Idle,
  RunLeadup,
  Run,
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
      AnimEnum::Idle => new_anim!(IDLE, Some(AnimEnum::Idle), (0, 30), (1, 5), (2, 5), (3, 30), (2, 5), (1, 5)),
      AnimEnum::RunLeadup => new_anim!(IDLE, Some(AnimEnum::Run), (0, 60)),
      AnimEnum::Run => new_anim!(IDLE, Some(AnimEnum::Run), (0, 60)),
    }
  }

  pub mod pipe {
    use super::*;
    static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/gramble_pipe.aseprite");
    static IDLE: &Tag = GRAPHICS.tags().get("Idle");

    pub fn get_next_anim(anim_enum: AnimEnum) -> Anim<AnimEnum> {
      new_anim!(IDLE, Some(AnimEnum::Idle), (0, 60))
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

pub struct GramblePipe<'obj> {
  anim: AnimPlayer<'obj, AnimEnum>,
  position: Vector2D<PosNum>,
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
      AnimPlayer::new(object, gramble_sprites::get_next_anim, AnimEnum::Idle),
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

  pub fn draw(&mut self, camera: &Camera, object: &'obj OamManaged, input: Option<&ButtonController>) {
    if input.map(|input| input.x_tri() != Tri::Zero) == Some(true) {
      if self.anim.cur_anim() != AnimEnum::Run {
        self.anim.set_anim(AnimEnum::RunLeadup, object);
      }
    } else {
      self.anim.set_anim(AnimEnum::Idle, object);
    }

    let sprite_position = (self.position - camera.position()).trunc();
    self.sprite().set_position(sprite_position);
    self.anim.draw(object);
  }

  fn sprite(&mut self) -> &mut Object<'obj> {
    self.anim.sprite_mut()
  }
}

impl<'obj> Entity for Player<'obj> {
  fn move_by(&mut self, offset: Vector2D<PosNum>, snap_to_ground: bool) {
    self.on_ground = self.velocity.y > ZERO && offset.y == ZERO || snap_to_ground;
    if snap_to_ground {
      self.velocity = Vector2D::new(offset.x, ZERO);
    } else {
      self.velocity = offset;
    }
    self.set_position(self.position + offset);
  }

  fn set_position(&mut self, position: Vector2D<PosNum>) {
    self.position = position;
  }

  fn position(&self) -> Vector2D<PosNum> {
    self.position
  }

  fn col_rect(&self) -> Rect<PosNum> {
    Rect::new(self.position + self.col_rect.position, self.col_rect.size)
  }

  fn col_layer(&self) -> CollisionLayer {
    CollisionLayer::Normal
  }
}

impl<'obj> ControllableEntity for Player<'obj> {
  fn propose_movement(&mut self, input: Option<&ButtonController>) -> Vector2D<PosNum> {
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
      Tri::Negative => {
        self.sprite().set_hflip(true);
      },
      Tri::Positive => {
        self.sprite().set_hflip(false);
      },
      _ => {}
    };

    self.velocity
  }
}


impl<'obj> GramblePipe<'obj> {
  pub fn new(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Self {
    Self {
      anim: AnimPlayer::new(object, gramble_sprites::pipe::get_next_anim, AnimEnum::Idle),
      position
    }
  }

  pub fn draw(&mut self, camera: &Camera, object: &'obj OamManaged) {
    let sprite_position = (self.position - camera.position()).trunc();
    self.sprite().set_position(sprite_position);
    self.anim.draw(object);
  }

  fn sprite(&mut self) -> &mut Object<'obj> {
    self.anim.sprite_mut()
  }

  pub fn hide_sprite(&mut self) {
    self.sprite().hide();
  }
}

const PIPE_MOVE_SPEED: PosNum = const_num_i32(3,5);

impl<'obj> Entity for GramblePipe<'obj> {
  fn move_by(&mut self, offset: Vector2D<PosNum>, _snap_to_ground: bool) {
    self.set_position(self.position + offset);
  }

  fn set_position(&mut self, position: Vector2D<PosNum>) {
    self.position = position;
  }

  fn position(&self) -> Vector2D<PosNum> {
    self.position
  }

  fn col_rect(&self) -> Rect<PosNum> {
    Rect::new(self.position, (16, 16).into())
  }

  fn col_layer(&self) -> CollisionLayer {
    CollisionLayer::Pipe
  }
}

impl<'obj> ControllableEntity for GramblePipe<'obj> {
  fn propose_movement(&mut self, input: Option<&ButtonController>) -> Vector2D<PosNum> {
    if let Some(input) = input {
      let x_tri = input.x_tri();
      let y_tri = input.y_tri();

      Vector2D::new(PosNum::new(x_tri as i32) * PIPE_MOVE_SPEED, PosNum::new(y_tri as i32) * PIPE_MOVE_SPEED)
    } else {
      Vector2D::new(ZERO, ZERO)
    }
  }
}