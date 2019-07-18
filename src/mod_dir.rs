use crate::render::Map;
use image::DynamicImage;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

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

    pub fn map(&self, filename: &str) -> PathBuf {
        self.root.join(format!("./map/{}.xz", filename))
    }

    pub fn texture(&self, filename: &str) -> PathBuf {
        self.root.join(format!(
            "./img/tex/{}/{}.png",
            area_name(filename),
            filename
        ))
    }

    pub fn read_bg(&self, name: &str) -> Result<DynamicImage, io::Error> {
        let reader = BufReader::new(File::open(self.background(name))?);
        image::load(reader, image::PNG).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub fn read_map(&self, name: &str) -> Result<Map, io::Error> {
        let filename = self.map(name);

        // Do we need to steal?
        if File::open(&filename).is_err() {
            println!("missing map '{}', trying to steal...", name);

            let steal = File::open(format!("steal/map/src/{}.json", name))?;
            let map: Map = serde_json::from_reader(steal)?;

            let target_file = File::create(&filename)?;
            let writer = XzEncoder::new(target_file, 6);
            serde_json::to_writer(writer, &map)?;
        }

        let file = File::open(&filename)?;
        let reader = XzDecoder::new(file);
        let map = serde_json::from_reader(reader)?;
        Ok(map)
    }

    pub fn read_tex(&self, name: &str) -> Result<DynamicImage, io::Error> {
        let filename = self.texture(name);

        // Do we need to steal?
        if File::open(&filename).is_err() {
            println!("missing texture '{}', trying to steal...", name);

            let mut steal = File::open(format!(
                "steal/image/texture/{}_tex/{}.png",
                area_name(name),
                name
            ))?;

            // create the area directory if necessary
            // ignore errors (e.g. if it already exists)
            let _ = fs::create_dir(format!("mod/img/tex/{}", area_name(name)));

            let mut target_file = File::create(&filename)?;
            io::copy(&mut steal, &mut target_file)?;
        }

        let reader = BufReader::new(File::open(&filename)?);
        image::load(reader, image::PNG).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

fn area_name(full: &str) -> &str {
    full.split('_').nth(0).unwrap()
}
