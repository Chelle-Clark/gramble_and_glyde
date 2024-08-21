use agb::{
  fixnum::{Rect, Num},
  display::blend::{Blend, Layer},
};
use agb_ext::{
  collision::Entity,
  math::PosNum,
};

#[derive(Copy, Clone)]
pub enum ObjectInit {
  ForegroundHide(i32, i32, i32, i32),
}

pub enum GameObject {
  ForegroundHide{
    hitbox: Rect<PosNum>,
    opacity: Num<u8, 4>,
  }
}

impl ObjectInit {
  pub fn build(self) -> GameObject {
    match self {
      Self::ForegroundHide(x, y, w, h) => GameObject::ForegroundHide {
        hitbox: Rect::new((x, y).into(), (w, h).into()),
        opacity: Num::from_raw(0),
      }
    }
  }
}

impl GameObject {
  // pub fn frame(&mut self, player: &Player, blend: &mut Blend) {
  //   match self {
  //     Self::ForegroundHide {hitbox, opacity} => {
  //       if player.col_rect().touches(*hitbox) {
  //         *opacity += Num::from_raw(1);
  //       } else if *opacity > Num::from_raw(0) {
  //         *opacity -= Num::from_raw(1);
  //       }
  //       *opacity = *opacity.clamp(&mut Num::from_raw(0), &mut Num::new(1));
  // 
  //       blend.set_blend_weight(Layer::Top, Num::new(1) - *opacity);
  //       blend.set_blend_weight(Layer::Bottom, *opacity);
  //     }
  //   }
  // }
}
