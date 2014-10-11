# rs-tiled

![Travis](https://travis-ci.org/mattyhall/rs-tiled.svg?branch=master)

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust for use in video games. It is game engine agnostic and pretty barebones at the moment. Documentation is available [on rust-ci](http://rust-ci.org/mattyhall/rs-tiled/doc/tiled/).


### Things missing
There are a few things missing at the moment:

  * Storing any colour - eg. transparency colours on images or background colours on maps.
  * Terrain
  * Loading files that aren't base64 encoded and compressed with zlib
  * Tile flipping
  * Image layers
  * A nice API. At the moment you can access attributes and properties, find tilesets by GID and loop through the tiles. This leaves a user of the library with a bit to do.

assets/tilesheet.png by Buch (http://blog-buch.rhcloud.com/)

Licenced under MIT
