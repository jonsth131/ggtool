# ggtool
Tools for ggpack files used in Return to Monkey Island.

# Usage
```
Return to Monkey Island ggpack tool

USAGE:
    ggtool <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    extract-file    Extracts a file
    extract-keys    Extracts encryption keys from Return to Monkey Island.exe
    help            Print this message or the help of the given subcommand(s)
    list-files      Lists files in the ggpack
```

## Extract keys
To use the tool the encryption keys need to be extracted from the 'Return to Monkey Island.exe' file.
To do this run `ggtool extract-keys <EXE_FILE>` and the keys will be extracted to the folder *keys*.

## List files in ggpack
To list existing files in a ggpack file run `ggtool list-files <PACK_PATH>`.

## Extract a file from a ggpack
To extract a file from a ggpack file run `ggtool extract-file <PACK_PATH> <FILENAME> <OUTPATH>`.