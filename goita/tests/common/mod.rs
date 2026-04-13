use goita::{Error, GoitaGame, PlayerAction};

// 駒と枚数の組み合わせから手札(Vec)を生成する補助マクロ。
// `piece => count` を可変長で受け取り、各駒を指定枚数だけ `v` に追加して返す。
#[macro_export]
macro_rules! hand {
    ($($piece:expr => $count:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $(
            for _ in 0..$count {
                v.push($piece);
            }
        )*
        v
    }};
}

// 最後に駒をおいたプレイヤーまでパスするヘルパー関数
#[allow(dead_code)]
pub fn pass_until_last_placed_player(game: &mut GoitaGame) {
    for _ in 0..3 {
        // Normal pass
        let player = game.current_turn_plyer().unwrap();
        let action_result = game.play_turn(player, PlayerAction::Pass);
        if let Err(Error::InvalidPass) = action_result {
            // If the pass is invalid, it means the last placed player is the same as the current player,
            // so we break the loop to avoid infinite loop.
            break;
        }
        println!("{:?} passes: {:?}", player, action_result);
    }
}
