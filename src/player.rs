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
  anim::{AnimPlayer, AnimOffset},
  camera::Camera,
  collision::{Entity, ControllableEntity, CollisionLayer, Acc, OnGround, Pos, Size, Vel},
  ecs::Entity as EcsEntity,
  anim_enum
};
use crate::world::{World, MutEntityAccessor, HasEntity};

anim_enum!(AnimEnum {
  Idle => 0,
  RunLeadup => 1,
  Run => 2
});

mod gramble_sprites {
  use agb::{
    display::object::{Graphics, Tag},
  };
  use agb_ext::{
    anim::Anim,
    new_anim,
  };
  use agb_ext::anim::AnimId;
  use super::AnimEnum;

  static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/gramble.aseprite");
  static IDLE: &Tag = GRAPHICS.tags().get("Idle");

  pub fn get_next_anim(anim_enum: AnimId) -> Anim {
    match anim_enum.into() {
      AnimEnum::Idle => new_anim!(IDLE, Some(AnimEnum::Idle.into()), (0, 30), (1, 5), (2, 5), (3, 30), (2, 5), (1, 5)),
      AnimEnum::RunLeadup => new_anim!(IDLE, Some(AnimEnum::Run.into()), (0, 60)),
      AnimEnum::Run => new_anim!(IDLE, Some(AnimEnum::Run.into()), (0, 60)),
    }
  }

  pub mod pipe {
    use super::*;
    static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/gramble_pipe.aseprite");
    static IDLE: &Tag = GRAPHICS.tags().get("Idle");

    pub fn get_next_anim(anim_enum: AnimId) -> Anim {
      new_anim!(IDLE, Some(AnimEnum::Idle.into()), (0, 60))
    }
  }
}

pub enum PlayerType {
  Gramble,
  Glyde,
}

pub struct CurrentPlayer;

pub struct GramblePipe<'obj> {
  anim: AnimPlayer<'obj>,
  position: Vector2D<PosNum>,
}

const GRAVITY: PosNum = const_num_i32(0,1);
const ACCELERATION: PosNum = const_num_i32(0,2);
const GRAMBLE_MAX_VEL: PosNum = const_num_i32(3,0);
const GLYDE_MAX_VEL: PosNum = const_num_i32(2,5);
const GRAMBLE_MAX_HEIGHT: PosNum = const_num_i32(35,0);
const GLYDE_MAX_HEIGHT: PosNum = const_num_i32(19,5);

fn jump_impulse(max_height: PosNum) -> PosNum {
  (PosNum::new(2) * GRAVITY * max_height).sqrt()
}

pub fn gramble<'obj>(world: &mut World<'obj>, object: &'obj OamManaged<'obj>, position: Vector2D<PosNum>) -> EcsEntity {
  world.build_entity()
    .set(Pos(position))
    .set(Vel(Vector2D::new(ZERO, ZERO)))
    .set(Acc(Vector2D::new(ZERO, GRAVITY)))
    .set(Size((14, 28).into()))
    .set(OnGround(false))
    .set(CollisionLayer::Normal)
    .set(PlayerType::Gramble)
    .set(AnimPlayer::new(object, gramble_sprites::get_next_anim, AnimEnum::Idle.into()))
    .set(AnimOffset((1, 4).into()))
    .entity()
}

pub fn glyde<'obj>(world: &mut World<'obj>, object: &'obj OamManaged, position: Vector2D<PosNum>) -> EcsEntity {
  world.build_entity()
    .set(Pos(position))
    .set(Vel(Vector2D::new(ZERO, ZERO)))
    .set(Acc(Vector2D::new(ZERO, GRAVITY)))
    .set(Size((24, 28).into()))
    .set(OnGround(false))
    .set(CollisionLayer::Normal)
    .set(PlayerType::Glyde)
    .set(AnimPlayer::new(object, gramble_sprites::get_next_anim, AnimEnum::Idle.into()))
    .set(AnimOffset((4, 4).into()))
    .entity()
}

pub mod system {
  use super::*;

  pub fn run_anim<'o>(_: &PlayerType, anim: &mut AnimPlayer<'o>, current_player: Option<&CurrentPlayer>, object: &'o OamManaged, input: &ButtonController) {
    let tri = input.x_tri();
    if current_player.is_some() && tri != Tri::Zero {
      match tri {
        Tri::Negative => {
          anim.sprite_mut().set_hflip(true);
        },
        Tri::Positive => {
          anim.sprite_mut().set_hflip(false);
        },
        _ => {}
      };
      if anim.cur_anim() != AnimEnum::Run.into() {
        anim.set_anim(AnimEnum::RunLeadup.into(), object);
      }
    } else {
      anim.set_anim(AnimEnum::Idle.into(), object);
    }
  }

  pub fn player_movement(player_type: &PlayerType, current_player: Option<&CurrentPlayer>, vel: &mut Vel, on_ground: &OnGround, input: &ButtonController) {
    let max_velocity = match player_type {
      PlayerType::Gramble => GRAMBLE_MAX_VEL,
      PlayerType::Glyde => GLYDE_MAX_VEL,
    };

    let tri = {
      if current_player.is_some() {
        input.x_tri()
      } else {
        Tri::Zero
      }
    };
    let desired_x_vel = PosNum::new(tri as i32) * max_velocity;
    vel.0 = {
      let mut vel = vel.0.clone();
      if vel.x > desired_x_vel {
        vel.x -= ACCELERATION;
        vel.x = vel.x.clamp(desired_x_vel, max_velocity);
      } else {
        vel.x += ACCELERATION;
        vel.x = vel.x.clamp(-max_velocity, desired_x_vel);
      }
      if vel.x.abs() < const_num_i32(0,5) {
        match tri {
          Tri::Zero => vel.x = 0.into(),
          Tri::Positive => vel.x = const_num_i32(0,5),
          Tri::Negative => vel.x = -const_num_i32(0,5),
        }
      }

      if current_player.is_some() {
        if input.is_just_pressed(Button::B) && on_ground.0 {
          let jump_impulse = match player_type {
            PlayerType::Gramble => jump_impulse(GRAMBLE_MAX_HEIGHT),
            PlayerType::Glyde => jump_impulse(GLYDE_MAX_HEIGHT),
          };
          vel.y = -jump_impulse;
        } else if input.is_released(Button::B) {
          vel.y += GRAVITY * const_num_i32(0,75);
        }
      }

      let tile_size = const_num_i32(16,0);
      vel.x = vel.x.clamp(-tile_size, tile_size);
      vel.y = vel.y.clamp(-tile_size, tile_size);
      vel
    }
  }

  pub fn center_camera(_: &CurrentPlayer, pos: &Pos, size: &Size, camera: &mut Camera) {
    camera.smoothed_center_on(pos.0 + (size.0 / const_num_i32(2,0)));
  }
}


impl<'obj> GramblePipe<'obj> {
  pub fn new(object: &'obj OamManaged, position: Vector2D<PosNum>) -> Self {
    Self {
      anim: AnimPlayer::new(object, gramble_sprites::pipe::get_next_anim, AnimEnum::Idle.into()),
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