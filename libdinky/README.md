# libdinky
Parses data files from Return to Monkey Island.

Currently supports parsing
* ggpack archives
* yack-files (partial)
* json/wimpy files
* ktxbz textures

There is a feature `decompress_ktx` that enables PNG conversion of ktxbz textures.
In that case the extractor will spit out a .PNG file in addition to the inflated KTX texture.

It can be enabled by building ggtool like this:

`cargo build --features decompress_ktx`