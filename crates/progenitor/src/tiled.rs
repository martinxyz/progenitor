use hex2d::Coordinate;
use serde::{Deserialize, Serialize};

use crate::{coords, AxialTile};

#[derive(Debug, Deserialize)]
struct TiledMap {
    orientation: String,
    staggeraxis: String,
    staggerindex: String,
    layers: Vec<TiledTileLayer>,
}

#[derive(Debug, Deserialize)]
struct TiledTileLayer {
    data: Vec<i32>,
    height: i32,
    width: i32,
}

impl TiledMap {
    fn to_axial_tile<T: Copy>(&self, fill: T, convert: impl Fn(i32) -> T) -> AxialTile<T> {
        assert_eq!(self.orientation, "hexagonal");
        assert_eq!(self.staggeraxis, "y");
        assert_eq!(self.staggerindex, "odd");
        let layer = &self.layers[0];
        assert_eq!((layer.height * layer.width) as usize, layer.data.len());

        let pad = layer.height / 2;
        let mut map = AxialTile::new(layer.width + pad, layer.height, fill);

        for row in 0..layer.width {
            for col in 0..layer.height {
                let pos: coords::Cube = coords::Offset { col: col + pad, row }.into();
                if map.valid(pos) {
                    map.set_cell(
                        pos,
                        convert(layer.data[(row * layer.height + col) as usize]),
                    );
                }
            }
        }
        map
    }
}

pub fn load_axial_tile_from_json<T: Copy>(
    json: &str,
    fill: T,
    convert: impl Fn(i32) -> T,
) -> AxialTile<T> {
    let map: TiledMap = serde_json::from_str(json).unwrap();
    map.to_axial_tile(fill, convert)
}
