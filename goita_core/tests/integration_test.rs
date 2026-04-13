use goita_core::*;

use std::collections::HashMap;

#[test]
fn default_deal_gives_each_player_eight_pieces() {
    // 標準構成の駒をフラットな配列に展開する。
    let mut deck = Vec::new();
    for (piece, count) in DEFAULT_PIECES {
        for _ in 0..count {
            deck.push(piece);
        }
    }
    assert_eq!(deck.len(), 32);

    // 北から順に時計回りで配って、各プレイヤーの手駒を作る。
    let mut hands: HashMap<BoardDirection, Hand> = HashMap::from([
        (BoardDirection::North, Hand::new()),
        (BoardDirection::East, Hand::new()),
        (BoardDirection::South, Hand::new()),
        (BoardDirection::West, Hand::new()),
    ]);
    let mut direction = BoardDirection::North;
    for piece in deck {
        hands.get_mut(&direction).unwrap().add(piece);
        direction = direction.next();
    }

    assert_eq!(hands[&BoardDirection::North].len(), 8);
    assert_eq!(hands[&BoardDirection::East].len(), 8);
    assert_eq!(hands[&BoardDirection::South].len(), 8);
    assert_eq!(hands[&BoardDirection::West].len(), 8);

    // 4人の手駒を再集計して、標準構成と一致することを確認する。
    let mut total_counts: HashMap<Piece, u8> = HashMap::new();
    for hand in hands.values() {
        for piece in hand.pieces() {
            *total_counts.entry(piece).or_insert(0) += 1;
        }
    }
    for (piece, expected) in DEFAULT_PIECES {
        assert_eq!(total_counts.get(&piece).copied().unwrap_or(0), expected);
    }
}

#[test]
fn board_facing_and_team_mapping_work_together() {
    // 方向の回転と数値変換の整合性を確認する。
    assert_eq!(BoardDirection::from(0), BoardDirection::North);
    assert_eq!(BoardDirection::from(1), BoardDirection::East);
    assert_eq!(BoardDirection::from(2), BoardDirection::South);
    assert_eq!(BoardDirection::from(3), BoardDirection::West);
    assert_eq!(BoardDirection::North.next(), BoardDirection::East);

    // 盤面へ配置した駒が向き付きで取得できることを確認する。
    let mut board = Board::new();
    board.place_pieces(
        BoardDirection::North,
        PieceWithFacing::Down(Piece::Pawn),
        Piece::Gold,
    );
    board.place_pieces(
        BoardDirection::East,
        PieceWithFacing::Up(Piece::King),
        Piece::Pawn,
    );

    let north = board.get_pieces(BoardDirection::North);
    assert_eq!(
        north,
        vec![
            PieceWithFacing::Down(Piece::Pawn),
            PieceWithFacing::Up(Piece::Gold)
        ]
    );

    let east = board.get_pieces(BoardDirection::East);
    assert_eq!(
        east,
        vec![
            PieceWithFacing::Up(Piece::King),
            PieceWithFacing::Up(Piece::Pawn)
        ]
    );

    assert_eq!(board.get_all_pieces().len(), 4);

    // プレイヤー方向からチームが正しく導出されることを確認する。
    assert_eq!(Team::from(BoardDirection::North), Team::NorthSouth);
    assert_eq!(Team::from(BoardDirection::South), Team::NorthSouth);
    assert_eq!(Team::from(BoardDirection::East), Team::EastWest);
    assert_eq!(Team::from(BoardDirection::West), Team::EastWest);
}

#[test]
fn player_action_can_update_hand_and_board() {
    // PlayerAction を使った最低限のターン進行を統合的に確認する。
    let mut hand = hand! {
        Piece::Pawn => 1,
        Piece::Gold => 1,
    };
    let mut board = Board::new();
    let action = PlayerAction::Place {
        top: Piece::Pawn,
        bottom: Piece::Gold,
    };

    match action {
        PlayerAction::Pass => {}
        PlayerAction::Place { top, bottom } => {
            hand.remove(top);
            hand.remove(bottom);
            board.place_pieces(BoardDirection::North, PieceWithFacing::Down(top), bottom);
        }
    }

    assert!(hand.is_empty());
    assert_eq!(
        board.get_pieces(BoardDirection::North),
        vec![
            PieceWithFacing::Down(Piece::Pawn),
            PieceWithFacing::Up(Piece::Gold)
        ]
    );
}
