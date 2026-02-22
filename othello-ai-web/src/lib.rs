use wasm_bindgen::prelude::*;

const WHITE: i32 = 1;
const BLACK: i32 = 2;

fn check_directions(
    board: &[[i32; 8]; 8],
    row: usize,
    col: usize,
    player: i32,
    dy: i32,
    dx: i32,
) -> bool {
    let mut r = row as i32 + dy;
    let mut c = col as i32 + dx;

    // First step must be opponent.
    if r < 0 || r >= 8 || c < 0 || c >= 8 {
        return false;
    }

    let mut piece = board[r as usize][c as usize];
    if piece == 0 || piece == player {
        return false;
    }

    // Move further in the same direction.
    for _ in 0..6 {
        r += dy;
        c += dx;

        if r < 0 || r >= 8 || c < 0 || c >= 8 {
            return false;
        }

        piece = board[r as usize][c as usize];
        if piece == 0 {
            return false;
        }
        if piece == player {
            return true;
        }
    }

    false
}

fn valid_move(board: &[[i32; 8]; 8], row: usize, col: usize, player: i32) -> bool {
    if row >= 8 || col >= 8 || board[row][col] != 0 {
        return false;
    }

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

    for &(dy, dx) in DIRS.iter() {
        if check_directions(board, row, col, player, dy, dx) {
            return true;
        }
    }

    false
}

fn all_moves(board: &[[i32; 8]; 8], player: i32) -> Vec<(usize, usize)> {
    let mut moves = Vec::with_capacity(60);

    for row in 0..8 {
        for col in 0..8 {
            if valid_move(board, row, col, player) {
                moves.push((row, col));
            }
        }
    }

    moves
}

