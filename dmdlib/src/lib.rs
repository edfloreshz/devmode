use crate::utils::constants::messages::{DATA_DIR_NOT_CREATED, HOME_DIR_NOT_CREATED};
use std::path::PathBuf;

pub mod utils;

pub fn data() -> PathBuf {
    dirs::data_dir().expect(DATA_DIR_NOT_CREATED)
}

pub fn home() -> PathBuf {
    dirs::home_dir().expect(HOME_DIR_NOT_CREATED)
}
