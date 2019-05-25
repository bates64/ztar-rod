use std::path::{Path, PathBuf};
use std::fs;
use std::io;

pub struct ModDir<'a> {
    root: &'a Path,
}

impl<'a> ModDir<'a> {
    pub fn open(root: &'a Path) -> ModDir<'a> {
        ModDir { root }
    }

    pub fn reset(&self) -> Result<(), io::Error> {
        let _ = fs::remove_dir_all(self.root);

        fs::create_dir(self.root)?;
        fs::create_dir(self.root.join("./map/"))?;

        Ok(())
    }

    pub fn asset_table(&self) -> PathBuf {
        self.root.join("./map/AssetTable.txt")
    }
}
