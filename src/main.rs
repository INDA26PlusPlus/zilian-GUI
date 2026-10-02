//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

const SQUARE_SIZE: f32 = 100.0;

use ggez::{
    Context, GameResult, event, glam::*, graphics::{self, Color, Mesh, Rect},
};
use std::{env, path, io};
use chess::*;
mod network;
mod manage_board;

struct MainState {
    // Game display parameters
    squares: [Mesh; 64],    // For displaying board
    game: chess::Board,     // For running game
    clicked_square: Option<usize>, // For moving pieces
    turn: chess::Color,     // For displaying who's turn it is 
    game_state: String,      // For showing check, checkmate, stalemate ect.
    // Network parameters
    network: Option<network::Network>, // Network connection (none if local)
    sending_game: Option<chess::Board> // Copy of board to make moves and send over network
}

impl MainState {
    // I took the colors from:
    // https://colorswall.com/palette/190559

    fn new(ctx: &mut Context, network: Option<network::Network>) -> GameResult<MainState> {

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
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                square_color
            ).expect("")

        });
        let mut game = Board::init_board();
        game.fill_board();

        let clicked_square = None;

        let turn = chess::Color::White;

        let game_state: String = " ".to_string();

        let sending_game = None;

        Ok(MainState { squares, game, clicked_square, turn, game_state, network, sending_game})
    }

    // Sends move to opponent before its made given a set of conditions
    fn send_move(&mut self, start: usize, stop: usize, promotion: Option<Rank>, board: &chess::Board) {
        let message = manage_board::move_as_string(start, stop, promotion, board);

        // If we have a valid network
        if self.network.is_some() {
            match self.network.as_mut().unwrap().send_message(&message) {
                Ok(()) => (),
                Err(e) => println!("{:?}", e)
            }
        }
    }

    // Sends a reply/confirmation to the opponent
    fn send_reply(&mut self, reply: &str) {
        if self.network.is_some() {
            match self.network.as_mut().unwrap().send_message(reply) {
                Ok(()) => (),
                Err(e) => println!("{:?}", e)
            }
        }
    }

    // Replies to opponents move
    fn receive_move(&mut self, message: &str) {

        // Turn the message into a move, if failed, reject
        let (start, stop, promotion, board) = match manage_board::string_as_move(message) {
            Some(received_move) => received_move,
            None => {
                self.send_reply("REJECT");
                return;
            }
        };

        // Reject if moving out of turn
        let local_color = self.network.as_ref().unwrap().self_color();
        if self.turn == local_color || self.game.squares[start].color != self.turn {
            self.send_reply("REJECT");
            return;
        }

        // Copy board to check
        let mut after_move = self.game;
        after_move.move_piece(start, stop);

        // Invalid move
        if after_move == self.game {
            self.send_reply("REJECT");
            return;
        }

        // Promotes pawn
        if promotion.is_some() {
            after_move.upgrade_pawn(stop, promotion.unwrap());
        }

        // Compare our "after_move" temp board
        // With "board" received from opponent
        if manage_board::board_as_string(&after_move) != board {
            self.send_reply("REJECT");
            return;
        }

        // Move is valid, actually do the move and change the turn
        self.game = after_move;

        if self.turn == chess::Color::Black {
            self.turn = chess::Color::White;
        }
        else {
            self.turn = chess::Color::Black;
        }

        self.game_state = manage_board::check_state(&self.game, self.turn);

        // Give bakc the board-state or OK if nothing
        if self.game_state == "Checkmate".to_string() {
            self.send_reply("CHECKMATE");
        }
        else if self.game_state == "Stalemate".to_string() {
            self.send_reply("STALEMATE");
        }
        else {
            self.send_reply("OK");
        }
    }
}

impl event::EventHandler for MainState {

