const EMPTY = 0;
const WHITE = 1;
const BLACK = 2;
const DIRS = [
  [-1, -1],
  [-1, 0],
  [-1, 1],
  [0, -1],
  [0, 1],
  [1, -1],
  [1, 0],
  [1, 1],
];

const boardEl = document.getElementById("board");
const blackScoreEl = document.getElementById("black-score");
const whiteScoreEl = document.getElementById("white-score");
const turnLabelEl = document.getElementById("turn-label");
const newGameBtn = document.getElementById("new-game");
const undoBtn = document.getElementById("undo");
const sideSelect = document.getElementById("side");
const depthSelect = document.getElementById("depth");

let board = newBoard();
let currentPlayer = BLACK;
let human = BLACK;
let ai = WHITE;
let gameOver = false;
let aiThinking = false;
let wasmReady = false;
let bestMoveFn = null;
let bestMoveFinalFn = null;
let wasmInitError = "";
let moveHistory = [];
let aiTimerId = null;

function newBoard() {
  const b = Array.from({ length: 8 }, () => Array(8).fill(EMPTY));
  b[3][3] = WHITE;
  b[3][4] = BLACK;
  b[4][3] = BLACK;
  b[4][4] = WHITE;
  return b;
}

function other(player) {
  return player === BLACK ? WHITE : BLACK;
}

function colorName(player) {
  return player === BLACK ? "Black" : "White";
}

function onBoard(row, col) {
  return row >= 0 && row < 8 && col >= 0 && col < 8;
}

function capturesInDirection(b, row, col, player, dy, dx) {
  const opp = other(player);
  let r = row + dy;
  let c = col + dx;
  let seenOpp = 0;

  while (onBoard(r, c)) {
    const v = b[r][c];
    if (v === opp) {
      seenOpp += 1;
      r += dy;
      c += dx;
      continue;
    }
    if (v === player) {
      return seenOpp > 0;
    }
    return false;
  }
  return false;
}

function validMove(b, row, col, player) {
  if (!onBoard(row, col) || b[row][col] !== EMPTY) return false;
  for (const [dy, dx] of DIRS) {
    if (capturesInDirection(b, row, col, player, dy, dx)) return true;
  }
  return false;
}

function getMoves(b, player) {
  const moves = [];
  for (let r = 0; r < 8; r += 1) {
    for (let c = 0; c < 8; c += 1) {
      if (validMove(b, r, c, player)) moves.push([r, c]);
    }
  }
  return moves;
}

function applyMove(b, row, col, player) {
  const next = b.map((r) => r.slice());
  next[row][col] = player;
  for (const [dy, dx] of DIRS) {
    if (!capturesInDirection(next, row, col, player, dy, dx)) continue;
    let rr = row + dy;
    let cc = col + dx;
    while (onBoard(rr, cc) && next[rr][cc] === other(player)) {
      next[rr][cc] = player;
      rr += dy;
      cc += dx;
    }
  }
  return next;
}

function countPieces(b) {
  let black = 0;
  let white = 0;
  for (let r = 0; r < 8; r += 1) {
    for (let c = 0; c < 8; c += 1) {
      if (b[r][c] === BLACK) black += 1;
      if (b[r][c] === WHITE) white += 1;
    }
  }
  return { black, white, empty: 64 - black - white };
}

function terminal(b) {
  return getMoves(b, BLACK).length === 0 && getMoves(b, WHITE).length === 0;
}

function getSearchDepth() {
  return Number(depthSelect.value) || 8;
}

function flattenBoard(b) {
  const out = new Uint8Array(64);
  let i = 0;
  for (let r = 0; r < 8; r += 1) {
    for (let c = 0; c < 8; c += 1) {
      out[i] = b[r][c];
      i += 1;
    }
  }
  return out;
}

function setStatus(text) {
  void text;
}

function formatErrorMessage(err) {
  if (!err) return "Unknown initialization error.";
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.stack || err.message;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}

