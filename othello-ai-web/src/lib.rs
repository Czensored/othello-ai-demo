use wasm_bindgen::prelude::*;

const EMPTY: u8 = 0;
const WHITE: u8 = 1;
const BLACK: u8 = 2;
const DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[wasm_bindgen]
pub fn best_move(board: &[u8], ai_color: u8, depth: u8) -> i32 {
    if board.len() != 64 || (ai_color != BLACK && ai_color != WHITE) {
        return -1;
    }
    let mut b = [0_u8; 64];
    b.copy_from_slice(board);
    let moves = get_moves(&b, ai_color);
    if moves.is_empty() {
        return -1;
    }
    let depth = depth.clamp(1, 10);
    let (_, mv) = minimax(&b, ai_color, ai_color, depth, f64::NEG_INFINITY, f64::INFINITY, 0);
    let chosen = mv.unwrap_or(moves[0]);
    (chosen.0 * 8 + chosen.1) as i32
}

#[wasm_bindgen]
pub fn best_move_final(board: &[u8], ai_color: u8, depth: u8) -> i32 {
    if board.len() != 64 || (ai_color != BLACK && ai_color != WHITE) {
        return -1;
    }
    let mut b = [0_u8; 64];
    b.copy_from_slice(board);
    let moves = get_moves(&b, ai_color);
    if moves.is_empty() {
        return -1;
    }
    let depth = depth.max(1);
    let (_, mv) = minimax_final(
        &b,
        ai_color,
        ai_color,
        depth,
        f64::NEG_INFINITY,
        f64::INFINITY,
    );
    let chosen = mv.unwrap_or(moves[0]);
    (chosen.0 * 8 + chosen.1) as i32
}

fn other(player: u8) -> u8 {
    if player == BLACK { WHITE } else { BLACK }
}

fn on_board(row: i32, col: i32) -> bool {
    (0..8).contains(&row) && (0..8).contains(&col)
}

fn idx(row: usize, col: usize) -> usize {
    row * 8 + col
}

fn captures_in_direction(board: &[u8; 64], row: usize, col: usize, player: u8, dy: i32, dx: i32) -> bool {
    let opp = other(player);
    let mut r = row as i32 + dy;
    let mut c = col as i32 + dx;
    let mut seen_opp = false;

    while on_board(r, c) {
        let v = board[idx(r as usize, c as usize)];
        if v == opp {
            seen_opp = true;
            r += dy;
            c += dx;
            continue;
        }
        if v == player {
            return seen_opp;
        }
        return false;
    }
    false
}

fn valid_move(board: &[u8; 64], row: usize, col: usize, player: u8) -> bool {
    if board[idx(row, col)] != EMPTY {
        return false;
    }
    for (dy, dx) in DIRS {
        if captures_in_direction(board, row, col, player, dy, dx) {
            return true;
        }
    }
    false
}

fn get_moves(board: &[u8; 64], player: u8) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(32);
    for row in 0..8 {
        for col in 0..8 {
            if valid_move(board, row, col, player) {
                out.push((row, col));
            }
        }
    }
    out
}

fn apply_move(board: &[u8; 64], row: usize, col: usize, player: u8) -> [u8; 64] {
    let mut next = *board;
    next[idx(row, col)] = player;
    let opp = other(player);

    for (dy, dx) in DIRS {
        if !captures_in_direction(&next, row, col, player, dy, dx) {
            continue;
        }

        let mut r = row as i32 + dy;
        let mut c = col as i32 + dx;
        while on_board(r, c) && next[idx(r as usize, c as usize)] == opp {
            next[idx(r as usize, c as usize)] = player;
            r += dy;
            c += dx;
        }
    }

    next
}

fn terminal(board: &[u8; 64]) -> bool {
    get_moves(board, BLACK).is_empty() && get_moves(board, WHITE).is_empty()
}

