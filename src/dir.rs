use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write, Seek};
use failure::{Error, bail};
use itertools::Itertools;
use crate::rom::{Rom, Location};
use crate::script::bc;

pub struct ModDir {
    root: PathBuf,
}

impl ModDir {
    /// Opens a ModDir for reading/writing. If it doesn't exist, it will be
    /// created. If the path exists but points to a file, returns `Err`.
    pub fn open(path: &str) -> Result<ModDir, Error> {
        let path = PathBuf::from(path);

        if !path.is_dir() {
            fs::create_dir(&path)?;
        } else if path.is_file() {
            bail!("Mod dir {:?} exists but is a file", path);
        }

        Ok(ModDir {
            root: path,
        })
    }

    /// Deletes all files within the `ModDir`. You shouldn't have to call this
    /// yourself, as `ModDir::dump()` calls it automatically.
    pub fn clear(&self) -> Result<(), Error> {
        fs::remove_dir_all(&self.root)?;
        fs::create_dir(&self.root)?;
        Ok(())
    }

    /// Dumps the given rom to the filesystem. You might want to `clear()`
    /// beforehand. Returns `Err` if files/directories exist but are the wrong
    /// type.
    pub fn dump<T: Read + Write + Seek>(&self, rom: &mut Rom<T>) -> Result<(), Error> {
        let areas_loc = Location::new(0x0006E8F0);
        for i in 0..28 {
            rom.seek(areas_loc.add_offset(i * 16));
            let area = rom.read_area();

            // TODO: dump maptable

            for map in area.maps.iter().unique_by(|map| map.name.clone()) {
                let mut path = self.root.clone();
                path.push(&area.name);
                path.push(&map.name);

                println!("dumping {}", map.name);

                if path.is_file() {
                    bail!("Map dir {:?} exists but is a file", path);
                } else if !path.is_dir() {
                    fs::create_dir_all(&path)?;
                }

                let path = path.to_str().unwrap().to_owned();

                // TODO: dump header data etc

                let mut script_f = File::create(path + "/script.txt")?;
                let blocks = map.main_func.scan_data_offsets();

                writeln!(script_f, "// {:?}\n", &map.dma)?;
                writeln!(script_f, "// main function\n{}", &map.main_func)?;

                for (datatype, offset) in blocks {
                    use bc::DataType::*;

                    writeln!(script_f, "\n// offset 0x{:04X} (rom 0x{:08X})", offset, map.dma.start + offset)?;

                    match datatype {
                        Function => {
                            rom.seek(map.dma.loc_at_offset(offset));
                            let bc = rom.read_bytecode(offset);

                            writeln!(script_f, "{}", bc)?;
                        },
                        Asm => writeln!(script_f, "// assembly here soon (tm)!")?,
                        Unknown => writeln!(script_f, "// unknown struct")?,
                    }
                }
            }
        }

        Ok(())
    }
}
