use std::{collections::HashMap, fmt, io::Read, path::Path, str::FromStr};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    error::{ParseTileError, TiledError},
    layers::{ImageLayer, Layer},
    objects::ObjectGroup,
    properties::{parse_properties, Colour, Properties},
    tileset::Tileset,
    util::*,
};

/// All Tiled files will be parsed into this. Holds all the layers and tilesets
#[derive(Debug, PartialEq, Clone)]
pub struct Map {
    pub version: String,
    pub orientation: Orientation,
    /// Width of the map, in tiles
    pub width: u32,
    /// Height of the map, in tiles
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tilesets: Vec<Tileset>,
    pub layers: Vec<Layer>,
    pub image_layers: Vec<ImageLayer>,
    pub object_groups: Vec<ObjectGroup>,
    pub properties: Properties,
    pub background_colour: Option<Colour>,
    pub infinite: bool,
}

impl Map {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        mut external_file_loader: impl FnMut(&str)->Result<Vec<u8>, TiledError>,
    ) -> Result<Map, TiledError> {
        let ((c, infinite), (v, o, w, h, tw, th)) = get_attrs!(
            attrs,
            optionals: [
                ("backgroundcolor", colour, |v:String| v.parse().ok()),
                ("infinite", infinite, |v:String| Some(v == "1")),
            ],
            required: [
                ("version", version, |v| Some(v)),
                ("orientation", orientation, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("tilewidth", tile_width, |v:String| v.parse().ok()),
                ("tileheight", tile_height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("map must have a version, width and height with correct types".to_string())
        );

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();
        let mut image_layers = Vec::new();
        let mut properties = HashMap::new();
        let mut object_groups = Vec::new();
        let mut layer_index = 0;
        parse_tag!(parser, "map", {
            "tileset" => |attrs| {
                tilesets.push(Tileset::new(parser, attrs, &mut external_file_loader)?);
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(Layer::new(parser, attrs, w, layer_index, infinite.unwrap_or(false))?);
                layer_index += 1;
                Ok(())
            },
            "imagelayer" => |attrs| {
                image_layers.push(ImageLayer::new(parser, attrs, layer_index)?);
                layer_index += 1;
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                object_groups.push(ObjectGroup::new(parser, attrs, Some(layer_index))?);
                layer_index += 1;
                Ok(())
            },
        });
        Ok(Map {
            version: v,
            orientation: o,
            width: w,
            height: h,
            tile_width: tw,
            tile_height: th,
            tilesets,
            layers,
            image_layers,
            object_groups,
            properties,
            background_colour: c,
            infinite: infinite.unwrap_or(false),
        })
    }

    /// This function will return the correct Tileset given a GID.
    pub fn get_tileset_by_gid(&self, gid: u32) -> Option<&Tileset> {
        let mut maximum_gid: i32 = -1;
        let mut maximum_ts = None;
        for tileset in self.tilesets.iter() {
            if tileset.first_gid as i32 > maximum_gid && tileset.first_gid <= gid {
                maximum_gid = tileset.first_gid as i32;
                maximum_ts = Some(tileset);
            }
        }
        maximum_ts
    }

    /// Computes the rectangle on the image where the sprite is stored for the given tile ID.
    /// If the ID is not found in any tileset, or if there is no image associated with the tileset, `None` is returned.
    /// On success, returns `Some(x, y, w, h)`, where `(x, y)` is the coordinates of the top-left corner, and `(w, h)` are the width and height of the rectangle
    pub fn get_tile_rectangle_by_id(&self, id: u32) -> Option<(u32, u32, u32, u32)> {
        let tileset = self.get_tileset_by_gid(id)?;
        let img = tileset.images.get(0)?; // we suppose there is only 1 image per tileset

        let id = id - tileset.first_gid;
        let columns =
            img.width as u32 / (tileset.spacing + tileset.tile_width);

        // coordinates in tiles
        let x = id % columns;
        let y = id.div_euclid(columns);

        // coordinates in pixels
        let x = tileset.margin + (tileset.tile_width + tileset.spacing) * x;
        let y = tileset.margin + (tileset.tile_height + tileset.spacing) * y;

        let w = tileset.tile_width;
        let h = tileset.tile_height;

        Some((x, y, w, h))
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl FromStr for Orientation {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Orientation, ParseTileError> {
        match s {
            "orthogonal" => Ok(Orientation::Orthogonal),
            "isometric" => Ok(Orientation::Isometric),
            "staggered" => Ok(Orientation::Staggered),
            "hexagonal" => Ok(Orientation::Hexagonal),
            _ => Err(ParseTileError::OrientationError),
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Orthogonal => write!(f, "orthogonal"),
            Orientation::Isometric => write!(f, "isometric"),
            Orientation::Staggered => write!(f, "staggered"),
            Orientation::Hexagonal => write!(f, "hexagonal"),
        }
    }
}
