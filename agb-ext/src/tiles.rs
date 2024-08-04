use agb::{display::{
  tiled::{MapLoan, VRamManager, RegularMap, TileSet},
  tile_data::TileData,
  palette16::Palette16,
}, fixnum::{Vector2D, Rect, num}};
use crate::math::{PosNum, ZERO, MIN_INC};

pub struct TileSetData {
  pub metatiles: &'static [[usize; 4]],
  pub palettes: &'static [Palette16],
  pub tile_data: &'static TileData,
}

pub struct Tilemap {
  data: &'static [u8],
  width: usize,
  height: usize,
  tileset: &'static TileSet<'static>,
  tileset_data: TileSetData,
}

#[derive(Clone)]
pub struct Collision {
  pub x_seam: Option<i32>,
  pub y_seam: Option<i32>,
}

impl Tilemap {
  pub fn new(data: &'static [u8], width: usize, tileset_data: TileSetData) -> Self {
    Tilemap {
      data,
      width,
      height: data.len() / width,
      tileset: &tileset_data.tile_data.tiles,
      tileset_data,
    }
  }

  pub fn draw_background(&self, background: &mut MapLoan<RegularMap>, vram: &mut VRamManager) {
    vram.set_background_palettes(self.tileset_data.palettes);
    for (i, metatile) in self.data.iter().enumerate() {
      if *metatile != 0_u8 {
        let x = 2 * (i % self.width) as u16;
        let y = 2 * (i / self.width) as u16;
        let metatile_idx = (*metatile - 1) as usize;
        let metatile = self.tileset_data.metatiles[metatile_idx];
        let tile_settings = self.tileset_data.tile_data.tile_settings;
        background.set_tile(vram, (x, y), self.tileset, tile_settings[metatile[0]]);
        background.set_tile(vram, (x + 1, y), self.tileset, tile_settings[metatile[1]]);
        background.set_tile(vram, (x, y + 1), self.tileset, tile_settings[metatile[2]]);
        background.set_tile(vram, (x + 1, y + 1), self.tileset, tile_settings[metatile[3]]);
      }
    }
  }

  pub fn get_collision_seams(&self, movement: Vector2D<PosNum>, hitbox: Rect<PosNum>) -> Collision {
    let px_per_tile = PosNum::new(16);
    let entered_x = {
      let cur_edge = {
        if movement.x > ZERO {
          hitbox.position.x + hitbox.size.x - MIN_INC
        } else {
          hitbox.position.x
        }
      };
      if (cur_edge / px_per_tile).floor() != ((cur_edge + movement.x) / px_per_tile).floor() {
        Some(((cur_edge + movement.x) / px_per_tile).floor())
      } else {
        None
      }
    };
    let entered_y = {
      let cur_edge = {
        if movement.y > ZERO {
          hitbox.position.y + hitbox.size.y - MIN_INC
        } else {
          hitbox.position.y
        }
      };
      if (cur_edge / px_per_tile).floor() != ((cur_edge + movement.y) / px_per_tile).floor() {
        Some(((cur_edge + movement.y) / px_per_tile).floor())
      } else {
        None
      }
    };


    let y_seam = {
      if let Some(entered_y) = entered_y {
        if entered_y >= 0 && entered_y < self.height as i32 {
          let tilemap_entered_y = entered_y as usize;
          let (tile_left_x, tile_right_x) = {
            let left_x = hitbox.position.x;
            let right_x = left_x + hitbox.size.x - MIN_INC;
            ((left_x / px_per_tile).floor(), (right_x / px_per_tile).floor())
          };
          let mut result = None;
          for i in tile_left_x..=tile_right_x {
            if i >= 0 && i < self.width as i32 && self.data[tilemap_entered_y * self.width + i as usize] != 0 {
              let upper_y = {
                if movement.y < ZERO {
                  entered_y + 1
                } else {
                  entered_y
                }
              };
              result = Some(upper_y * 16);
            }
          }
          result
        } else {
          None
        }
      } else {
        None
      }
    };
    let x_seam = {
      if let Some(entered_x) = entered_x {
        if entered_x >= 0 && entered_x < self.width as i32 {
          let tilemap_entered_x = entered_x as usize;
          let (tile_up_y, tile_down_y) = {
            let up_y = hitbox.position.y;
            let down_y = up_y + hitbox.size.y - MIN_INC;
            ((up_y / px_per_tile).floor(), (down_y / px_per_tile).floor())
          };
          let mut result = None;
          for i in tile_up_y..=tile_down_y {
            if i >= 0 && i < self.height as i32 && self.data[tilemap_entered_x + i as usize * self.width] != 0 {
              let left_x = {
                if movement.x < ZERO {
                  entered_x + 1
                } else {
                  entered_x
                }
              };
              result = Some(left_x * 16);
            }
          }
          result
        } else {
          None
        }
      } else {
        None
      }
    };

    return Collision {
      x_seam,
      y_seam,
    }
  }
}