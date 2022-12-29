use std::path::PathBuf;

const USAGES_STR: &str = include_str!("usages.txt");

mod editor;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if !([3, 5].contains(&args.len())) {
        println!("{}", USAGES_STR);
        return;
    }

    let command = args[1].clone();

    match command.as_str() {
        "--edit" => {
            let ron_file_name = args[2].clone();
            let ron_path: PathBuf = (&ron_file_name).into();
            editor::open_for_edit(&ron_path);
        }
        "--create" => {
            let ron_file_name = args[2].clone();
            let ron_path: PathBuf = (&ron_file_name).into();
            if args.len() != 5 {
                println!("{}", USAGES_STR);
                return;
            }
            let width: usize = args[3].parse().unwrap();
            let height: usize = args[4].parse().unwrap();
            let map = rl23_map_format::MapInfo::create_new(width, height);
            map.save_to_path(&ron_path);
            editor::open_for_edit(&ron_path);
        }
        _ => {
            println!("{}", USAGES_STR);
            return;
        }
    }
}
