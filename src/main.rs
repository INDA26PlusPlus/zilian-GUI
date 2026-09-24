//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

const SQUARE_SIZE: f32 = 100.0;

use ggez::{
    Context, GameResult, event, glam::*, graphics::{self, Color, LineCap::Square, Mesh, Rect},
};
use std::{env, path};

struct MainState {
    // represents a square with a background color and a piece on it
    squares: [Mesh; 64],    // For displaying board
    game: chess::Board,     // For running game
    clicked_square: Option<usize>, // For moving pieces
    turn: chess::Color,     // For displaying who's turn it is 
    game_state: String      // For showing check, checkmate, stalemate ect.

}

impl MainState {
    // I took the colors from:
    // https://colorswall.com/palette/190559

    fn new(ctx: &mut Context) -> GameResult<MainState> {

        // For displaying pieces
        ctx.gfx.add_font(
            "chess",
            graphics::FontData::from_path(ctx, "/chess.otf")?,
        );


        // Squares and background colors
        let squares: [Mesh; 64] = std::array::from_fn(|i| {
            let mut square_color = Color::from_rgb(150, 77, 34);
            let column = i % 8;
            let row = i / 8;

            if (column + row)%2 == 0 {
                square_color = Color::from_rgb(218, 217, 181);
            }

            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(100.0, 100.0, SQUARE_SIZE, SQUARE_SIZE),
                square_color
            ).expect("")

        });
        let mut game = Board::init_board();
        game.fill_board();

        let clicked_square = None;

        Ok(MainState { squares, game, clicked_square})
    }
}

impl event::EventHandler for MainState {

    // WUpdate
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    
    // What should be drawn on update
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));


        for i in 0..64 {
            // Position of square
            let square_position = Vec2::new((i % 8) as f32 * SQUARE_SIZE, (7 - i / 8) as f32 * SQUARE_SIZE);

            // Draw square background color
            canvas.draw(&self.squares[i], square_position);

            // Highlights selected square
            if !self.clicked_square.is_none() {
                if self.clicked_square.unwrap() == i {
                    canvas.draw(
                    &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect::new(100.0, 100.0, SQUARE_SIZE, SQUARE_SIZE),
                    Color::GREEN
                    )?,
                    square_position

                    )
                }
            }

            // Draw piece on-top of square;
            let current_piece = self.game.check_square(i);
            let piece_text = pice_to_char(current_piece);

            canvas.draw(
            graphics::Text::new(piece_text)
                .set_font("chess")
                .set_scale(SQUARE_SIZE),
            Vec2::new(square_position[0] + SQUARE_SIZE, square_position[1] + SQUARE_SIZE*1.05),
            )
        }

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: ggez::winit::event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError>
    // What to do when a mouse button down even happens
    {
        let column = ((x-SQUARE_SIZE) / SQUARE_SIZE) as usize;
        let row = ((y-SQUARE_SIZE) / SQUARE_SIZE) as usize;

        if column >= 8 || row >= 8 {
            return Ok(());
        }

        let square_number = (7-row) * 8 + column;
        println!("clicked {}: c{}, r{}", &square_number, column, row);

        if self.clicked_square == None {
            if self.game.check_square(square_number).rank != Rank::Empty {
                self.clicked_square = Some(square_number)
            }
        } 
        else {
            self.game.move_piece(self.clicked_square.unwrap(), square_number);
            self.clicked_square = None;
        }

        Ok(())
    }

}

pub fn main() -> GameResult {

    // Make 64 square (0..63)
    // Check if a square was clicked on         (Start)
    // Check if another square was clicked on   (Dest)
    // Try to move from start to dest
    // If on square 0..7 or 55..63 and is pawn call the .upgrade_pawn(Rank)

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    // From where should our build pull information
    let cb = ggez::ContextBuilder::new("super_simple", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    
    // Run the events in a loop
    event::run(ctx, event_loop, state)

}

use chess::*;

// Init a full board.
fn fake_main() {

    // init a board object.
    let mut gameboard = Board::init_board();

    let (start, dest) = (1, 2);

    // fills the created board object.
    gameboard.fill_board();
    gameboard.move_piece(start, dest);
    
    // Displays board with ascii in CLI
    cli_display_board(&gameboard);
}

// Method to display board in CLI
fn cli_display_board (game: &Board) {

    let mut rank: i32 = 0;
    for square_number in 0..64 {

        let current_square_piece = game.check_square(square_number);

        // If this is the first square in the rank
        // Print current rank and a | 
        if square_number%8 == 0 {
            rank += 1;

            print!("{} | ", rank)
        }

        if square_number%8 == 7 {
            println!("{} ", pice_to_char(current_square_piece) ) 
        }
        else {
            print!("{} ", pice_to_char(current_square_piece) ) 
        }
    }
    println!("------------------");
    println!("  | a b c d e f g h");
}

fn pice_to_char (piece: Piece) -> char {

    let piece_text = match piece.rank {
        Rank::Pawn => 'p',
        Rank::Knight => 'n',
        Rank::Bishop => 'b',
        Rank::Rook => 'r',
        Rank::Queen => 'q',
        Rank::King => 'k',
        Rank::Empty => ' '
    };

    if piece.color.eq(&chess::Color::Black) {
        return piece_text.to_ascii_uppercase();
    }
    return piece_text;
}