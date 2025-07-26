#![allow(dead_code)]

use crate::{AxialTile, coords};

pub struct Location {
    from_center: coords::Cube,
    radius: i32,
}

impl Location {
    pub fn dist_from_center(&self) -> i32 {
        self.from_center.distance(coords::Cube::new(0, 0))
    }
    pub fn dist_from_border(&self) -> i32 {
        self.radius - self.dist_from_center()
    }
    pub fn dist_from_top(&self) -> i32 {
        self.from_center.z() + self.radius
    }
    pub fn dist_from_bottom(&self) -> i32 {
        self.radius - self.from_center.z()
    }
}

struct TileGeometry {
    width: i32,
    height: i32,
    center: coords::Cube,
}

fn geometry(radius: i32) -> TileGeometry {
    let radius_with_border = radius + 1;
    let height = 2 * radius_with_border + 1;
    let width = height;
    TileGeometry {
        width,
        height,
        center: coords::Cube {
            x: width / 2,
            y: -width / 2 - height / 2,
        },
    }
}

pub fn new<Cell: Copy>(
    radius: i32,
    border: Cell,
    cb: impl FnMut(Location) -> Cell,
) -> AxialTile<Cell> {
    let g = geometry(radius);
    let mut cb = cb;
    AxialTile::from_fn(g.width, g.height, |pos: coords::Cube| {
        let location = Location {
            from_center: pos - g.center,
            radius,
        };
        if location.dist_from_center() > radius {
            border
        } else {
            cb(location)
        }
    })
}

pub fn center(radius: i32) -> coords::Cube {
    geometry(radius).center
}

pub fn viewport(radius: i32) -> coords::Rectangle {
    let g = geometry(radius);
    coords::Rectangle {
        pos: g.center
            - coords::Cube {
                x: (radius + 1) / 2,
                y: -(radius + 1) / 2 - radius,
            },
        width: 2 * radius + 1,
        height: 2 * radius + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_all_hexes() {
        let radius = 1;
        let map = new(radius, 0, |_| 1);
        assert_eq!(map.iter_cells().sum::<i32>(), 6 + 1);
    }

    #[test]
    fn contains_all_borders() {
        let radius = 1;
        let map = new(radius, 0, |_| 1);

        let is_outside = |value| value == 0;
        let cells_adjacent_to_border = map
            .iter_valid_neighbourhoods()
            .filter(|l| !is_outside(l.center))
            .filter(|l| l.neighbours.into_iter().any(is_outside))
            .count();
        assert_eq!(cells_adjacent_to_border, 6)
    }

    #[test]
    fn validate_distances() {
        let radius = 3;

        let map = new(radius, 0, |loc| {
            assert!(loc.dist_from_center() <= 3);
            assert!(loc.dist_from_top() <= 6);
            assert!(loc.dist_from_bottom() <= 6);
            if loc.dist_from_center() == 0 {
                assert_eq!(loc.dist_from_bottom(), loc.dist_from_top());
            }
            loc.dist_from_top()
        });
        let max_dist_from_top = *map.iter_cells().max().unwrap();
        assert_eq!(max_dist_from_top, 6);

        let second_row_length = map
            .iter_cells()
            .filter(|&&dist_from_top| dist_from_top == 1)
            .count();
        assert_eq!(second_row_length, (radius + 2) as usize);
    }
}
