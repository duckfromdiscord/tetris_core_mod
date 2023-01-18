mod active_figure;
mod board;
pub mod figure;
mod game;
mod move_validator;

use active_figure::ActiveFigure;
use board::Board;
pub use figure::{block, geometry, graphics, Figure, FigureType, Matrix};
use geometry::Point;
use graphics::Color;

pub use block::Block;
pub use game::{Game, Randomizer, Action};
pub use geometry::Size;
