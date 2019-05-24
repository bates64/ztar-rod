use std::path::PathBuf;
use std::io::{self, BufWriter, Write};
use std::fs::{self, File};
use failure_derive::*;
use itertools::Itertools;
use crate::rom::{Rom, loc::Location};
use crate::script;

fn create_dir_if_not_exist(path: &str) -> Result<(), Error> {
    let path = PathBuf::from(path);

    if !path.is_dir() {
        fs::create_dir(&path)?;
        Ok(())
    } else if path.is_file() {
        Err(Error::DirectoryIsFile(path.to_str().unwrap().to_string()))
    } else {
        Ok(())
    }
}

pub struct ModDir {
    root: String,
}

impl ModDir {
    /// Opens a ModDir for reading/writing. If it doesn't exist, it will be
    /// created. If the path exists but points to a file, returns `Err`.
    pub fn open(path: &str) -> Result<ModDir, Error> {
        create_dir_if_not_exist(path)?;
        Ok(ModDir {
            root: path.to_string(),
        })
    }

    /// Deletes all files within the `ModDir`. You shouldn't have to call this
    /// yourself, as `ModDir::dump()` calls it automatically.
    pub fn clear(&self) -> Result<(), Error> {
        fs::remove_dir_all(&self.root)?;
        fs::create_dir(&self.root)?;
        Ok(())
    }

    /// Dumps the given rom to the filesystem.
    pub fn dump(&self, rom: &mut Rom) -> Result<(), Error> {
        let areas_loc = Location::new(0x0006E8F0);
        for i in 0..28 {
            rom.seek(areas_loc.add_offset(i * 16));
            let area = rom.read_area();

            create_dir_if_not_exist(&format!("{}/{}", self.root, area.name))?;

            for map in area.maps.into_iter().unique_by(|map| map.name.clone()) {
                println!("-- dumping {} --", map.name);

                // Write script file
                {
                    let path = format!("{}/{}/{}.script", self.root, area.name, map.name);
                    let file = File::create(&path)?;

                    let mut w = BufWriter::new(&file);
                    write!(w, "{}",
                        script::decompile_map(map, rom)
                        .or_else(|err| Err(Error::ScriptDecompilation(path, err)))?
                    )?;
                }

                // TODO other stuff
            }
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "io error: {}", _0)]
    Io(#[fail(cause)] io::Error),

    #[fail(display = "file {} should be a directory", _0)]
    DirectoryIsFile(String),

    #[fail(display = "error dumping {} ({})", _0, _1)]
    ScriptDecompilation(String, #[fail(cause)] script::Error)
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
