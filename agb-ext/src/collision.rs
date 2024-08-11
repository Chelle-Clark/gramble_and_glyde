use core::convert::Into;
use agb::{fixnum::{Vector2D, Num, Rect}, include_wav, input::ButtonController};
use crate::collision::CollisionLayer::Pipe;
use crate::math::{PosNum, const_num_i32, ZERO, MIN_INC};

#[derive(Clone, Copy, PartialEq)]
pub enum CollideTileType {
  Pass,
  Solid,
  LWall,
  RWall,
  Pipe,
  RSteepSlope,
  RLowSlope1,
  RLowSlope2,
  PipeSolid,
  LSteepSlope,
  LLowSlope1,
  LLowSlope2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CollisionLayer {
  Normal, Pipe
}

#[derive(Clone)]
pub struct Collision {
  pub x_seam: Option<i32>,
  pub y_seam: Option<i32>,
}

pub struct CollideTilemap {
  pub data: &'static [CollideTileType],
  pub width: usize,
  pub height: usize,
}


pub trait Entity {
  fn move_by(&mut self, offset: Vector2D<PosNum>);
  fn set_position(&mut self, position: Vector2D<PosNum>);
  fn position(&self) -> Vector2D<PosNum>;
  fn col_rect(&self) -> Rect<PosNum>;
  fn col_layer(&self) -> CollisionLayer { CollisionLayer::Normal }
}

pub trait ControllableEntity: Entity {
  fn propose_movement(&mut self, input: Option<&ButtonController>) -> Vector2D<PosNum>;

