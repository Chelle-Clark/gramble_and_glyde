use agb::fixnum::{
  Num,
};

pub type PosNum = Num<i32, 8>;

pub const ZERO: PosNum = PosNum::from_raw(0);
pub const MIN_INC: PosNum = PosNum::from_raw(1);