mod board;

fn main() {
    println!("Hello, world!");
    board::hello();
    let mut board = board::Board::new();
    //board.main_board[0] = 4_294_119_914;

    board.set_cell(5, 5, 1);
    board.set_cell(5, 4, 1);
    board.set_cell(5, 8, 2);

    board.set_cell(0, 0, 1);
    board.set_cell(0, 1, 1);
    board.set_cell(0, 2, 2);

    board.set_cell(4, 4, 1);
    board.set_cell(6, 7, 2);
    board.set_cell(5, 3, 2);
    //println!("{:b}", board.get_row_cells(5));
    println!("{}", board);
    println!("{:#}", board);
    println!("metadata:{:14b}", board.get_row_metadata(0));

}
