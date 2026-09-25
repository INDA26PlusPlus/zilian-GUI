//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

const SQUARE_SIZE: f32 = 100.0;

use ggez::{
    Context, GameResult, event, glam::*, graphics::{self, Color, LineCap::Square, Mesh, Rect},
};
use std::{env, path};
use chess::*;

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
            let mut square_color = Color::from_rgb(218, 217, 181);
            let column = i % 8;
            let row = i / 8;

            if (column + row)%2 == 0 {
                square_color = Color::from_rgb(150, 77, 34);
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

        let turn = chess::Color::White;

        let game_state: String = " ".to_string();

        Ok(MainState { squares, game, clicked_square, turn, game_state})
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

        if self.game_state == "Checkmate".to_string() || self.game_state == "Stalemate".to_string() {
            let restart_game = graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                Rect::new(250.0, 450.0, 500.0, 125.0), 
                Color::from_rgba(0, 0, 0, 225)
            ).expect(" ");

            canvas.draw(&restart_game, Vec2::new(0.0, 0.0));

            let mut restart_text: String = " ".to_string();
            if self.game_state == "Checkmate".to_string() {
                if self.turn == chess::Color::White {
                    restart_text = "Checkmate! Black wins".to_string();
                }
                else {
                    restart_text = "Checkmate! White wins".to_string();
                }
            }
            else if self.game_state == "Stalemate".to_string() {
                restart_text = "Stalemate!".to_string();
            }

            canvas.draw(
                graphics::Text::new(restart_text).set_scale(40.0),
                Vec2::new(275.0, 475.0)
            );
            
            canvas.draw(
                graphics::Text::new("Press any key to restart!").set_scale(30.0),
                Vec2::new(295.0, 525.0)
            );
            
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
        
        // Finds the square that was clicked on and prints in terminal (mainly for debugging)
        let square_number = (7-row) * 8 + column;
        println!("clicked {}: c{}, r{}", &square_number, column, row);
        println!("Game_state {}", self.game_state);

        if self.clicked_square == None {
            if self.game.check_square(square_number).rank != Rank::Empty {
                self.clicked_square = Some(square_number)
            }
        } 
        else if self.game_state==" ".to_string() || self.game_state=="Check".to_string() {
            // Saves the board before the move was made
            let before_move = self.game;
            self.game.move_piece(self.clicked_square.unwrap(), square_number);
            if self.game != before_move {
                // Will only run if the move was made and was valid
                if self.turn == chess::Color::Black {
                    self.turn = chess::Color::White
                } 
                else {
                    self.turn = chess::Color::Black
                }
                if self.game.check_square(square_number).rank == Rank::Pawn && (square_number / 8 == 7 || square_number / 8 == 0) {
                    self.game.upgrade_pawn(square_number, Rank::Queen);
                }

                self.game_state = check_state(&self.game, self.turn);
            }

            self.clicked_square = None;
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> Result<(), ggez::GameError> {
        if self.game_state == "Checkmate".to_string() || self.game_state == "Stalemate".to_string()  {
            self.game.fill_board();
            self.game_state = " ".to_string();
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

fn check_state(board: &Board, color: chess::Color) -> String {

    // If in check but has legal moves -> Check
    if in_check(board, color) && has_legal_move(board, color) {
        "Check".to_string()
    }
    // In check no legal moves -> Checkmate
    else if in_check(board, color) && !has_legal_move(board, color) {
        "Checkmate".to_string()
    }
    // Not in check, no legal moves -> Stalemate
    else if !in_check(board, color) && !has_legal_move(board, color) {
        "Stalemate".to_string()
    }
    else {
        " ".to_string()
    }
}

fn in_check(board: &Board, color: chess::Color) -> bool {

    // Go through all squares
    for i in 0..64 {

        // If we find friendly king
        if board.squares[i].rank == Rank::King && board.squares[i].color == color {

            // Set king square
            let king_square = i;

            // Go through all squares
            for i in 0..64 {

                // Check if enemy can see the king
                let piece = board.squares[i];
                if piece.rank != Rank::Empty && piece.color != color {
                    let mut board_copy = *board;
                    if board_copy.fetch_movelist(i).contains(&king_square) {
                        return true;
                    }
                }
            }

            return false;
        }
    }
    return false;

}

fn has_legal_move(board: &Board, color: chess::Color) -> bool {
     // Go through all pieces
     for start in 0..64 {

        // If we find friendly piece
        if board.squares[start].color == color {
            // Save board pre-move
            let mut board_copy = *board;

            for stop in board_copy.fetch_movelist(start) {
                let mut board_post_test_move = *board;
                board_post_test_move.move_piece(start, stop);

                // We found legal move
                if board_copy != board_post_test_move && !in_check(&board_post_test_move, color){
                    return true;
                }
            }



        }
     }
     return false;
   
}