use agb::{
  fixnum::{Rect, Num},
  display::blend::{Blend, Layer},
};
use agb_ext::{
  math::PosNum,
};
use agb_ext::collision::{Pos, Size};
use agb_ext::ecs::{Entity, HasEntity, MutEntityAccessor, Map};
use agb_ext::math::ZERO;
use crate::{Vector2D, World};

#[derive(Copy, Clone)]
pub enum ObjectInit {
  ForegroundHide(i32, i32, i32, i32),
}

pub struct ForegroundHide;

impl ObjectInit {
  pub fn build(self, world: &mut World) -> Entity {
    match self {
      Self::ForegroundHide(x, y, w, h) => {
        world.build_entity()
          .set(Pos((x, y).into()))
          .set(Size((w, h).into()))
          .set(ForegroundHide)
          .entity()
      }
    }
  }
}

pub mod system {
  use agb_ext::blend::ManagedBlend;
  use super::*;
  use crate::player::CurrentPlayer;

  pub fn foreground_hide(_: &CurrentPlayer, current_player_en: &Entity, pos_map: &Map<Pos>, size_map: &Map<Size>, foreground_hide_map: &Map<ForegroundHide>, blend: &mut ManagedBlend) {
    if player_colliding(current_player_en, pos_map, size_map, foreground_hide_map.keys()).is_some() {
      blend.min_dec_top_opacity();
    } else {
      blend.min_inc_top_opacity();
    }
  }
}

fn player_colliding<'e>(current_player_en: &Entity, pos_map: &Map<Pos>, size_map: &Map<Size>, obj_iter: impl Iterator<Item=&'e Entity>) -> Option<Entity> {
  if let (Some(player_pos), Some(player_size)) = (pos_map.get(current_player_en), size_map.get(current_player_en)) {
    let player_rect = Rect::new(player_pos.0, player_size.0);
    for obj_en in obj_iter {
      if let (Some(obj_pos), Some(obj_size)) = (pos_map.get(obj_en), size_map.get(obj_en)) {
        let obj_rect = Rect::new(obj_pos.0, obj_size.0);
        if player_rect.touches(obj_rect) {
          return Some(obj_en.clone());
        }
      }
    }
  }
  return None;
}
