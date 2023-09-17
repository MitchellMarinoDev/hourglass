use std::str::FromStr;

use chess::MoveGen;

use crate::{idx_to_square_name, square_name_to_idx, squares_to_edge, Direction};

#[test]
fn test_squares_to_edge() {
    assert_eq!(squares_to_edge(0, Direction::North), 7);
    assert_eq!(squares_to_edge(0, Direction::South), 0);
    assert_eq!(squares_to_edge(0, Direction::West), 0);
    assert_eq!(squares_to_edge(0, Direction::East), 7);

    assert_eq!(squares_to_edge(3, Direction::North), 7);
    assert_eq!(squares_to_edge(3, Direction::South), 0);
    assert_eq!(squares_to_edge(3, Direction::West), 3);
    assert_eq!(squares_to_edge(3, Direction::East), 4);

    assert_eq!(squares_to_edge(27, Direction::North), 4);
    assert_eq!(squares_to_edge(27, Direction::South), 3);
    assert_eq!(squares_to_edge(27, Direction::West), 3);
    assert_eq!(squares_to_edge(27, Direction::East), 4);

    // Non-Cardinal
    assert_eq!(squares_to_edge(0, Direction::NorthWest), 0);
    assert_eq!(squares_to_edge(0, Direction::NorthEast), 7);
    assert_eq!(squares_to_edge(0, Direction::SouthWest), 0);
    assert_eq!(squares_to_edge(0, Direction::SouthEast), 0);

    assert_eq!(squares_to_edge(3, Direction::NorthWest), 3);
    assert_eq!(squares_to_edge(3, Direction::NorthEast), 4);
    assert_eq!(squares_to_edge(3, Direction::SouthWest), 0);
    assert_eq!(squares_to_edge(3, Direction::SouthEast), 0);

    assert_eq!(squares_to_edge(27, Direction::NorthWest), 3);
    assert_eq!(squares_to_edge(27, Direction::NorthEast), 4);
    assert_eq!(squares_to_edge(27, Direction::SouthWest), 3);
    assert_eq!(squares_to_edge(27, Direction::SouthEast), 3);
}

#[test]
fn test_board_moves() {
    let my_board = crate::Board::new();
    test_move_gen(my_board, 5);
}

fn test_move_gen(my_board: crate::Board, depth: usize) {
    if depth == 0 {
        return;
    }

    // Create a equivilent board from the `chess` lib.
    let chess_board = chess::Board::from_str(&my_board.get_fen())
        .expect("My chess board produced an invalid fen");
    let mut chess_move_gen = MoveGen::new_legal(&chess_board)
        .into_iter()
        .map(|c_move| (c_move, false))
        .collect::<Vec<_>>();
    let my_moves = my_board.generate_moves();

    for my_move in my_moves.iter() {
        let mut good_move = false;
        for (chess_move, accounted_for) in chess_move_gen.iter_mut() {
            let sources_match = chess_move.get_source().to_index() == my_move.from;
            let destinations_match = chess_move.get_dest().to_index() == my_move.to;
            let promotions_match = compare_pieces(my_move.promote, chess_move.get_promotion());
            let matches = sources_match && destinations_match && promotions_match;

            if matches {
                *accounted_for = true;
                good_move = true;
            }
        }

        if !good_move {
            let promoting_to = match my_move.promote {
                None => "".to_owned(),
                Some(piece) => format!(" promoting to {:?}", piece),
            };
            panic!(
                "My chess engine came up with the creative move {} {}{}.\n Here is the fen :\"{}\"",
                idx_to_square_name(my_move.from).unwrap(),
                idx_to_square_name(my_move.to).unwrap(),
                promoting_to,
                my_board.get_fen(),
            );
        }
    }

    for missed_chess_move in chess_move_gen
        .iter()
        .filter(|(_, accounted_for)| !accounted_for)
    {
        let promoting_to = match missed_chess_move.0.get_promotion() {
            None => "".to_owned(),
            Some(piece) => format!(" promoting to {:?}", piece),
        };
        panic!(
            "My chess engine missed the move {} {}{}.\n Here is the fen :\"{}\"",
            idx_to_square_name(missed_chess_move.0.get_source().to_index()).unwrap(),
            idx_to_square_name(missed_chess_move.0.get_dest().to_index()).unwrap(),
            promoting_to,
            my_board.get_fen(),
        );
    }

    // One last check.
    assert_eq!(
        my_moves.len(),
        chess_move_gen.len(),
        "Incorrect number of moves"
    );

    // Now make check every move and check those positions
    for my_move in my_moves {
        let mut board = my_board.clone();
        board
            .try_move(my_move)
            .expect("A generated move should be legal");
        test_move_gen(board, depth - 1);
    }
}

fn compare_pieces(my_piece: Option<crate::Piece>, other: Option<chess::Piece>) -> bool {
    // First compare the options
    let (my_piece, other) = match (my_piece, other) {
        (None, None) => return true,
        (Some(_), None) => return false,
        (None, Some(_)) => return false,
        (Some(my_piece), Some(other)) => (my_piece, other),
    };

    let other = other.to_string(chess::Color::White);

    match my_piece & crate::Piece::PieceType {
        crate::Piece::King => other == "K",
        crate::Piece::Queen => other == "Q",
        crate::Piece::Knight => other == "N",
        crate::Piece::Bishop => other == "B",
        crate::Piece::Rook => other == "R",
        crate::Piece::Pawn => other == "P",
        _ => panic!("Invalid piece type"),
    }
}

#[test]
fn test_idx_to_square_name() {
    let square_names = ["a1", "b1", "c3", "h8"];
    let square_idxs = [0, 1, 18, 63];

    for (square_name, square_idx) in square_names.iter().zip(square_idxs) {
        assert_eq!(*square_name, idx_to_square_name(square_idx).unwrap());
        assert_eq!(square_idx, square_name_to_idx(square_name).unwrap());
    }
}
