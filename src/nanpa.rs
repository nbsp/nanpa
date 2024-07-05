use std::{fs, io, process};
use crate::util;

pub struct Nanpa {
    config: io::BufReader<fs::File>,
}

impl Nanpa {
    pub fn new() -> Self {
        Self { 
            config: match util::find_root() {
                Some(path) => { 
                    match fs::File::open(path.join(".nanparc")) {
                        Ok(file) => io::BufReader::new(file),
                        Err(e) => { 
                            util::error(e.to_string().as_str());
                            process::exit(1);
                        },
                    }
                },
                None => {
                    util::error("could not find .nanparc file, refer to nanparc(5)");
                    process::exit(1);
                }
            } 
        }
    }
}

pub fn new() -> Nanpa {
    Nanpa::new()
}
