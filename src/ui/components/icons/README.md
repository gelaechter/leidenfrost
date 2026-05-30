The icons in leidenfrost have been arranged using [icomoon](https://icomoon.io/).

The three files in this directory serve the following purpose:

1. `leidenfrost.json` is the icomoon project file which can be imported to make changes to the icon pack
2. `leidenfrost.icomoon.json` are the icon definitions containing metadata like the icon names and their corresponding codepoints
3. `leidenfrost.ttf` is a font which displays the corresponding codepoints as the given icons

The `build.rs` build-scripts reads the icon definitions in `leidenfrost.icomoon.json` and generates `src/ui/components/icons.rs` based on that to provide an easy interface to displaying the icons in leidenfrost.