function wasmUnavailableStatus() {
  if (wasmInitError) {
    return `AI stopped: Wasm failed to initialize. ${wasmInitError}`;
  }
  return "AI stopped: Wasm is not initialized.";
}

function cloneBoard(b) {
  return b.map((row) => row.slice());
}

function saveSnapshot() {
  moveHistory.push({
    board: cloneBoard(board),
    currentPlayer,
    gameOver,
  });
}

function restoreSnapshot(snapshot) {
  board = cloneBoard(snapshot.board);
  currentPlayer = snapshot.currentPlayer;
  gameOver = snapshot.gameOver;
  aiThinking = false;
}

function updateUndoButton() {
  if (undoBtn) {
    undoBtn.disabled = moveHistory.length < 2 || aiThinking;
  }
}

function refreshBoard() {
  boardEl.innerHTML = "";
  const legal = currentPlayer === human && !gameOver && !aiThinking
    ? new Set(getMoves(board, human).map(([r, c]) => `${r},${c}`))
    : new Set();

  for (let r = 0; r < 8; r += 1) {
    for (let c = 0; c < 8; c += 1) {
      const cell = document.createElement("button");
      cell.type = "button";
      cell.className = "cell";
      cell.disabled = gameOver || aiThinking || currentPlayer !== human;
      cell.dataset.row = String(r);
      cell.dataset.col = String(c);

      const piece = board[r][c];
      if (piece === BLACK || piece === WHITE) {
        const disk = document.createElement("div");
        disk.className = `piece ${piece === BLACK ? "black" : "white"}`;
        cell.appendChild(disk);
      } else if (legal.has(`${r},${c}`)) {
        const hint = document.createElement("div");
        hint.className = "hint";
        cell.appendChild(hint);
      }
      boardEl.appendChild(cell);
    }
  }

  const counts = countPieces(board);
  blackScoreEl.textContent = String(counts.black);
  whiteScoreEl.textContent = String(counts.white);
  turnLabelEl.textContent = colorName(currentPlayer);
  updateUndoButton();
}

function endGame() {
  gameOver = true;
  aiThinking = false;
  const { black, white } = countPieces(board);
  if (black > white) setStatus(`Game over. Black wins ${black}-${white}.`);
  else if (white > black) setStatus(`Game over. White wins ${white}-${black}.`);
  else setStatus(`Game over. Draw ${black}-${white}.`);
  refreshBoard();
}

function passTurnIfNeeded() {
  const moves = getMoves(board, currentPlayer);
  if (moves.length > 0) return false;

  const waiting = other(currentPlayer);
  const waitingMoves = getMoves(board, waiting);
  if (waitingMoves.length === 0) {
    endGame();
    return true;
  }

  setStatus(`${colorName(currentPlayer)} has no legal moves. Turn passes.`);
  currentPlayer = waiting;
  refreshBoard();
  return true;
}

function afterMove() {
  currentPlayer = other(currentPlayer);
  if (passTurnIfNeeded()) {
    if (!gameOver && currentPlayer === ai) {
      runAiTurn();
    }
    return;
  }

  refreshBoard();
  if (terminal(board)) {
    endGame();
    return;
  }

  if (currentPlayer === ai) runAiTurn();
  else setStatus(`${colorName(currentPlayer)} to move.`);
}

