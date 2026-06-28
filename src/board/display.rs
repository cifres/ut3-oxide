use super::{Board, iterator::MiniboardStatusesIterator};
use std::fmt::{self};

#[macro_export]
macro_rules! uline {
    ($s:literal) => {
        concat!("\x1b[4m", $s, "\x1b[0m")
    };
}

impl Board {
    /// Create a mask where there are only 1s over the `18` bits for the u32 row
    /// with the remaining `14` being 0s to zero-out the row metadata.
    fn get_row_cells(&self, row: u8) -> u32 {
        let mask = (1 << 18) - 1;
        self.main_board[row as usize] & mask
    }

    #[allow(dead_code)]
    pub fn display_mb_statuses(&self) {
        for (i, mb) in MiniboardStatusesIterator::new(self.get_miniboard_statuses()).enumerate() {
            if i % 3 == 0 {
                println!();
                print!("|");
            }
            print!(" {mb} |");
        }
        println!();
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // print the board with Xs and Os
        // (board:#)
        if f.alternate() {
            writeln!(f)?;

            // Crudely colour coordinate if it has a valid cell
            // TODO: Valid moves iter -> add to [rows] [cols] separate vec
            let valid_cols = (0..9)
                .map(|c| self.valid_moves().any(|vc| vc.1 == c))
                .collect::<Vec<bool>>();
            let valid_rows = (0..9)
                .map(|c| self.valid_moves().any(|vr| vr.0 == c))
                .collect::<Vec<bool>>();

            let col_string = "    0 1 2   3 4 5   6 7 8  ";
            for c in col_string.split("") {
                if c.is_empty() || c == " " || !valid_cols[c.parse::<usize>().unwrap()] {
                    write!(f, "{c}")?;
                    continue;
                }
                write!(f, "\x1b[32m{c}\x1b[0m")?;
            }

            writeln!(f)?;

            writeln!(f, "  ┌───────┬───────┬───────┐")?;
            for row in 0..9 {
                for col in 0..9 {
                    let cell = self.get_cell(row, col);
                    let cellmbstatus = self.get_status_of(Self::move_miniboard(row, col));

                    if col == 0 {
                        if valid_rows[row as usize] {
                            write!(f, "\x1b[32m{row}\x1b[0m │ ")?;
                        } else {
                            write!(f, "{row} │ ")?;
                        }
                    }

                    // Backspace once and then start colouring because the default case is "_ "
                    // where the each character includes a space after it and thus relies on the
                    // previous character's space
                    let repr = match (cell, cellmbstatus) {
                        (0, _) if self.is_valid_move(row, col) => "\x1b[32m─ \x1b[0m",
                        (1, 1) if (row, col) == self.last_move => "\x08 \x08\x1b[1;5;44m X \x1b[0m",
                        (2, 2) if (row, col) == self.last_move => "\x08 \x08\x1b[1;5;41m O \x1b[0m",
                        (1, _) if (row, col) == self.last_move => " \x08 \x08\x1b[1;5;34mX\x1b[0m ",
                        (2, _) if (row, col) == self.last_move => " \x08 \x08\x1b[1;5;31mO\x1b[0m ",
                        (0, 1) => "\x08 \x08\x1b[44m _ \x1b[0m",
                        (0, 2) => "\x08 \x08\x1b[41m _ \x1b[0m",
                        (1, 1) => "\x08 \x08\x1b[44m X \x1b[0m",
                        (2, 1) => "\x08 \x08\x1b[44m O \x1b[0m",
                        (1, 2) => "\x08 \x08\x1b[41m X \x1b[0m",
                        (2, 2) => "\x08 \x08\x1b[41m O \x1b[0m",
                        (0, _) => "─ ",
                        (1, _) => "\x1b[34mX \x1b[0m",
                        (2, _) => "\x1b[31mO \x1b[0m",
                        _ => unreachable!(),
                    };

                    write!(f, "{repr}")?;

                    if (col + 1) % 3 == 0 {
                        write!(f, "│ ")?;
                    }
                }
                writeln!(f)?;

                if (row + 1) % 3 == 0 {
                    if row == 8 {
                        writeln!(f, "  └───────┴───────┴───────┘")?;
                    } else {
                        writeln!(f, "  ├───────┼───────┼───────┤")?;
                    }
                }
            }

            Ok(())
        } else {
            for i in 0..self.main_board.len() {
                writeln!(
                    f,
                    "[{i}]: cells[{:018b}] — meta[{:014b}]",
                    self.get_row_cells(i as u8),
                    self.main_board[i] >> 18
                )?;
            }

            Ok(())
        }
    }
}
