mod decoder;
mod directory;
mod easy_br;
use clap::{Parser, Subcommand};
mod keys;
use easy_br::EasyRead;
use keys::Keys;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use crate::decoder::decode_data;

pub fn decode_at(
    reader: &mut BufReader<File>,
    keys: &Keys,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, std::io::Error> {
    reader.seek(SeekFrom::Start(offset))?;
    let mut data = read_bytes(reader, size)?;
    decoder::decode_data(&mut data, &keys.key1, &keys.key2);
    Ok(data)
}

fn read_bytes(reader: &mut BufReader<File>, count: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; count];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
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
        // Output path
        outpath: String
    },
}

fn main() {
    let args = Args::parse();
    let keys = keys::read_keys(&args.exe_path).expect("Failed to extract keys from exe file");

    let file = File::open(&Path::new(&args.pack_path)).unwrap();
    let mut reader = BufReader::new(file);

    let offset = reader
        .read_u32_le()
        .expect("Failed to read directory offset") as u64;
    let size = reader.read_u32_le().expect("Failed to read directory size") as usize;

    let directory_data =
        decode_at(&mut reader, &keys, offset, size).expect("Failed to decode directory");
    let directory = directory::Directory::parse(directory_data).expect("Failed to parse directory");

    let file_list = directory.get_files();

    match args.command {
        Commands::ListFiles => {
            let filenames: Vec<&String> = file_list.iter().map(|f| f.filename).collect();

            println!("{:#?}", filenames);
        }
        Commands::ExtractFile { filename, outpath } => {
            let file = file_list
                .iter()
                .find(|f| f.filename.eq(&filename))
                .expect("File not found in ggpack");

            println!(
                "Extracting {}. Size = {}, offset = {}",
                file.filename, file.size, file.offset
            );

            reader
                .seek(SeekFrom::Start(file.offset))
                .expect("Failed to seek to offset");
            let mut data = read_bytes(&mut reader, file.size).expect("Failed to read data");
            decode_data(&mut data, &keys.key1, &keys.key2);

            std::fs::write(outpath + "/" + file.filename, data).expect("Failed to write data to disk");
        }
    }
}
