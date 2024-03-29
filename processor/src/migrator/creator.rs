use chrono::offset::Local;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;

use std::path::PathBuf;

pub fn hasher<T: Hash>(t: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn create_migration_blank_file(dir: PathBuf) {
    let mut dir = dir;
    let ext = "sql";
    let t_time = Local::now().format("%s");
    let time = format!("{}.{}", t_time, ext);
    dir.push(time);

    let full_display_path = dir.display();
    match File::create(dir.clone()) {
        Ok(_) => println!("created migration file: {}", full_display_path),
        Err(e) => eprintln!("unable to create new migration file: {}", e),
    }
}