fn evaluate(board: &[u8; 64], ai: u8) -> f64 {
    let opp = other(ai);
    let mut my_tiles = 0_i32;
    let mut opp_tiles = 0_i32;
    let mut my_front_tiles = 0_i32;
    let mut opp_front_tiles = 0_i32;
    let x1: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
    let y1: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
    let board_values: [[i32; 8]; 8] = [
        [20, -3, 11, 8, 8, 11, -3, 20],
        [-3, -7, -4, 1, 1, -4, -7, -3],
        [11, -4, 2, 2, 2, 2, -4, 11],
        [8, 1, 2, -3, -3, 2, 1, 8],
        [8, 1, 2, -3, -3, 2, 1, 8],
        [11, -4, 2, 2, 2, 2, -4, 11],
        [-3, -7, -4, 1, 1, -4, -7, -3],
        [20, -3, 11, 8, 8, 11, -3, 20],
    ];
    let mut d = 0_i32;

    for row in 0..8 {
        for col in 0..8 {
            let v = board[idx(row, col)];
            if v == ai {
                d += board_values[row][col];
                my_tiles += 1;
            } else if v == opp {
                d -= board_values[row][col];
                opp_tiles += 1;
            }

            if v != EMPTY {
                for k in 0..8 {
                    let y = row as i32 + y1[k];
                    let x = col as i32 + x1[k];
                    if on_board(y, x) && board[idx(y as usize, x as usize)] == EMPTY {
                        if v == ai {
                            my_front_tiles += 1;
                        } else {
                            opp_front_tiles += 1;
                        }
                    }
                }
            }
        }
    }

    if my_tiles == 0 {
        return f64::NEG_INFINITY;
    }
    if opp_tiles == 0 {
        return f64::INFINITY;
    }

    let p = if my_tiles > opp_tiles {
        (100.0 * my_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else if my_tiles < opp_tiles {
        -(100.0 * opp_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else {
        0.0
    };

    let f = if my_front_tiles > opp_front_tiles {
        -(100.0 * my_front_tiles as f64) / (my_front_tiles + opp_front_tiles) as f64
    } else if my_front_tiles < opp_front_tiles {
        (100.0 * opp_front_tiles as f64) / (my_front_tiles + opp_front_tiles) as f64
    } else {
        0.0
    };

    my_tiles = 0;
    opp_tiles = 0;

    for (r, c) in [(0, 0), (0, 7), (7, 0), (7, 7)] {
        let v = board[idx(r, c)];
        if v == ai {
            my_tiles += 1;
        } else if v == opp {
            opp_tiles += 1;
        }
    }

    let c = 25.0 * (my_tiles as f64 - opp_tiles as f64);

    my_tiles = 0;
    opp_tiles = 0;

    if board[idx(0, 0)] == EMPTY {
        for (r, c2) in [(0, 1), (1, 1), (1, 0)] {
            let v = board[idx(r, c2)];
            if v == ai {
                my_tiles += 1;
            } else if v == opp {
                opp_tiles += 1;
            }
        }
    }
    if board[idx(0, 7)] == EMPTY {
        for (r, c2) in [(0, 6), (1, 6), (1, 7)] {
            let v = board[idx(r, c2)];
            if v == ai {
                my_tiles += 1;
            } else if v == opp {
                opp_tiles += 1;
            }
        }
    }
    if board[idx(7, 0)] == EMPTY {
        for (r, c2) in [(7, 1), (6, 1), (6, 0)] {
            let v = board[idx(r, c2)];
            if v == ai {
                my_tiles += 1;
            } else if v == opp {
                opp_tiles += 1;
            }
        }
    }
    if board[idx(7, 7)] == EMPTY {
        for (r, c2) in [(6, 7), (6, 6), (7, 6)] {
            let v = board[idx(r, c2)];
            if v == ai {
                my_tiles += 1;
            } else if v == opp {
                opp_tiles += 1;
            }
        }
    }

    let l = -12.5 * (my_tiles as f64 - opp_tiles as f64);

    my_tiles = get_moves(board, ai).len() as i32;
    opp_tiles = get_moves(board, opp).len() as i32;

    let m = if my_tiles > opp_tiles {
        (100.0 * my_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else if my_tiles < opp_tiles {
        (100.0 * opp_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else {
        0.0
    };

    (10.0 * p) + (801.724 * c) + (382.026 * l) + (78.922 * m) + (74.396 * f) + (10.0 * d as f64)
}

fn evaluate_final(board: &[u8; 64], ai: u8) -> f64 {
    let opp = other(ai);
    let mut my_tiles = 0_i32;
    let mut opp_tiles = 0_i32;

    for row in 0..8 {
        for col in 0..8 {
            let v = board[idx(row, col)];
            if v == ai {
                my_tiles += 1;
            } else if v == opp {
                opp_tiles += 1;
            }
        }
    }

    (my_tiles - opp_tiles) as f64
}

fn minimax(
    board: &[u8; 64],
    player: u8,
    ai: u8,
    depth: u8,
    mut alpha: f64,
    mut beta: f64,
    pass_count: u8,
) -> (f64, Option<(usize, usize)>) {
    if depth == 0 || pass_count >= 2 || terminal(board) {
        return (evaluate(board, ai), None);
    }

    let moves = get_moves(board, player);
    if moves.is_empty() {
        return minimax(board, other(player), ai, depth - 1, alpha, beta, pass_count + 1);
    }

    if player == ai {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_move = None;
        for mv in moves {
            let next = apply_move(board, mv.0, mv.1, player);
            let (score, _) = minimax(&next, other(player), ai, depth - 1, alpha, beta, 0);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            alpha = alpha.max(best_score);
            if beta <= alpha {
                break;
            }
        }
        (best_score, best_move)
    } else {
        let mut best_score = f64::INFINITY;
        let mut best_move = None;
        for mv in moves {
            let next = apply_move(board, mv.0, mv.1, player);
            let (score, _) = minimax(&next, other(player), ai, depth - 1, alpha, beta, 0);
            if score < best_score {
                best_score = score;
                best_move = Some(mv);
            }
            beta = beta.min(best_score);
            if beta <= alpha {
                break;
            }
        }
        (best_score, best_move)
    }
}

fn minimax_final(
    board: &[u8; 64],
    player: u8,
    ai: u8,
    depth: u8,
    mut alpha: f64,
    mut beta: f64,
) -> (f64, Option<(usize, usize)>) {
    let moves = get_moves(board, player);
    let player_has_no_moves = moves.is_empty();

    if depth == 0 || (player_has_no_moves && get_moves(board, other(player)).is_empty()) {
        return (evaluate_final(board, ai), None);
    }

    // Match the original implementation: passing does not consume depth.
    if player_has_no_moves {
        return minimax_final(board, other(player), ai, depth, alpha, beta);
    }

    let mut best_move = None;

    for mv in moves {
        let next = apply_move(board, mv.0, mv.1, player);
        let (value, _) = minimax_final(&next, other(player), ai, depth - 1, alpha, beta);

        if player == ai {
            if value > alpha {
                alpha = value;
                best_move = Some(mv);
            }
            if alpha >= beta {
                break;
            }
        } else {
            if value < beta {
                beta = value;
                best_move = Some(mv);
            }
            if beta <= alpha {
                break;
            }
        }
    }

    if player == ai {
        (alpha, best_move)
    } else {
        (beta, best_move)
    }
}
