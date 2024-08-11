use tiled::{
  Loader,
};
use tiled_export::{export_level, export_tileset};

const LEVELS: &[&str] = &[
  "grambles_room",
  "single_screen_demo",
  "slope_test",
];

fn main() -> std::io::Result<()> {
  let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
  let mut loader = Loader::new();

  export_tileset("tileset", "metatileset", &out_dir, &mut loader)?;
  for level in LEVELS {
    export_level(level, &out_dir, &mut loader)?;
  }
  Ok(())
}

mod tiled_export {
  use std::fmt::{Display, Formatter};
  use tiled::{Loader, LayerType, TileLayer, LayerTileData, TileId};
  use std::fs::File;
  use std::io::{BufWriter, Result, Write};

  const CLEAR_COLOR: &str = "333333";

  struct DeserializedFlipTile {
    tile_id: TileId,
    suffix: &'static str,
    flip: &'static str,
  }

  impl Display for DeserializedFlipTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
      write!(f, "FlipTile::{}({}{})", self.flip, self.tile_id, self.suffix)
    }
  }

  pub fn export_tileset(tileset: &str, metatileset: &str, out_dir: &str, loader: &mut Loader) -> Result<()> {
    let tileset_path = format!("maps/{tileset}.tsx");
    println!("cargo::rerun-if-changed={tileset_path}");
    let tileset = loader.load_tsx_tileset(tileset_path).unwrap();
    let metatileset_path = format!("maps/{metatileset}.tmx");
    println!("cargo::rerun-if-changed={metatileset_path}");
    let metatileset = loader.load_tmx_map(metatileset_path).unwrap();

    let output_file = File::create(format!("{out_dir}/tileset.rs"))?;
    let mut writer = BufWriter::new(output_file);

    let image_source = {
      if let Some(image) = tileset.image {
        image.source
          .into_os_string()
          .into_string()
          .expect("Couldn't convert image path to String.")
          .replace("\\", "/")
      } else {
        panic!("No single image defined for base tileset");
      }
    };
    writeln!(&mut writer, r#"
    use agb_ext::tiles::{{TileSetData, Metatile, FlipTile}};

    agb::include_background_gfx!(tileset, "{CLEAR_COLOR}", background => "{image_source}");
    "#)?;

    if let Some(LayerType::Tiles(tile_layer)) = metatileset.layers().next().map(|l| l.layer_type()) {
      if let TileLayer::Finite(layer) = tile_layer {
        writeln!(&mut writer, "const METATILES: &[Metatile] = &[")?;
        for yi in (0..layer.height()).step_by(2) {
          for xi in (0..layer.width()).step_by(2) {
            let xi = xi as i32;
            let yi = yi as i32;
            let ul = get_tile_id(layer.get_tile_data(xi, yi));
            let ur = get_tile_id(layer.get_tile_data(xi + 1, yi));
            let ll = get_tile_id(layer.get_tile_data(xi, yi + 1));
            let lr = get_tile_id(layer.get_tile_data(xi + 1, yi + 1));

            writeln!(&mut writer, "Metatile::new({ul},{ur},{ll},{lr}),")?;
          }
        }
        writeln!(&mut writer, "];")?;
      } else {
        panic!("Infinite tile layer not supported for metatileset");
      }
    } else {
      panic!("Non-tile type layer not supported for metatileset")
    }

    writeln!(
      &mut writer,
      r#"
      pub static TILESET_DATA: TileSetData = TileSetData{{
        metatiles: &METATILES,
        palettes: tileset::PALETTES,
        tile_data: &tileset::background,
      }};
      "#
    )?;

    Ok(())
  }

  fn get_flip_str(tile: &LayerTileData) -> &'static str {
    match (tile.flip_h, tile.flip_v) {
      (false, false) => "N",
      (false, true) => "Y",
      (true, false) => "X",
      (true, true) => "XY",
    }
  }

  fn get_tile_id(tile: Option<&LayerTileData>) -> DeserializedFlipTile {
    if let Some(tile) = tile {
      DeserializedFlipTile {
        tile_id: tile.id() + 1,
        suffix: "",
        flip: get_flip_str(tile),
      }
    } else {
      DeserializedFlipTile {
        tile_id: 0,
        suffix: "",
        flip: "N",
      }
    }
  }

  pub fn export_level(level: &str, out_dir: &str, loader: &mut Loader) -> Result<()> {
    let full_path = format!("maps/{level}.tmx");
    println!("cargo::rerun-if-changed={full_path}");
    let map = loader.load_tmx_map(full_path).unwrap();

    let output_file = File::create(format!("{out_dir}/{level}.rs"))?;
    let mut writer = BufWriter::new(output_file);

    let mut has_background = false;
    let mut has_foreground = false;
    for layer in map.layers() {
      let layer_name = layer.name.clone();
      match layer.layer_type() {
        LayerType::Tiles(tile_layer) => {
          match tile_layer {
            TileLayer::Finite(layer) => {
              if layer_name.as_str() == "Collision" {
                write!(&mut writer, "const COLLISION: &[C] = &[")?;
                for yi in 0..layer.height() {
                  for xi in 0..layer.width() {
                    let collide_tile_type = match layer.get_tile_data(xi as i32, yi as i32) {
                      Some(tile) => get_collide_tile_type(tile.id()),
                      None => "Pass",
                    };
                    write!(&mut writer, "C::{},", collide_tile_type)?;
                  }
                }
                writeln!(&mut writer, "];")?;
              } else {
                let const_name = match layer_name.as_str() {
                  "Primary" => "DATA",
                  "Background" => {
                    has_background = true;
                    "BACKGROUND_DATA"
                  },
                  "Foreground" => {
                    has_foreground = true;
                    "FOREGROUND_DATA"
                  }
                  _ => "",
                };
                write!(&mut writer, "const {const_name}: &[FlipTile<u8>] = &[")?;
                for yi in 0..layer.height() {
                  for xi in 0..layer.width() {
                    let tile_id = get_metatile_id(layer.get_tile_data(xi as i32, yi as i32));
                    write!(&mut writer, "{},", tile_id)?;
                  }
                }
                writeln!(&mut writer, "];")?;
              }
            },
            _ => {
              panic!("Infinite tile layers not supported!");
            }
          }
        },
        LayerType::Objects(_) => {

        },
        _ => {

        },
      }
    }

    let map_w = map.width;
    let background_data = if has_background {"Some(&BACKGROUND_DATA)"} else {"None"};
    let foreground_data = if has_foreground {"Some(&FOREGROUND_DATA)"} else {"None"};
    writeln!(
      &mut writer,
      r#"
      use agb_ext::{{
        tiles::{{Tilemap, FlipTile}},
        collision::CollideTileType as C,
      }};
      use crate::tileset;

      pub static TILEMAP: Tilemap = Tilemap::new(&DATA, {background_data}, {foreground_data}, &COLLISION, {map_w}, &tileset::TILESET_DATA);
      "#
    )?;

    Ok(())
  }

  fn get_metatile_id(tile: Option<&LayerTileData>) -> DeserializedFlipTile {
    if let Some(tile) = tile {
      DeserializedFlipTile {
        tile_id: tile.id() + 1,
        suffix: "_u8",
        flip: get_flip_str(tile),
      }
    } else {
      DeserializedFlipTile {
        tile_id: 0,
        suffix: "_u8",
        flip: "N",
      }
    }
  }

  fn get_collide_tile_type(tile: u32) -> &'static str {
    match tile {
      0 => "Solid",
      1 => "LWall",
      2 => "RWall",
      3 => "Pipe",
      4 => "RSteepSlope",
      5 => "RLowSlope1",
      6 => "RLowSlope2",
      7 => "PipeSolid",
      8 => "LSteepSlope",
      9 => "LLowSlope1",
      10 => "LLowSlope2",
      _ => "Pass",
    }
  }
}