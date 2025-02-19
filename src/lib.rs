pub mod board;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    /// validate moves by ensuring that invalidity if:
    /// 1) cell is occupied
    /// 2) miniboard is 'uncontested' i.e. won by X or O, or drawn
    /// 3) miniboard coords don't correspond to previous move
    fn is_valid_move() {
        let mut b = Board::default();

        // 1) cell is occupied
        b.do_move(5, 5, 1);
        assert!(!b.is_valid_move(5, 5));

        // 2) miniboard is uncontestable
        assert!(!b.is_valid_move(1, 7));

        assert!(b.is_valid_move(6, 8));
        assert!(b.is_valid_move(6, 7));

        let uncontestable_minboard = 8;
        b.set_meta_data(
            uncontestable_minboard,
            board::flag::MINIBOARD_STATUS,
            board::flag::STATUS_BIT_SIZE,
            board::flag::STATUS_X_WIN as u32,
        );

        for row in 0..9 {
            for column in 0..9 {
                if (row, column) == (5, 5)
                    || Board::move_miniboard(row, column) == uncontestable_minboard
                {
                    continue;
                }

                assert!(b.is_valid_move(row, column), "{row} {column} was invalid!");
            }
        }

        b.reset();

        // 3) miniboard coordinate correspondence/matching

        // 4) exception of uncontestable minboard to play any other valid miniboard
        let (_row, _column) = (1, 1);
        let _move_corresponding = Board::move_corresponding_miniboard(_row, _column);
        b.do_move(_row, _column, 1);
        b.set_meta_data(
            _move_corresponding,
            board::flag::MINIBOARD_STATUS,
            board::flag::STATUS_BIT_SIZE,
            board::flag::STATUS_X_WIN as u32,
        );

        for row in 0..9 {
            for col in 0..9 {
                if Board::move_miniboard(row, col) == _move_corresponding
                    || (row, col) == (_row, _column)
                {
                    println!("{row}, {col}");
                    assert!(!b.is_valid_move(row, col));
                    continue;
                }

                assert!(b.is_valid_move(row, col));
            }
        }
    }
}