function runAiTurn() {
  if (gameOver) return;
  if (!wasmReady || !bestMoveFn || !bestMoveFinalFn) {
    aiThinking = false;
    setStatus(wasmUnavailableStatus());
    refreshBoard();
    return;
  }

  aiThinking = true;
  refreshBoard();
  setStatus("AI thinking...");

  aiTimerId = window.setTimeout(() => {
    aiTimerId = null;
    if (!wasmReady || !bestMoveFn || !bestMoveFinalFn) {
      aiThinking = false;
      setStatus(wasmUnavailableStatus());
      refreshBoard();
      return;
    }

    const moves = getMoves(board, ai);
    if (moves.length === 0) {
      aiThinking = false;
      currentPlayer = human;
      setStatus("AI has no legal moves. Your turn.");
      refreshBoard();
      if (getMoves(board, human).length === 0) endGame();
      return;
    }

    const depth = getSearchDepth();
    const { black, white } = countPieces(board);
    const currentScore = black + white;
    const useFinalSearch = currentScore + 6 + depth >= 64;
    const encoded = useFinalSearch
      ? bestMoveFinalFn(flattenBoard(board), ai, 64 - currentScore)
      : bestMoveFn(flattenBoard(board), ai, depth);

    if (encoded < 0) {
      aiThinking = false;
      setStatus(`AI stopped: Wasm returned no move (encoded=${encoded}).`);
      refreshBoard();
      return;
    }

    const row = Math.floor(encoded / 8);
    const col = encoded % 8;
    if (!moves.some(([r, c]) => r === row && c === col)) {
      aiThinking = false;
      setStatus(`AI stopped: Wasm returned illegal move ${row},${col}.`);
      refreshBoard();
      return;
    }
    const move = [row, col];

    saveSnapshot();
    board = applyMove(board, move[0], move[1], ai);
    aiThinking = false;
    setStatus(`AI played ${String.fromCharCode(65 + move[1])}${move[0] + 1}.`);
    afterMove();
  }, 30);
}

function startGame() {
  if (aiTimerId !== null) {
    window.clearTimeout(aiTimerId);
    aiTimerId = null;
  }
  board = newBoard();
  human = Number(sideSelect.value);
  ai = other(human);
  currentPlayer = BLACK;
  gameOver = false;
  aiThinking = false;
  moveHistory = [];
  setStatus("New game started.");
  refreshBoard();

  if (human !== BLACK) runAiTurn();
  else setStatus("Black to move.");
}

boardEl.addEventListener("click", (event) => {
  const target = event.target;
  const cell = target.closest(".cell");
  if (!cell || aiThinking || gameOver || currentPlayer !== human) return;

  const row = Number(cell.dataset.row);
  const col = Number(cell.dataset.col);
  if (!validMove(board, row, col, human)) return;

  saveSnapshot();
  board = applyMove(board, row, col, human);
  afterMove();
});

newGameBtn.addEventListener("click", startGame);
sideSelect.addEventListener("change", startGame);
function undoLastTurn() {
  if (aiTimerId !== null) {
    window.clearTimeout(aiTimerId);
    aiTimerId = null;
  }
  if (moveHistory.length < 2) return;

  const steps = 2;
  let snapshot = null;
  for (let i = 0; i < steps; i += 1) {
    snapshot = moveHistory.pop();
  }
  if (!snapshot) return;

  restoreSnapshot(snapshot);
  setStatus("Undid last turn.");
  refreshBoard();

  if (!gameOver && currentPlayer === ai) {
    runAiTurn();
  }
}

if (undoBtn) {
  undoBtn.addEventListener("click", undoLastTurn);
}

window.addEventListener("keydown", (event) => {
  if (event.defaultPrevented) return;
  if (event.ctrlKey || event.metaKey || event.altKey) return;
  const target = event.target;
  if (
    target &&
    (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
  ) {
    return;
  }
  if (event.key.toLowerCase() === "u") {
    event.preventDefault();
    undoLastTurn();
  }
});

try {
  const wasmMod = await import("./wasm/othello_ai_web.js");
  bestMoveFn = wasmMod.best_move;
  bestMoveFinalFn = wasmMod.best_move_final;
  await wasmMod.default();
  wasmReady = true;
  wasmInitError = "";
} catch (err) {
  console.error("Failed to initialize Rust/Wasm AI:", err);
  wasmReady = false;
  wasmInitError = formatErrorMessage(err);
}

startGame();
