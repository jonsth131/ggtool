mod decoder;
mod directory;
mod easy_br;
use clap::{Parser, Subcommand};
mod keys;
use easy_br::EasyRead;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

fn open_file(file_name: &str) -> BufReader<File> {
    let file = File::open(&Path::new(file_name)).unwrap();
    let reader = BufReader::new(file);
    return reader;
}

fn read_bytes(reader: &mut BufReader<File>, count: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; count];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn read_root(exe_path: &str, file_name: &str) -> Result<Vec<u8>, std::io::Error> {
    let keys = keys::read_keys(exe_path)?;

    let mut reader = open_file(file_name);
    let offset = reader.read_u32_le()?;
    let size = reader.read_u32_le()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    let mut data = read_bytes(&mut reader, size as usize)?;

    decoder::decode_data(&mut data, &keys.key1, &keys.key2);

    Ok(data)
}

/// Return to Monkey Island ggpack tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to Return to Monkey Island.exe
    #[clap(value_parser)]
    exe_path: String,

    /// Path to the ggpack-file
    #[clap(value_parser)]
    pack_path: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Lists files in the ggpack
    ListFiles,
    /// Extracts a file
    ExtractFile {
        // Name of the file to extract
        filename: String,
    },
}

fn main() {
    let args = Args::parse();

    let directory_data = read_root(&args.exe_path, &args.pack_path).expect("Failed to read directory data");

    let directory = directory::Directory::parse(directory_data).expect("Failed to parse directory");

    let file_list = directory.get_files();

    match args.command {
        Commands::ListFiles => {
            let filenames: Vec<&String> = file_list.iter().map(|f| f.filename).collect();

            println!("{:#?}", filenames);
        }
        Commands::ExtractFile { filename } => {
            let file = file_list
                .iter()
                .find(|f| f.filename.eq(&filename))
                .expect("File not found in ggpack");

            // TODO!

            println!(
                "Extracting {}. Size = {}, offset = {}",
                file.filename, file.size, file.offset
            );
        }
    }
}
