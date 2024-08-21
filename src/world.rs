use agb::display::blend::Blend;
use agb::display::object::OamManaged;
use agb::display::tiled::VRamManager;
use agb::hash_map::HashMap;
use agb::input::ButtonController;
use agb::sound::mixer::Mixer;
use agb_ext::{
  collision::{Pos, Vel, Acc, OnGround, Size, system as colsys},
  ecs::{Entity},
  anim::{AnimOffset, AnimPlayer, system as anisys},
};
use agb_ext::blend::ManagedBlend;
use agb_ext::camera::Camera;
use agb_ext::collision::{CollideTilemap, CollisionLayer};
use agb_ext::tiles::Tilemap;
use crate::{
  player::{PlayerType, CurrentPlayer, system as playersys},
  object::{ForegroundHide, system as objsys}
};

pub type Map<T> = HashMap<Entity, T>;
type Entities = Map<()>;

type Components<'o> = (Map<Pos>, Map<Vel>, Map<Acc>, Map<Size>, Map<OnGround>, Map<CollisionLayer>, Map<PlayerType>, Map<AnimPlayer<'o>>, Map<AnimOffset>, Map<ForegroundHide>);

pub struct World<'o> {
  pub(self) components: Components<'o>,
  entities: Entities,
  next_entity_id: i32,
}

impl<'o> World<'o> {
  pub fn new() -> Self {
    World {
      components: (Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),Map::new(),),
      entities: Map::new(),
      next_entity_id: 0,
    }
  }

  pub fn build_entity<'w>(&'w mut self) -> MutEntityData<'w, 'o> {
    let en = Entity{ id: self.next_entity_id };
    self.next_entity_id += 1;
    self.entities.insert(en, ());
    MutEntityData {
      world: self,
      en,
    }
  }

  pub fn entity_data<'w>(&'w self, en: Entity) -> EntityData<'w, 'o> {
    EntityData {
      world: self,
      en
    }
  }

  pub fn entity_data_mut<'w>(&'w mut self, en: Entity) -> MutEntityData<'w, 'o> {
    MutEntityData {
      world: self,
      en
    }
  }

  pub fn frame(&mut self, input: &ButtonController, object: &'o OamManaged<'o>, camera: &mut Camera, collide_tilemap: &CollideTilemap, blend: &mut ManagedBlend) {
    for (en, vel) in self.components.1.iter_mut() {
      if let Some(acc) = self.components.2.get(en) {
        colsys::apply_acc(vel, acc);
      }
      let (pos, size) = (self.components.0.get(en), self.components.3.get(en));
      if let (Some(player_type), Some(on_ground)) = (self.components.6.get(en), self.components.4.get(en)) {
        playersys::player_movement(player_type, Some(&CurrentPlayer), vel, on_ground, input);
        if let (Some(pos), Some(size)) = (pos, size) {
          playersys::center_camera(&CurrentPlayer, pos, size, camera);
        }
      }
      if let (Some(pos), Some(size), Some(col_layer)) =
          (pos, size, self.components.5.get(en)) {
        colsys::physics_process(pos, vel, size, col_layer, self.components.4.get_mut(en), &collide_tilemap);
      }
    }
    for (en, pos) in self.components.0.iter_mut() {
      if let Some(vel) = self.components.1.get(en) {
        colsys::apply_vel(pos, vel);
      }
    }
    for (en, player) in self.components.7.iter_mut() {
      if let Some(pos) = self.components.0.get(en) {
        anisys::position_anim(player, pos, self.components.8.get(en), &camera);
      }
      anisys::draw(player, &object);
    }
    for (en, player_type) in self.components.6.iter() {
      if let Some(anim) = self.components.7.get_mut(en) {
        playersys::run_anim(player_type, anim, Some(&CurrentPlayer), object, input);
      }
      objsys::foreground_hide(&CurrentPlayer, en, &self.components.0, &self.components.3, &self.components.9, blend);
    }
  }
}


pub trait HasEntity {
  fn entity(&self) -> Entity;
}

pub trait EntityAccessor<T>: HasEntity + Sized {
  fn component(&self) -> &Map<T>;

  fn get(&self) -> Option<&T> {
    self.component().get(&self.entity())
  }
}

pub trait MutEntityAccessor<T>: HasEntity + Sized {
  fn component_mut(&mut self) -> &mut Map<T>;

  fn set(&mut self, val: T) -> &mut Self {
    let en = self.entity();
    let mut component = self.component_mut();
    component.insert(en, val);
    self
  }

  fn get_mut(&mut self) -> Option<&mut T> {
    let en = self.entity().clone();
    self.component_mut().get_mut(&en)
  }

  fn remove(&mut self) -> Option<T> {
    let en = self.entity().clone();
    self.component_mut().remove(&en)
  }
}

pub struct EntityData<'w, 'o> {
  world: &'w World<'o>,
  en: Entity,
}

pub struct MutEntityData<'w, 'o> {
  world: &'w mut World<'o>,
  en: Entity,
}

impl<'w, 'o> HasEntity for EntityData<'w, 'o> {
  fn entity(&self) -> Entity { self.en }
}

impl<'w, 'o> HasEntity for MutEntityData<'w, 'o> {
  fn entity(&self) -> Entity { self.en }
}

macro_rules! impl_entity_accessor {
  ($t:ty, $i:tt) => {
    impl<'w, 'o> EntityAccessor<$t> for EntityData<'w, 'o> {
      fn component(&self) -> &Map<$t> { &self.world.components.$i }
    }
    impl<'w, 'o> EntityAccessor<$t> for MutEntityData<'w, 'o> {
      fn component(&self) -> &Map<$t> { &self.world.components.$i }
    }

    impl<'w, 'o> MutEntityAccessor<$t> for MutEntityData<'w, 'o> {
      fn component_mut(&mut self) -> &mut Map<$t> { &mut self.world.components.$i }
    }
  }
}

impl_entity_accessor!(Pos, 0);
impl_entity_accessor!(Vel, 1);
impl_entity_accessor!(Acc, 2);
impl_entity_accessor!(Size, 3);
impl_entity_accessor!(OnGround, 4);
impl_entity_accessor!(CollisionLayer, 5);
impl_entity_accessor!(PlayerType, 6);
impl_entity_accessor!(AnimPlayer<'o>, 7);
impl_entity_accessor!(AnimOffset, 8);
impl_entity_accessor!(ForegroundHide, 9);
