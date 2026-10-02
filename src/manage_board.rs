use chess::*;

pub fn piece_to_char (piece: Piece) -> char {

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

pub fn check_state(board: &Board, color: chess::Color) -> String {

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

pub fn board_as_string(board: &Board) -> String {

    // Blank template
    let mut string_representation = String::new();

    // Protocol order is A8, B8 ... H8, A7 ... H1, so go from rank 8 down to rank 1
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;
            let piece_char = piece_to_char(board.squares[square]);

            // Flip the case of text cause of font
            if piece_char.is_ascii_lowercase() {
                string_representation.push(piece_char.to_ascii_uppercase());
            }
            else {
                string_representation.push(piece_char.to_ascii_lowercase());
            }
        }
    }

    return string_representation;
}

pub fn square_as_string(square: usize) -> String {

    // Every 8 wraps to new row
    let file = match square % 8 {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        4 => 'E',
        5 => 'F',
        6 => 'G',
        _ => 'H'
    };
    let rank = match square / 8 {
        0 => '1',
        1 => '2',
        2 => '3',
        3 => '4',
        4 => '5',
        5 => '6',
        6 => '7',
        _ => '8'
    };
    let mut string_notation = String::new();
    string_notation.push(file);
    string_notation.push(rank);

    return string_notation;
}

pub fn string_as_square(square: &str) -> Option<usize> {

    // Takes notation and returns a square
    let column = match square.chars().nth(0).unwrap() {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        'E' => 4,
        'F' => 5,
        'G' => 6,
        'H' => 7,
        _ => return None
    };

    let row = match square.chars().nth(1).unwrap() {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => return None
    };

    return Some(row * 8 + column);
}

pub fn promotion_as_string(promotion: Option<Rank>) -> String {

    // P for no promotion
    match promotion {
        Some(Rank::Queen) => "Q".to_string(),
        Some(Rank::Rook) => "R".to_string(),
        Some(Rank::Bishop) => "B".to_string(),
        Some(Rank::Knight) => "N".to_string(),
        _ => "-".to_string()
    }
}

pub fn string_as_promotion(promotion: &str) -> Option<Rank> {

    match promotion {
        "Q" => Some(Rank::Queen),
        "R" => Some(Rank::Rook),
        "B" => Some(Rank::Bishop),
        "N" => Some(Rank::Knight),
        _ => None
    }
}

pub fn move_as_string(start: usize, stop: usize, promotion: Option<Rank>, board: &Board) -> String {

    // Turns a move into a string for network comms
    let mut message = String::new();
    message.push_str(&square_as_string(start));   // 2 start
    message.push_str(&square_as_string(stop));   // 2 stop
    message.push_str(&promotion_as_string(promotion));  // 1 promotion
    message.push_str(&board_as_string(board));         // 64 board

    return message;
}

pub fn string_as_move(message: &str) -> Option<(usize, usize, Option<Rank>, String)> {

    // 2 start + 2 stop + 1 promotion + 64 board = 69
    if message.len() != 69 || !message.is_ascii() {
        return None;
    }

    // ? returns None right away if the square isn't valid
    let start = string_as_square(&message[0..2])?;
    let stop = string_as_square(&message[2..4])?;
    let promotion = string_as_promotion(&message[4..5]);
    let board = message[5..69].to_string();

    return Some((start, stop, promotion, board));
}
