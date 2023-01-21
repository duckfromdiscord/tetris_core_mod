use super::move_validator::{can_move_down, has_valid_position};
use super::{ActiveFigure, Block, Board, FigureType, Point, Size};

const MOVING_PERIOD: f64 = 1f64; //secs

pub enum Action {
    MoveDown,
    MoveLeft,
    MoveRight,
    Rotate,
}

pub trait Randomizer {
    fn random(&self) -> i32;
}

#[derive(PartialEq)]
pub enum GameState {
    Playing,
    GameOver,
}

pub struct Game {
    board: Board,
    score: u64,
    active: ActiveFigure,
    next: ActiveFigure,
    waiting_time: f64,
    randomizer: Box<dyn Randomizer + 'static>,
    state: GameState,
    lines: usize
}

impl Game {
    pub fn new(size: &Size, randomizer: Box<dyn Randomizer + 'static>) -> Game {
        let start_point = Game::figure_start_point(size.width);
        let active = Game::random_figure(start_point, &randomizer);
        let next = Game::random_figure(start_point, &randomizer);

        let board = Board::new(size);
        return Game {
            board,
            score: 0,
            active,
            next,
            waiting_time: 0.0,
            randomizer,
            state: GameState::Playing,
            lines: 0,
        };
    }

    fn figure_start_point(width: usize) -> Point {
        let mid_point = (width as i32).wrapping_div(2) - 2;
        return Point { x: mid_point, y: 0 };
    }

    fn random_figure(position: Point, randomizer: &Box<dyn Randomizer + 'static>) -> ActiveFigure {
        let figure = match randomizer.random() {
            0 => FigureType::I,
            1 => FigureType::J,
            2 => FigureType::L,
            3 => FigureType::O,
            4 => FigureType::S,
            5 => FigureType::T,
            _ => FigureType::Z,
        };
        return ActiveFigure::new(figure, position);
    }

    pub fn is_game_over(&self) -> bool {
        return self.state == GameState::GameOver;
    }

    // DRAWING FUNCTIONS

    pub fn draw(&self) -> Vec<Block> {
        let board = self.draw_board();
        let figure = self.draw_active_figure();
        return board.iter().chain(&figure).cloned().collect();
    }

    pub fn draw_active_figure(&self) -> Vec<Block> {
        let figure = self.active.to_cartesian();
        return figure
            .iter()
            .map(|point| Block::new(point.x, point.y, 1, 1, self.active.color()))
            .collect();
    }

    pub fn access_active_figure(&self) -> Vec<Point> {
        return self.active.to_cartesian();
    }

    pub fn draw_board(&self) -> Vec<Block> {
        let mut blocks = vec![];
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                if let Some(square) = self.board.figure_at_xy(x, y) {
                    let block = Block::new(x as i32, y as i32, 1, 1, square.color());
                    blocks.push(block);
                }
            }
        }
        return blocks;
    }


    pub fn access_board(&self) -> Vec<Point> {
        let mut points = vec![];
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                if let Some(square) = self.board.figure_at_xy(x, y) {
                    let point = Point{x: x as i32, y: y as i32}; // it does not matter what block is there
                    points.push(point);
                }
            }
        }
        return points;
    }
    // GAME UPDATE

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;
        if self.waiting_time > MOVING_PERIOD {
            self.update_game();
            self.waiting_time = 0.0;
        }
    }

    fn update_game(&mut self) {
        if self.state == GameState::GameOver {
            return;
        }
        if can_move_down(&self.active, &self.board) {
            self.move_down();
        } else {
            self.update_next_figure();
        }
    }

    fn update_next_figure(&mut self) {
        self.add_active_figure_to_board();
        let completed_lines_count = self.remove_completed_lines();
        self.add_score_for(completed_lines_count);
        self.add_new_active_figure();
        self.update_state();
    }

    fn update_state(&mut self) {
        if self.check_is_game_over() {
            self.state = GameState::GameOver;
        }
    }

    // MOVEMENT FUNCTIONS

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::MoveLeft => self.move_left(),
            Action::MoveRight => self.move_right(),
            Action::MoveDown => self.move_down(),
            Action::Rotate => self.rotate_active_figure(),
        }
    }

    fn move_left(&mut self) {
        self.update_active_with(self.active.moved_left());
    }

    fn move_right(&mut self) {
        self.update_active_with(self.active.moved_right());
    }

    fn move_down(&mut self) {
        self.update_active_with(self.active.moved_down());
    }

    fn rotate_active_figure(&mut self) {
        if let Some(rotated) = self.wall_kicked_rotated_active_figure() {
            self.update_active_with(rotated);
        }
    }

    // WALL KICK

    fn wall_kicked_rotated_active_figure(&self) -> Option<ActiveFigure> {
        return self
            .active
            .wall_kicked_rotation_tests()
            .into_iter()
            .find(|figure| has_valid_position(figure, &self.board));
    }

    // Game state mutation

    fn update_active_with(&mut self, new_active: ActiveFigure) {
        if has_valid_position(&new_active, &self.board) {
            self.active = new_active;
        }
    }

    fn add_active_figure_to_board(&mut self) {
        for point in self.active.to_cartesian() {
            self.board = self.board.replacing_figure_at_xy(
                point.x as usize,
                point.y as usize,
                Some(self.active.get_type()),
            );
        }
    }

    fn add_new_active_figure(&mut self) {
        let start_point = Game::figure_start_point(self.board.width());
        self.update_active_with(self.next.clone());
        self.next = Game::random_figure(start_point, &self.randomizer);
    }

    fn remove_completed_lines(&mut self) -> usize {
        let lines = self.lines_completed();
        self.board = self.board.removing_lines(&lines);
        self.lines += lines.len();
        return lines.len();
    }

    // Lines checks

    fn lines_completed(&self) -> Vec<usize> {
        let mut completed_lines: Vec<usize> = vec![];
        for line_number in 0..self.board.height() {
            if self.is_line_completed(line_number) {
                completed_lines.push(line_number);
            }
        }
        return completed_lines;
    }

    fn is_line_completed(&self, line_number: usize) -> bool {
        if let Some(line) = self.board.get_line(line_number) {
            return !line.contains(&None);
        }
        return false;
    }

    // Score

    fn add_score_for(&mut self, completed_lines: usize) {
        self.score += (completed_lines as u64) * 100;
    }

    fn check_is_game_over(&self) -> bool {
        return self.active.position().y == 0 && !has_valid_position(&self.active, &self.board);
    }

    pub fn get_score(&self) -> u64 {
        return self.score;
    }

    pub fn get_lines_completed(&self) -> usize {
        return self.lines;
    }
}