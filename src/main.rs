mod reader;

use reader::{read_metadata, read_root};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ggtool <pack-file>");
        std::process::exit(1);
    }

    let pack_path = &args[1];

    let mut data = read_root(&pack_path);
    // std::fs::write("out/blah.bin", &data).unwrap();

    let metatable = read_metadata(&data).expect("Failed to parse file metadata");
    for file in metatable {
        println!("{}", file);
    }
}
