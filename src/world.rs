use agb::hash_map::HashMap;
use agb_ext::{
  collision::{Pos, Vel, Acc, system as colsys},
  ecs::{Entity},
};

type Map<T> = HashMap<Entity, T>;
type Entities = Map<()>;

type Components = (Map<Pos>, Map<Vel>, Map<Acc>,);

pub struct World {
  pub(self) components: Components,
  entities: Entities,
  next_entity_id: i32,
}

impl World {
  pub fn new() -> World {
    World {
      components: (Map::new(), Map::new(), Map::new(),),
      entities: Entities::new(),
      next_entity_id: 0,
    }
  }

  pub fn build_entity(&mut self) -> EntityBuilder {
    let en = Entity{ id: self.next_entity_id };
    self.next_entity_id += 1;
    self.entities.insert(en, ());
    EntityBuilder {
      world: self,
      en,
    }
  }

  pub fn frame(&mut self) {
    for (en, pos) in self.components.0.iter_mut() {
      if let Some(vel) = self.components.1.get(en) {
        colsys::apply_vel(pos, vel);
      }
      colsys::print_pos(en, pos);
    }
    for (en, vel) in self.components.1.iter_mut() {
      if let Some(acc) = self.components.2.get(en) {
        colsys::apply_acc(vel, acc);
      }
    }
  }
}


trait HasEntity {
  fn entity(&self) -> Entity;
}

pub trait WorldSetter<T>: HasEntity + Sized {
  fn component(&mut self) -> &mut Map<T>;

  fn set(mut self, val: T) -> Self {
    let en = self.entity();
    let mut component = self.component();
    component.insert(en, val);
    self
  }
}

pub struct EntityBuilder<'a> {
  world: &'a mut World,
  en: Entity,
}

impl<'a> EntityBuilder<'a> {
  pub fn build(self) -> Entity { self.en }
}

impl<'a> HasEntity for EntityBuilder<'a> {
  fn entity(&self) -> Entity { self.en }
}

macro_rules! impl_world_setter {
  ($t:ident, $i:tt) => {
    impl<'a> WorldSetter<$t> for EntityBuilder<'a> {
      fn component(&mut self) -> &mut Map<$t> { &mut self.world.components.$i }
    }
  }
}

impl_world_setter!(Pos, 0);
impl_world_setter!(Vel, 1);
impl_world_setter!(Acc, 2);
