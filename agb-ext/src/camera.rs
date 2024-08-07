use agb::fixnum::Vector2D;
use crate::math::{PosNum, ZERO};

const SCREEN_W: i32 = 240;
const SCREEN_H: i32 = 160;

pub struct Camera {
  pos: Vector2D<PosNum>,
  max_limit: Vector2D<PosNum>,
}

impl Camera {
  pub fn new() -> Self {
    Camera {
      pos: Vector2D::new(ZERO, ZERO),
      max_limit: Vector2D::new(PosNum::new(SCREEN_W + 16), PosNum::new(SCREEN_H)),
    }
  }

  pub fn center_on(&mut self, pos: Vector2D<PosNum>) {
    self.set_position(pos - Vector2D::new(SCREEN_W / 2, SCREEN_H / 2).into());
  }

  pub fn set_position(&mut self, pos: Vector2D<PosNum>) {
    self.pos.x = pos.x.clamp(ZERO, self.max_limit.x - PosNum::new(SCREEN_W));
    self.pos.y = pos.y.clamp(ZERO, self.max_limit.y - PosNum::new(SCREEN_H));
  }

  pub fn position(&self) -> Vector2D<PosNum> {
    self.pos
  }

  pub fn position_i16(&self) -> Vector2D<i16> {
    let trunc_pos = self.pos.trunc();
    Vector2D::new(trunc_pos.x as i16, trunc_pos.y as i16)
  }
}
