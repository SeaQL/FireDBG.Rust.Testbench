use firedbg_lib::fire;

pub const EMPTY: char = '_';
pub const PLAYER: char = 'X';
pub const OPPONENT: char = 'O';

pub struct TicTacToe {
    board: [[char; 3]; 3],
    round: char,
}

impl TicTacToe {
    pub fn new(board: [[char; 3]; 3]) -> Self {
        Self {
            board,
            round: PLAYER,
        }
    }

    pub fn get_empty_space(&self) -> Option<(usize, usize)> {
        for i in 0..3 {
            for j in 0..3 {
                if self.board[i][j] == EMPTY {
                    return Some((i, j));
                }
            }
        }
        return None;
    }

    pub fn get_winning_side(&self) -> Option<char> {
        for side in [PLAYER, OPPONENT] {
            for i in 0..3 {
                let mut horizontal = 0;
                let mut vertical = 0;
                for j in 0..3 {
                    if self.board[i][j] == side {
                        horizontal += 1;
                    }
                    if self.board[j][i] == side {
                        vertical += 1;
                    }
                }
                if horizontal == 3 || vertical == 3 {
                    return Some(side);
                }
            }
            let mut x1 = 0;
            let mut x2 = 0;
            for i in 0..3 {
                if self.board[i][i] == side {
                    x1 += 1;
                }
                let j = 2 - i;
                if self.board[i][j] == side {
                    x2 += 1;
                }
            }
            if x1 == 3 || x2 == 3 {
                return Some(side);
            }
        }
        return None;
    }

    pub fn solve(&mut self, round: char, depth: i32) -> i32 {
        if let Some(winner) = self.get_winning_side() {
            if winner == self.round {
                return 10;
            } else {
                return -10;
            }
        }
        if self.get_empty_space().is_none() {
            return depth;
        }
        let mut val = if self.round == round {
            i32::MIN
        } else {
            i32::MAX
        };
        for i in 0..3 {
            for j in 0..3 {
                if self.board[i][j] != EMPTY {
                    continue;
                }

                self.board[i][j] = round;
                let next_round = get_next_round(round);
                let v = self.solve(next_round, depth + 1);
                val = if self.round == round {
                    val.max(v)
                } else {
                    val.min(v)
                };
                self.board[i][j] = EMPTY;
            }
        }
        return val;
    }

    pub fn print(&self) {
        println!("[");
        for i in 0..3 {
            print!("\t[ ");
            for j in 0..3 {
                if j != 0 {
                    print!(", ");
                }
                print!("'{}'", self.board[i][j]);
            }
            println!(" ],");
        }
        println!("]");
    }

    pub fn play(&mut self) {
        loop {
            self.print();
            if let Some(winner) = self.get_winning_side() {
                println!("The winner is `{winner}`!!");
                break;
            }
            if self.get_empty_space().is_none() {
                println!("Tie! No winner.");
                break;
            }
            let mut row = 0;
            let mut col = 0;
            let mut max = i32::MIN;
            for i in 0..3 {
                for j in 0..3 {
                    if self.board[i][j] != EMPTY {
                        continue;
                    }

                    self.board[i][j] = self.round;
                    let next_round = get_next_round(self.round);
                    let val = self.solve(next_round, 0);
                    self.board[i][j] = EMPTY;

                    if val >= max {
                        row = i;
                        col = j;
                        max = val;
                        fire::dbg!((&row, &col, &max));
                    }
                }
            }
            let round = self.round;
            println!("Placed `{round}` on ({row}, {col})");
            self.board[row][col] = self.round;
            self.round = get_next_round(self.round);
        }
    }
}

fn get_next_round(round: char) -> char {
    if round == PLAYER {
        OPPONENT
    } else {
        PLAYER
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn play_1() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['X', 'O', '_'],
            ['O', '_', '_'],
            ['X', '_', '_'],
        ]);
        game.play();

        assert_eq!(
            game.board,
            [
                // Final game board
                ['X', 'O', '_'],
                ['O', 'X', '_'],
                ['X', 'O', 'X'],
            ]
        );
    }

    #[test]
    fn play_2() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', 'X'],
            ['_', '_', 'O'],
            ['_', 'O', 'X'],
        ]);
        game.play();

        assert_eq!(
            game.board,
            [
                // Final game board
                ['X', '_', 'X'],
                ['_', 'X', 'O'],
                ['O', 'O', 'X'],
            ]
        );
    }

    #[test]
    fn play_3() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['X', '_', 'X'],
            ['O', '_', '_'],
            ['X', 'O', '_'],
        ]);
        game.play();

        assert_eq!(
            game.board,
            [
                // Final game board
                ['X', '_', 'X'],
                ['O', 'X', 'O'],
                ['X', 'O', 'X'],
            ]
        );
    }

    #[test]
    fn play_4() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['_', 'O', '_'],
            ['_', '_', '_'],
            ['X', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', '_', '_'],
            ['X', 'X', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', '_', 'O'],
            ['X', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', '_', 'O'],
            ['X', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));
    }

    #[test]
    fn play_5() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', '_', '_'],
            ['X', '_', 'O'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', 'X'],
            ['_', '_', '_'],
            ['_', '_', 'O'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['X', '_', '_'],
            ['_', '_', '_'],
            ['O', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));

        let mut game = TicTacToe::new([
            // Initial game board
            ['O', '_', 'X'],
            ['_', '_', '_'],
            ['_', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), Some(PLAYER));
    }

    #[test]
    fn play_6() {
        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', 'O', '_'],
            ['X', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), None);

        let mut game = TicTacToe::new([
            // Initial game board
            ['X', '_', '_'],
            ['_', 'O', '_'],
            ['_', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), None);

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', 'X'],
            ['_', 'O', '_'],
            ['_', '_', '_'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), None);

        let mut game = TicTacToe::new([
            // Initial game board
            ['_', '_', '_'],
            ['_', 'O', '_'],
            ['_', '_', 'X'],
        ]);
        game.play();
        assert_eq!(game.get_winning_side(), None);
    }

    #[test]
    fn get_empty_space() {
        let game = TicTacToe::new([
            // Initial game board
            ['O', 'O', '_'],
            ['X', 'O', 'X'],
            ['X', 'X', 'O'],
        ]);

        assert_eq!(game.get_empty_space(), Some((0, 2)));

        let game = TicTacToe::new([
            // Initial game board
            ['O', 'O', 'O'],
            ['X', 'O', 'X'],
            ['X', 'X', 'O'],
        ]);

        assert_eq!(game.get_empty_space(), None);
    }

    #[test]
    fn get_winning_side() {
        let game = TicTacToe::new([
            // Initial game board
            ['O', 'O', '_'],
            ['X', 'O', 'X'],
            ['X', 'X', 'O'],
        ]);

        assert_eq!(game.get_winning_side(), Some(OPPONENT));

        let game = TicTacToe::new([
            // Initial game board
            ['O', 'X', 'O'],
            ['X', 'O', 'O'],
            ['O', 'X', 'X'],
        ]);

        assert_eq!(game.get_winning_side(), Some(OPPONENT));
    }
}
