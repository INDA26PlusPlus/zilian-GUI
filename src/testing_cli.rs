use chess::*;

// Init a full board.
fn main() {

    // init a board object.
    let mut gameboard = Board::init_board();

    // fills the created board object.
    gameboard.fill_board();
    
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
        Rank::Pawn => 'P',
        Rank::Knight => 'N',
        Rank::Bishop => 'B',
        Rank::Rook => 'R',
        Rank::Queen => 'Q',
        Rank::King => 'K',
        Rank::Empty => '.'
    };

    if piece.color.eq(&Color::Black) {
        return piece_text.to_ascii_lowercase();
    }
    return piece_text;
}