    // WUpdate
    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        if self.network.is_some() {
            let message = match self.network.as_mut().unwrap().receive_message() {
                Ok(Some(message)) => message,
                Ok(None) => return Ok(()),
                Err(e) => {
                    println!("{:?}", e);
                    self.network = None;
                    return Ok(());
                },
            };

            match message.as_str() {
                "OK" | "CHECKMATE" | "STALEMATE" => {
                    // The move was received and approved
                    if self.sending_game.is_some() {
                        self.game = self.sending_game.unwrap();

                        if self.turn == chess::Color::Black {
                            self.turn = chess::Color::White;
                        }
                        else {
                            self.turn = chess::Color::Black;
                        }

                        self.game_state = manage_board::check_state(&self.game, self.turn);

                        // Opponent says the game is over
                        if message == "CHECKMATE" {
                            self.game_state = "Checkmate".to_string();
                        }
                        else if message == "STALEMATE" {
                            self.game_state = "Stalemate".to_string();
                        }
                    }

                    // No longer waiting for a reply
                    self.sending_game = None;
                }

                "REJECT" => {
                    // The move was received and rejected
                    self.sending_game = None;
                }

                _ => {
                    // Not our turn
                    self.receive_move(&message);
                }
            };
            
        }
        Ok(())
    }
    
    // What should be drawn on update
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // Draws squares with pieces on them
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
                    Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                    Color::from_rgba(0, 255, 0, 150)
                    )?,
                    square_position

                    )
                }
            }

            // Highlights / marks squares you can move to
            if !self.clicked_square.is_none() {

                //println!("{}", self.clicked_square.unwrap());

                for x in self.game.fetch_movelist(self.clicked_square.unwrap()) {
                    canvas.draw(
                    &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect::new(0.0, 0.0, SQUARE_SIZE/3.0, SQUARE_SIZE/3.0),
                    Color::from_rgba(0, 255, 0, 5)
                    )?,

                    Vec2::new(((x % 8) as f32 * SQUARE_SIZE)+SQUARE_SIZE/3.0, ((7 - x / 8) as f32 * SQUARE_SIZE)+SQUARE_SIZE/3.0)
                    );
                }
            }
            

            // Draw piece on-top of square;
            let current_piece = self.game.check_square(i);
            let piece_text = manage_board::piece_to_char(current_piece);

            canvas.draw(
            graphics::Text::new(piece_text)
                .set_font("chess")
                .set_scale(SQUARE_SIZE),
            Vec2::new(square_position[0], square_position[1] + SQUARE_SIZE*0.05),
            )
        }

        if self.game_state == "Checkmate".to_string() || self.game_state == "Stalemate".to_string() {
            let restart_game = graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                Rect::new(SQUARE_SIZE*1.5, SQUARE_SIZE*3.5, SQUARE_SIZE*5.0, SQUARE_SIZE*1.25), 
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
                graphics::Text::new(restart_text).set_scale(0.4*SQUARE_SIZE),
                Vec2::new(SQUARE_SIZE*1.75, SQUARE_SIZE*3.75)
            );
            
            canvas.draw(
                graphics::Text::new("Press any key to restart!").set_scale(0.3*SQUARE_SIZE),
                Vec2::new(SQUARE_SIZE*1.95, SQUARE_SIZE*4.25)
            );
            
        }

        let mut turn_text = 
        if self.turn == chess::Color::Black {"Black's turn".to_string()} 
        else {"White's turn".to_string()};

        if self.network.is_some() {
            if self.network.as_ref().unwrap().is_host() {
                turn_text.push_str(": HOST");
            }
            else {
                turn_text.push_str(": CLIENT");
            }
        }
        else {
            turn_text.push_str(": LOCAL");
        }

        canvas.draw(
                
                graphics::Text::new(turn_text).set_scale(0.3*SQUARE_SIZE),
                Vec2::new(0.0, SQUARE_SIZE*8.0)
            );
    

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::winit::event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError>
    // What to do when a mouse button down even happens
    {
        let column = ((x) / SQUARE_SIZE) as usize;
        let row = ((y) / SQUARE_SIZE) as usize;

        if column >= 8 || row >= 8 {
            return Ok(());
        }

        // Can't make moves in network game when not your "network turn" 
        if let Some(network) = &self.network {
            if network.self_color() != self.turn {
                return Ok(());
            }
        }

        // Can't make moves when we're still waiting for TCP reply
        if self.sending_game.is_some() {
            return Ok(());
        }

        // Finds the square that was clicked on and prints in terminal (mainly for debugging)
        let square_number = (7-row) * 8 + column;
        println!("clicked {}: c{}, r{}", &square_number, column, row);
        println!("Game_state {}", self.game_state);

        // If no square was clicked before
        if self.clicked_square == None {
            if self.game.check_square(square_number).color == self.turn {
                self.clicked_square = Some(square_number)
            }
        } 
        // Network game
        else if self.network.is_some() && (self.game_state==" ".to_string() || self.game_state=="Check".to_string()) {
            // Makes a copy of the board to make moves on without effecting the real board
            let mut after_move = self.game;
            let start = self.clicked_square.unwrap();

            after_move.move_piece(start, square_number);
            if after_move != self.game {
                // Will only run if the move was made and was valid according to local
                let mut promotion = None;
                if after_move.check_square(square_number).rank == Rank::Pawn && (square_number / 8 == 7 || square_number / 8 == 0) {
                    after_move.upgrade_pawn(square_number, Rank::Queen);
                    promotion = Some(Rank::Queen);
                }

                // Send the move out
                self.send_move(start, square_number, promotion, &after_move);
                self.sending_game = Some(after_move);
            }

            self.clicked_square = None;
        }
        // Local game
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

                self.game_state = manage_board::check_state(&self.game, self.turn);
            }

            self.clicked_square = None;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _input: ggez::input::keyboard::KeyInput, _repeated: bool) -> Result<(), ggez::GameError> {
        if self.game_state == "Checkmate".to_string() || self.game_state == "Stalemate".to_string()  {
            self.game.fill_board();
            self.game_state = " ".to_string();
        }
        Ok(())
    }



}

pub fn main() -> GameResult {

    // HERE WE DETERMINE WHAT TYPE OF GAME THIS IS
    // THAT IS ONE OF:
    // host, client, local (none)
    // Then we setup the network depending on that.

    println!("How would you like to play?");
    println!("1. Host");
    println!("2. Client");
    println!("3. Local");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Invalid input.");
    
    let network = match choice.trim() {
        // Setup local as host
        "1" => {
            println!("Searching for client to join on PORT: {}...", network::PORT);

            match network::Network::establish_as_host() {
                Ok(network) => Some(network),
                Err(error) => {
                    println!("Search failed: {:?}", error);
                    return Ok(());
                }
            }
        }
        // Setup local as client
        "2" => {
            println!("Enter address to join (leave empty for 127.0.0.1:6767):");
            let mut addr = String::new();
            io::stdin().read_line(&mut addr).expect("Invalid input.");
            if addr.trim().is_empty() {
                 addr = "127.0.0.1:6767".to_string();
            } 
            else { 
                addr = addr.trim().to_string();
            };

            match network::Network::establish_as_client(&addr, chess::Color::White) {
                Ok(network) => Some(network),
                Err(e) => {
                    println!("Failed to join {}: {:?}", addr, e);
                    return Ok(());
                }
            }
        }
        // Setup local as "local"
        _ => None,
    };

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
    let cb = ggez::ContextBuilder::new("super_simple", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(
            ggez::conf::WindowMode::default().dimensions(SQUARE_SIZE*8.0, SQUARE_SIZE*9.0)
        );
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx, network)?;
    
    // Run the events in a loop
    event::run(ctx, event_loop, state)

}
