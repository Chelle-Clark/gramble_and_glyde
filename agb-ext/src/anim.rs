use agb::{
  display::object::{Object, OamManaged, Tag},
};

#[derive(Clone, Copy)]
pub struct Frame {
  pub tag: &'static Tag,
  pub tag_idx: u8,
  pub duration: u8,
}

#[derive(Clone, Copy)]
pub struct Anim<Enum> {
  pub frames: &'static [Frame],
  pub next_anim: Option<Enum>,
}

pub struct AnimPlayer<'o, Enum> {
  get_next_anim: fn(Enum) -> Anim<Enum>,
  cur_anim: Anim<Enum>,
  cur_anim_enum: Enum,
  frame_idx: usize,
  frame_duration: u8,

  sprite: Object<'o>
}

#[macro_export]
macro_rules! new_anim {
  ( $tag:expr, $next_anim:expr, $( $frame_data:expr ),* ) => {
    {
      use agb_ext::anim::Frame;
      static frames: &[Frame] = &[
        $(
          Frame {
            tag: $tag,
            tag_idx: $frame_data.0,
            duration: $frame_data.1,
          },
        )*
      ];
      Anim {
        frames,
        next_anim: $next_anim,
      }
    }
  }
}

impl<'o, Enum: Clone + PartialEq> AnimPlayer<'o, Enum> {
  pub fn new(object: &'o OamManaged, get_next_anim: fn(Enum) -> Anim<Enum>, first_anim_enum: Enum) -> AnimPlayer<'o, Enum> {
    let first_anim = get_next_anim(first_anim_enum.clone());
    let first_frame = first_anim.frames[0];
    let mut sprite = object.object_sprite(first_frame.tag.sprite(first_frame.tag_idx as usize));
    sprite.show();

    AnimPlayer {
      get_next_anim,
      cur_anim: first_anim,
      cur_anim_enum: first_anim_enum,
      frame_idx: 0,
      frame_duration: first_frame.duration,
      sprite,
    }
  }

  pub fn draw(&mut self, object: &'o OamManaged) {
    self.frame_duration -= 1;
    if self.frame_duration == 0 {
      self.frame_idx += 1;
      if self.frame_idx == self.cur_anim.frames.len() {
        if let Some(next_anim_enum) = self.cur_anim.next_anim.clone() {
          self.force_set_anim(next_anim_enum, object);
        }
      } else {
        self.load_frame(object);
      }
    }
  }

  pub fn set_anim(&mut self, anim: Enum, object: &'o OamManaged) {
    if self.cur_anim_enum != anim {
      self.force_set_anim(anim, object);
    }
  }

  pub fn force_set_anim(&mut self, anim: Enum, object: &'o OamManaged) {
    self.cur_anim_enum = anim.clone();
    let next_anim = (self.get_next_anim)(anim);
    self.cur_anim = next_anim;
    self.frame_idx = 0;
    self.load_frame(object);
  }

  fn load_frame(&mut self, object: &'o OamManaged) {
    let frame = self.cur_anim.frames[self.frame_idx];
    self.sprite.set_sprite(object.sprite(frame.tag.sprite(frame.tag_idx as usize)));
    self.frame_duration = frame.duration;
  }

  pub fn sprite(&self) -> &Object<'o> {
    &self.sprite
  }

  pub fn sprite_mut(&mut self) -> &mut Object<'o> {
    &mut self.sprite
  }

  pub fn cur_anim(&self) -> Enum {
    self.cur_anim_enum.clone()
  }
}
