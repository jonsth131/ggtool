use clap::Parser;
use libdinky;

///Return to Monkey Island ggpack tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Args {
    ///Extracts encryption keys from Return to Monkey Island.exe
    ExtractKeys {
        ///Path to Return to Monkey Island.exe
        exe_path: String,
    },
    ///Lists files in the ggpack
    ListFiles {
        ///Path to the ggpack-file
        pack_path: String,
    },
    ///Extracts files
    ExtractFiles {
        ///Path to the ggpack-file
        pack_path: String,
        ///Pattern of files to extract.
        ///For instance *.yack extracts all yack-files.
        pattern: String,
        ///Output path
        outpath: String,
        ///Optional: Decompile *.yack files
        decompile_yack: bool,
    },
    /// Dink handling
    Dink {
        /// Dink file
        file: String,
    },
}

fn extract_keys(exe_path: &str) {
    let keys =
        libdinky::Keys::extract_from_exe(exe_path).expect("Failed to extract keys from exe file");
    if !std::path::Path::new("keys").exists() {
        std::fs::create_dir("keys").expect("Failed to create dir 'keys'");
    }
    std::fs::write("keys/key1.bin", keys.key1).expect("Failed to write keys/key1.bin");
    std::fs::write("keys/key2.bin", keys.key2).expect("Failed to write keys/key2.bin");
    std::fs::write("keys/key3.bin", keys.key3).expect("Failed to write keys/key3.bin");

    println!("Keys extracted successfully!");
}

fn main() {
    let args = Args::parse();
    match args {
        Args::ExtractKeys { exe_path } => extract_keys(&exe_path),
        Args::ListFiles { pack_path } => {
            let pack =
                libdinky::ggpack::OpenGGPack::from_path(&pack_path).expect("Failed to open ggpack");
            pack.list_files()
        }
        Args::ExtractFiles {
            pack_path,
            pattern,
            outpath,
            decompile_yack,
        } => {
            let mut pack =
                libdinky::ggpack::OpenGGPack::from_path(&pack_path).expect("Failed to open ggpack");

            pack.extract_files(&pattern, &outpath, decompile_yack)
        }
        Args::Dink { file } => {
            let result = libdinky::dink::read_dink(&file).expect("Failed to open dink file");
            println!("{:?}", result);
        }
    }
}
