# ggtool
Command-line tool for unpacking *.ggpack-files used in Return to Monkey Island.

# Building
You need Rust edition 2021. The easiest way to install Rust is via rustup: https://rustup.rs/

Once you have Rust installed, you can run the following command in the cloned folder to build an executable:
```
cargo build --release
```

libdinky includes a feature to decompress and convert KTX-textures to PNG. It can be enabled by building with
```
cargo build --release --features decompress_ktx
```
Whenever a *.ktxbz file is extracted, it is then also converted to PNG and saved alongside the extracted file.

Note that this uses your OpenGL-driver for decompressing the BPTC-compressed data. Your milage may vary depending on your GPU and how updated your drivers are.

# Usage
```
Return to Monkey Island ggpack tool

USAGE:
    ggtool.exe <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    extract-files    Extracts files
    extract-keys     Extracts encryption keys from Return to Monkey Island.exe
    help             Print this message or the help of the given subcommand(s)
    list-files       Lists files in the ggpack
```

## Extract keys
To use the tool the encryption keys need to be extracted from the 'Return to Monkey Island.exe' file.

To do this run `ggtool extract-keys <EXE_FILE>` and the keys will be extracted to the folder *keys*.

NB: This has been confirmed working with the macOS-binary as well.

## List files in ggpack
To list existing files in a ggpack file run `ggtool list-files <PACK_PATH>`.

## Extract file(s) from a ggpack
To extract one or more file from a ggpack file, run `ggtool.exe extract-files <PACK_PATH> <PATTERN> <OUTPATH> [decompile-yack]`.

Where `<PATTERN>` is a glob-pattern of the files to extract. For instance `AnchorKey02-hd.ktxbz` or `Anchor*`.

If you supply `decompile-yack`. ggtool will also spit out text-readable \*.yack files.