fn evaluate_board(board: &[[i32; 8]; 8], computer: i32) -> f64 {
    let opp = 3 - computer;

    let mut my_tiles = 0;
    let mut opp_tiles = 0;
    let mut my_front_tiles = 0;
    let mut opp_front_tiles = 0;

    let x1: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
    let y1: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];

    let board_values = [
        [20, -3, 11, 8, 8, 11, -3, 20],
        [-3, -7, -4, 1, 1, -4, -7, -3],
        [11, -4, 2, 2, 2, 2, -4, 11],
        [8, 1, 2, -3, -3, 2, 1, 8],
        [8, 1, 2, -3, -3, 2, 1, 8],
        [11, -4, 2, 2, 2, 2, -4, 11],
        [-3, -7, -4, 1, 1, -4, -7, -3],
        [20, -3, 11, 8, 8, 11, -3, 20],
    ];

    let mut d = 0;
    for row in 0..8 {
        for col in 0..8 {
            if board[row][col] == computer {
                d += board_values[row][col];
                my_tiles += 1;
            } else if board[row][col] == opp {
                d -= board_values[row][col];
                opp_tiles += 1;
            }

            if board[row][col] != 0 {
                for k in 0..8 {
                    let y = row as i32 + y1[k];
                    let x = col as i32 + x1[k];
                    if y >= 0 && y < 8 && x >= 0 && x < 8 && board[y as usize][x as usize] == 0 {
                        if board[row][col] == computer {
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

    if board[0][0] == computer {
        my_tiles += 1;
    } else if board[0][0] == opp {
        opp_tiles += 1;
    }

    if board[0][7] == computer {
        my_tiles += 1;
    } else if board[0][7] == opp {
        opp_tiles += 1;
    }

    if board[7][0] == computer {
        my_tiles += 1;
    } else if board[7][0] == opp {
        opp_tiles += 1;
    }

    if board[7][7] == computer {
        my_tiles += 1;
    } else if board[7][7] == opp {
        opp_tiles += 1;
    }

    let c = 25.0 * (my_tiles as f64 - opp_tiles as f64);

    my_tiles = 0;
    opp_tiles = 0;

    if board[0][0] == 0 {
        if board[0][1] == computer {
            my_tiles += 1;
        } else if board[0][1] == opp {
            opp_tiles += 1;
        }
        if board[1][1] == computer {
            my_tiles += 1;
        } else if board[1][1] == opp {
            opp_tiles += 1;
        }
        if board[1][0] == computer {
            my_tiles += 1;
        } else if board[1][0] == opp {
            opp_tiles += 1;
        }
    }

    if board[0][7] == 0 {
        if board[0][6] == computer {
            my_tiles += 1;
        } else if board[0][6] == opp {
            opp_tiles += 1;
        }
        if board[1][6] == computer {
            my_tiles += 1;
        } else if board[1][6] == opp {
            opp_tiles += 1;
        }
        if board[1][7] == computer {
            my_tiles += 1;
        } else if board[1][7] == opp {
            opp_tiles += 1;
        }
    }

    if board[7][0] == 0 {
        if board[7][1] == computer {
            my_tiles += 1;
        } else if board[7][1] == opp {
            opp_tiles += 1;
        }
        if board[6][1] == computer {
            my_tiles += 1;
        } else if board[6][1] == opp {
            opp_tiles += 1;
        }
        if board[6][0] == computer {
            my_tiles += 1;
        } else if board[6][0] == opp {
            opp_tiles += 1;
        }
    }

    if board[7][7] == 0 {
        if board[6][7] == computer {
            my_tiles += 1;
        } else if board[6][7] == opp {
            opp_tiles += 1;
        }
        if board[6][6] == computer {
            my_tiles += 1;
        } else if board[6][6] == opp {
            opp_tiles += 1;
        }
        if board[7][6] == computer {
            my_tiles += 1;
        } else if board[7][6] == opp {
            opp_tiles += 1;
        }
    }

    let l = -12.5 * (my_tiles as f64 - opp_tiles as f64);

    my_tiles = all_moves(board, computer).len() as i32;
    opp_tiles = all_moves(board, opp).len() as i32;

    let m = if my_tiles > opp_tiles {
        (100.0 * my_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else if my_tiles < opp_tiles {
        (100.0 * opp_tiles as f64) / (my_tiles + opp_tiles) as f64
    } else {
        0.0
    };

    (10.0 * p) + (801.724 * c) + (382.026 * l) + (78.922 * m) + (74.396 * f) + (10.0 * d as f64)
}

fn apply_move(board: &[[i32; 8]; 8], row: usize, col: usize, player: i32) -> [[i32; 8]; 8] {
    let mut new_board = *board;
    new_board[row][col] = player;

    for (y, x) in [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ]
    .iter()
    {
        if check_directions(board, row, col, player, *y, *x) {
            let mut new_y = *y;
            let mut new_x = *x;

            let mut next_row = (row as i32 + new_y) as usize;
            let mut next_col = (col as i32 + new_x) as usize;

            while new_board[next_row][next_col] != player {
                new_board[next_row][next_col] = player;

                new_y += y;
                new_x += x;

                next_row = (row as i32 + new_y) as usize;
                next_col = (col as i32 + new_x) as usize;
            }
        }
    }

    new_board
}

fn rust_alpha_beta_helper(
    board: &[[i32; 8]; 8],
    depth: i32,
    alpha: f64,
    beta: f64,
    player: i32,
    computer: i32,
) -> (f64, Option<(usize, usize)>) {
    let moves = all_moves(board, player);
    let player_has_no_moves = moves.is_empty();

    if depth == 0 || (player_has_no_moves && all_moves(board, 3 - player).is_empty()) {
        return (evaluate_board(board, computer), None);
    }
    if player_has_no_moves {
        return rust_alpha_beta_helper(board, depth, alpha, beta, 3 - player, computer);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut best_move = None;

    for (row, col) in moves {
        let new_board = apply_move(board, row, col, player);
        let (value, _) =
            rust_alpha_beta_helper(&new_board, depth - 1, alpha, beta, 3 - player, computer);

        if player == computer {
            if value > alpha {
                alpha = value;
                best_move = Some((row, col));
            }
            if alpha >= beta {
                break;
            }
        } else {
            if value < beta {
                beta = value;
                best_move = Some((row, col));
            }
            if beta <= alpha {
                break;
            }
        }
    }

    (if player == computer { alpha } else { beta }, best_move)
}

fn evaluate_board_final_moves(board: &[[i32; 8]; 8], computer: i32) -> f64 {
    let opp = 3 - computer;

    let mut my_tiles = 0;
    let mut opp_tiles = 0;

    for row in 0..8 {
        for col in 0..8 {
            if board[row][col] == computer {
                my_tiles += 1;
            } else if board[row][col] == opp {
                opp_tiles += 1;
            }
        }
    }
    my_tiles as f64 - opp_tiles as f64
}

fn rust_alpha_beta_final_moves_helper(
    board: &[[i32; 8]; 8],
    depth: i32,
    alpha: f64,
    beta: f64,
    player: i32,
    computer: i32,
) -> (f64, Option<(usize, usize)>) {
    let moves = all_moves(board, player);
    let player_has_no_moves = moves.is_empty();

    if depth == 0 || (player_has_no_moves && all_moves(board, 3 - player).is_empty()) {
        return (evaluate_board_final_moves(board, computer), None);
    }

    if player_has_no_moves {
        return rust_alpha_beta_final_moves_helper(board, depth, alpha, beta, 3 - player, computer);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut best_move = None;

    for (row, col) in moves {
        let new_board = apply_move(board, row, col, player);
        let (value, _) = rust_alpha_beta_final_moves_helper(
            &new_board,
            depth - 1,
            alpha,
            beta,
            3 - player,
            computer,
        );

        if player == computer {
            if value > alpha {
                alpha = value;
                best_move = Some((row, col));
            }
            if alpha >= beta {
                break;
            }
        } else {
            if value < beta {
                beta = value;
                best_move = Some((row, col));
            }
            if beta <= alpha {
                break;
            }
        }
    }

    (if player == computer { alpha } else { beta }, best_move)
}

fn board_from_flat(board: &[u8]) -> Option<[[i32; 8]; 8]> {
    if board.len() != 64 {
        return None;
    }

    let mut board_array = [[0_i32; 8]; 8];
    for row in 0..8 {
        for col in 0..8 {
            board_array[row][col] = board[row * 8 + col] as i32;
        }
    }
    Some(board_array)
}

#[wasm_bindgen]
pub fn best_move(board: &[u8], ai_color: u8, depth: u8) -> i32 {
    if ai_color != WHITE as u8 && ai_color != BLACK as u8 {
        return -1;
    }

    let Some(board_array) = board_from_flat(board) else {
        return -1;
    };

    let (_, mv) = rust_alpha_beta_helper(
        &board_array,
        depth as i32,
        f64::NEG_INFINITY,
        f64::INFINITY,
        ai_color as i32,
        ai_color as i32,
    );

    match mv {
        Some((row, col)) => (row * 8 + col) as i32,
        None => -1,
    }
}

#[wasm_bindgen]
pub fn best_move_final(board: &[u8], ai_color: u8, depth: u8) -> i32 {
    if ai_color != WHITE as u8 && ai_color != BLACK as u8 {
        return -1;
    }

    let Some(board_array) = board_from_flat(board) else {
        return -1;
    };

    let (_, mv) = rust_alpha_beta_final_moves_helper(
        &board_array,
        depth as i32,
        f64::NEG_INFINITY,
        f64::INFINITY,
        ai_color as i32,
        ai_color as i32,
    );

    match mv {
        Some((row, col)) => (row * 8 + col) as i32,
        None => -1,
    }
}
