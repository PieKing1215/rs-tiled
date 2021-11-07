pub mod animation;
pub mod error;
pub mod image;
pub mod layers;
pub mod map;
pub mod objects;
pub mod properties;
pub mod tile;
pub mod tileset;
mod util;

use base64;

use error::*;
use image::*;
use layers::*;
use map::*;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tile::*;
use tileset::*;
use util::*;
use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::reader::{Error as XmlError, EventReader};

// TODO move these

fn default_file_loader(map_path: Option<PathBuf>)->impl FnMut(&str)->Result<Vec<u8>, TiledError> {
    move |source: &str| {
        let tileset_path = map_path.as_ref()
            .ok_or(TiledError::Other("Maps with external tilesets must know their file location.  See parse_with_path(Path).".to_string()))?
            .with_file_name(source);
        std::fs::read(&tileset_path).map_err(|e| {
            TiledError::Other(format!("Failed to read external tileset file: {:?}, error {:?}", tileset_path, e))
        })
    }
}

/// Parse a buffer hopefully containing the contents of a Tiled file and try to
/// parse it. This augments `parse` with a file location: some engines
/// (e.g. Amethyst) simply hand over a byte stream (and file location) for parsing,
/// in which case this function may be required.
pub fn parse_with_path<R: Read>(reader: R, path: &Path) -> Result<Map, TiledError> {
    parse_impl(reader, default_file_loader(Some(path.to_owned())))
}

/// Parse a buffer hopefully containing the contents of a Tiled file and try to
/// parse it. When encountered, an external tileset will be loaded with `file_loader`
/// closure giving opportunity to override default access to the file system.
pub fn parse_with_file_loader<R: Read>(
    reader: R,
    external_file_loader: impl FnMut(&str)->Result<Vec<u8>, TiledError>
) -> Result<Map, TiledError> {
    parse_impl(reader, external_file_loader)
}

/// Parse a file hopefully containing a Tiled map and try to parse it.  If the
/// file has an external tileset, the tileset file will be loaded using a path
/// relative to the map file's path.
pub fn parse_file(path: &Path) -> Result<Map, TiledError> {
    let file = File::open(path)
        .map_err(|_| TiledError::Other(format!("Map file not found: {:?}", path)))?;
    parse_impl(file, default_file_loader(Some(path.to_owned())))
}

/// Parse a buffer hopefully containing the contents of a Tiled file and try to
/// parse it.
pub fn parse<R: Read>(reader: R) -> Result<Map, TiledError> {
    parse_impl(reader, default_file_loader(None))
}

/// Parse a buffer hopefully containing the contents of a Tiled tileset.
///
/// External tilesets do not have a firstgid attribute.  That lives in the
/// map. You must pass in `first_gid`.  If you do not need to use gids for anything,
/// passing in 1 will work fine.
pub fn parse_tileset<R: Read>(reader: R, first_gid: u32) -> Result<Tileset, TiledError> {
    Tileset::new_external(reader, first_gid)
}
