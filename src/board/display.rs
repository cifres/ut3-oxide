use super::{Board, iterator::MiniboardStatusesIterator};
use std::fmt::{self};

impl Board {
    /// Create a mask where there are only 1s over the `18` bits for the u32 row
    /// with the remaining `14` being 0s to zero-out the row metadata.
    fn get_row_cells(&self, row: u8) -> u32 {
        let mask = (1 << 18) - 1;
        self.main_board[row as usize] & mask
    }

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
            for row in 0..9 {
                for column in 0..9 {
                    let cell = self.get_cell(row, column);
                    //write!(f, "{row},{column} ")?;
                    //write!(f, "{:01} ", column + (row * 3) * 3)?;
                    if column == 0 {
                        write!(f, "| ")?
                    }

                    if cell == 1 {
                        write!(f, "X ")?;
                    } else if cell == 2 {
                        write!(f, "O ")?;
                    } else {
                        write!(f, "_ ")?;
                    }

                    if (column + 1) % 3 == 0 {
                        write!(f, "| ")?;
                    }
                }
                writeln!(f)?;
                if (row + 1) % 3 == 0 {
                    writeln!(f, "— — — — — — — — — — — — —")?;
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
                )?
            }

            Ok(())
        }
    }
}
