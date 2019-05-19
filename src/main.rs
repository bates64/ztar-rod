mod rom;
mod script;
mod dir;

use std::fs::File;
use rom::Rom;
use dir::ModDir;

fn main() {
    match File::open("Paper Mario (U) [!].z64") {
        Err(_) => println!("Unable to open 'Paper Mario (U) [!].z64'. Please copy a clean rom to the current working-directory and retry!"),
        Ok(file) => {
            let mut rom = Rom::from(file);
            let mod_dir = ModDir::open("./mod/").unwrap();

            //mod_dir.clear().unwrap();
            mod_dir.dump(&mut rom).unwrap();
        }
    }
}
