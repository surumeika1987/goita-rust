use goita::*;

mod common;

// 北家の手牌に歩が5枚含まれる配牌を作成し、deal_hands 実行時に
// FivePawn 役イベント（対象プレイヤー: North）が正しく発生することを検証するテスト。
#[test]
fn test_is_five_pawn() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Gold => 2,
                Piece::Silver => 1,
                Piece::Pawn => 5,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Silver => 3,
                Piece::Knight => 4,
                Piece::Pawn => 1,
            },
            hand! {
                Piece::Lance => 4,
                Piece::Pawn => 4,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(
        deal_event,
        DealEvent::FivePawn {
            player: BoardDirection::North
        }
    );
}

// 両チームに「歩5枚」の配牌者がいるケースを作り、deal_hands の結果として
// DealEvent::FivePawnBothTeams が返ることを確認するテスト。
#[test]
fn test_is_five_pawn_both_team() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Gold => 2,
                Piece::Silver => 1,
                Piece::Pawn => 5,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Silver => 3,
                Piece::Knight => 4,
                Piece::Lance => 1,
            },
            hand! {
                Piece::Lance => 3,
                Piece::Pawn => 5,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(deal_event, DealEvent::FivePawnBothTeams,);
}

// 北南チーム内で2人が歩を5枚ずつ持つ配牌を与え、配牌時に
// DealEvent::FivePawnSameTeam { team: Team::NorthSouth } が発生することを検証するテスト。
#[test]
fn test_is_five_pawn_same_team() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Gold => 2,
                Piece::Silver => 1,
                Piece::Pawn => 5,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Lance => 3,
                Piece::Pawn => 5,
            },
            hand! {
                Piece::Silver => 3,
                Piece::Knight => 4,
                Piece::Lance => 1,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(
        deal_event,
        DealEvent::FivePawnSameTeam {
            team: Team::NorthSouth
        }
    );
}

// 北家の手札が「歩6枚（六歩）」役になることを検証するテスト。
// GoitaRound を北家開始で初期化し、4人分の手札を固定で配る。
// 北家の手札は Gold1, Silver1, Pawn6 なので、期待される役は SixPawn（30点）。
// deal_hands の結果として返る DealEvent が、
// player: North / rank: HandRank::SixPawn { score: 30 } と一致することを確認している。
#[test]
fn test_is_six_pawn() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Gold => 1,
                Piece::Silver => 1,
                Piece::Pawn => 6,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Gold => 1,
                Piece::Silver => 3,
                Piece::Knight => 4,
            },
            hand! {
                Piece::Lance => 4,
                Piece::Pawn => 4,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(
        deal_event,
        DealEvent::HandRank {
            player: BoardDirection::North,
            rank: HandRank::SixPawn { score: 30 }
        }
    );
}

// 北家に「歩7枚（SevenPawn）」の手牌を配ったとき、
// deal_hands が HandRank::SevenPawn { score: 60 } を返すことを検証するテスト。
// 他の3人には通常の手牌を配り、役判定対象が北家のみであることを明確にしている。
#[test]
fn test_is_seven_pawn() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Gold => 1,
                Piece::Pawn => 7,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Gold => 1,
                Piece::Silver => 3,
                Piece::Knight => 4,
            },
            hand! {
                Piece::Silver => 1,
                Piece::Lance => 4,
                Piece::Pawn => 3,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(
        deal_event,
        DealEvent::HandRank {
            player: BoardDirection::North,
            rank: HandRank::SevenPawn { score: 60 }
        }
    );
}

// 北家に「歩8枚（EightPawn）」の手牌を配ったとき、
// deal_hands が HandRank::EightPawn を返すことを検証するテスト。
// 他の3人には通常構成の手牌を配り、役判定が北家の配牌に基づくことを確認している。
// println! は DealEvent の内容をデバッグ出力するために置かれている。
#[test]
fn test_is_eight_pawn() {
    let mut round = GoitaRound::new(BoardDirection::North);

    let deal_event = round
        .deal_hands(vec![
            hand! {
                Piece::Pawn => 8,
            },
            hand! {
                Piece::King => 2,
                Piece::Rook => 2,
                Piece::Bishop => 2,
                Piece::Gold => 2,
            },
            hand! {
                Piece::Gold => 1,
                Piece::Silver => 3,
                Piece::Knight => 4,
            },
            hand! {
                Piece::Gold => 1,
                Piece::Silver => 1,
                Piece::Lance => 4,
                Piece::Pawn => 2,
            },
        ])
        .unwrap();

    println!("Deal event: {:?}", deal_event);
    assert_eq!(
        deal_event,
        DealEvent::HandRank {
            player: BoardDirection::North,
            rank: HandRank::EightPawn
        }
    );
}
