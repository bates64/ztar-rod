use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
        fs::create_dir(self.root.join("./build/"))?;
        fs::create_dir(self.root.join("./map/"))?;
        fs::create_dir(self.root.join("./img/"))?;
        fs::create_dir(self.root.join("./img/bg/"))?;
        fs::create_dir(self.root.join("./img/tex/"))?;

        Ok(())
    }

    pub fn built_asset(&self, asset_name: &str) -> PathBuf {
        self.root.join(format!("./build/{}", asset_name))
    }

    pub fn asset_table(&self) -> PathBuf {
        self.root.join("./map/AssetTable.txt")
    }

    pub fn background(&self, filename: &str) -> PathBuf {
        self.root.join(format!("./img/bg/{}.png", filename))
    }
}