  fn physics_process(&mut self, tilemap: &CollideTilemap, input: Option<&ButtonController>) {
    let movement = self.propose_movement(input);
    self.move_by(move_and_collide(movement, self.col_rect(), tilemap, self.col_layer()));
  }
}


fn move_and_collide(movement: Vector2D<PosNum>, hitbox: Rect<PosNum>, tilemap: &CollideTilemap, layer: CollisionLayer) -> Vector2D<PosNum> {
  let tile_collisions = tilemap.get_collision_seams(movement, hitbox, layer);
  let mut actual = movement.clone();
  if let Some(x_collision) = tile_collisions.x_seam {
    let desired_x = {
      let mut value = PosNum::new(x_collision);
      if movement.x > PosNum::new(0) {
        value -= hitbox.size.x;
      }
      value
    };
    actual.x = desired_x - hitbox.position.x;
  }
  if let Some(y_collision) = tile_collisions.y_seam {
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


impl CollideTilemap {
  const PX_PER_TILE: PosNum = const_num_i32(16,0);

  fn get_collision_seams(&self, movement: Vector2D<PosNum>, hitbox: Rect<PosNum>, layer: CollisionLayer) -> Collision {
    let moving_left = movement.x < ZERO;
    let moving_up = movement.y < ZERO;
    let entered_x = {
      let cur_edge = {
        if !moving_left {
          hitbox.position.x + hitbox.size.x - MIN_INC
        } else {
          hitbox.position.x
        }
      };
      if (cur_edge / Self::PX_PER_TILE).floor() != ((cur_edge + movement.x) / Self::PX_PER_TILE).floor() {
        Some(((cur_edge + movement.x) / Self::PX_PER_TILE).floor())
      } else {
        None
      }
    };
    let entered_y = {
      let cur_edge = {
        if !moving_up {
          hitbox.position.y + hitbox.size.y - MIN_INC
        } else {
          hitbox.position.y
        }
      };
      if (cur_edge / Self::PX_PER_TILE).floor() != ((cur_edge + movement.y) / Self::PX_PER_TILE).floor() {
        Some(((cur_edge + movement.y) / Self::PX_PER_TILE).floor())
      } else {
        None
      }
    };

    let adjusted_hitbox = Rect::new(hitbox.position + movement, hitbox.size);
    let (tile_left_x, tile_right_x) = {
      let left_x = adjusted_hitbox.position.x;
      let right_x = left_x + hitbox.size.x - MIN_INC;
      ((left_x / Self::PX_PER_TILE).floor(), (right_x / Self::PX_PER_TILE).floor())
    };
    let (tile_up_y, tile_down_y) = {
      let up_y = adjusted_hitbox.position.y;
      let down_y = up_y + hitbox.size.y - MIN_INC;
      ((up_y / Self::PX_PER_TILE).floor(), (down_y / Self::PX_PER_TILE).floor())
    };
    let mut corner_y_seam = None;

    let mut x_seam = None;
    let mut y_seam = None;
    for xi in tile_left_x..=tile_right_x {
      for yi in tile_up_y..=tile_down_y {
        if xi > 0 && xi < self.width as i32 && yi > 0 && yi < self.height as i32 {
          let tile_idx = xi as usize + yi as usize * self.width;
          let tile: CollideTileType = self.data[tile_idx];
          if tile.is_tile_colliding((xi, yi).into(), adjusted_hitbox, layer) {
            match (entered_x == Some(xi), entered_y == Some(yi)) {
              (false, false) => {
                if tile.is_nonstandard_hitbox() {
                  let specialized_col = tile.specialized_collide((xi, yi).into(), adjusted_hitbox, moving_left, moving_up);
                  if let Some(new_x_seam) = specialized_col.x_seam {
                    x_seam = Some(Self::stricter_seam(x_seam, new_x_seam, moving_left));
                  }
                  if let Some(new_y_seam) = specialized_col.y_seam {
                    y_seam = Some(Self::stricter_seam(y_seam, new_y_seam, moving_up));
                  }
                }
              },
              (true, false) => {
                let new_seam = if moving_left { (xi + 1)  * 16 } else { xi * 16 };
                x_seam = Some(Self::stricter_seam(x_seam, new_seam, moving_left));
              },
              (false, true) => {
                let new_seam = if moving_up { (yi + 1)  * 16 } else { yi * 16 };
                y_seam = Some(Self::stricter_seam(y_seam, new_seam, moving_up));
              },
              (true, true) => {
                corner_y_seam = Some(if moving_up { (yi + 1)  * 16 } else { yi * 16 });
              },
            }
          }
        }
      }
    }

    if corner_y_seam.is_some() && x_seam.is_none() && y_seam.is_none() {
      y_seam = corner_y_seam;
    }

    Collision {x_seam, y_seam}
  }

  fn stricter_seam(cur: Option<i32>, new: i32, moving_negative: bool) -> i32 {
    if let Some(cur) = cur {
      if moving_negative {
        cur.max(new)
      } else {
        cur.min(new)
      }
    } else {
      new
    }
  }
}

impl CollideTileType {
  pub fn is_nonstandard_hitbox(self) -> bool {
    [
      Self::LWall, Self::RWall,
      Self::LSteepSlope, Self::LLowSlope1, Self::LLowSlope2,
      Self::RSteepSlope, Self::RLowSlope1, Self::RLowSlope2
    ].contains(&self)
  }

  pub fn is_tile_colliding(self, pos: Vector2D<i32>, adjusted_hitbox: Rect<PosNum>, layer: CollisionLayer) -> bool {
    match self {
      Self::Pass => layer == CollisionLayer::Pipe,
      Self::Solid => true,
      Self::Pipe => false,
      Self::PipeSolid => layer == CollisionLayer::Normal,

      Self::LWall => {
        let pos = Vector2D::new(PosNum::new(pos.x * 16), PosNum::new(pos.y * 16));
        let self_rect = Rect::new(pos, (2, 16).into());
        adjusted_hitbox.touches(self_rect)
      },
      Self::RWall => {
        let pos = Vector2D::new(PosNum::new(pos.x * 16 + 14), PosNum::new(pos.y * 16));
        let self_rect = Rect::new(pos, (2, 16).into());
        adjusted_hitbox.touches(self_rect)
      },

      Self::LSteepSlope => {
        let relative_hitbox = Rect::new(adjusted_hitbox.position - (pos * 16).into(), adjusted_hitbox.size);
        let corner = Vector2D::new(relative_hitbox.position.x, relative_hitbox.position.y + relative_hitbox.size.y);
        corner.y > const_num_i32(16,0) - corner.x
      },

      Self::RSteepSlope => {
        let relative_hitbox = Rect::new(adjusted_hitbox.position - (pos * 16).into(), adjusted_hitbox.size);
        let corner = Vector2D::new(relative_hitbox.position.x + relative_hitbox.size.x, relative_hitbox.position.y + relative_hitbox.size.y);
        corner.y > corner.x
      },

      Self::LLowSlope1 => true,
      Self::LLowSlope2 => true,
      Self::RLowSlope1 => true,
      Self::RLowSlope2 => true,
    }
  }

  fn specialized_collide(self, pos: Vector2D<i32>, adjusted_hitbox: Rect<PosNum>, moving_left: bool, _moving_up: bool) -> Collision {
    match self {
      Self::LWall => Collision {
        x_seam: Some(if moving_left {pos.x * 16 + 2} else {pos.x * 16}),
        y_seam: None,
      },
      Self::RWall => Collision {
        x_seam: Some(if moving_left {(pos.x + 1) * 16} else {(pos.x + 1) * 16 - 2}),
        y_seam: None,
      },

      Self::LSteepSlope => {
        let relative_hitbox = Rect::new(adjusted_hitbox.position - (pos * 16).into(), adjusted_hitbox.size);
        Collision {
          x_seam: None,
          y_seam: Some(pos.y + (16 - relative_hitbox.position.x.trunc()))
        }
      }

      _ => Collision{ x_seam: None, y_seam: None }
    }
  }
}
