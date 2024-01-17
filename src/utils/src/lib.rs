use crate::device::Device;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref DEVICE: Device = Device::new(None);
}

pub mod device;