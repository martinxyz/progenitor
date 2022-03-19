use hex2d::Direction;

pub trait Simulation<Cell: CellView> {
    fn tick(&mut self); // TODO: rename tick() --> step()
    fn get_cells_rectangle(&self) -> Vec<Cell>;
}

pub trait CellView {
    fn cell_type(&self) -> u8;
    fn energy(&self) -> Option<u8>;
    fn direction(&self) -> Option<Direction>;
}
