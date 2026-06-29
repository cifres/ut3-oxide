#[macro_export]
macro_rules! uline {
    ($s:literal) => {
        concat!("\x1b[4m", $s, "\x1b[0m")
    };
}

pub const QUESTIONS: [(&str, (u8, u8), &str); 7] = [
    ("Welcome to the UT3 Tutorial!", (0, 0), ""),
    (
        "1. Like regular tic-tac-toe, you win by getting a three-in-a-line\nTo form a line \
            here enter the coordinates for the \x1b[34m3rd row \x1b[31m5th column\x1b[0m like \
            \"\x1b[34mrow\x1b[0m\x1b[31mcol\x1b[0m\"",
        (3, 5),
        "\x1b[3mHint\x1b[0m: try 35",
    ),
    (
        concat!(
            "Excellent! You won a \x1b[34mminiboard\x1b[0m\n\n\
            2. Time to understand the core mechanic!\nIn UT3, your ",
            uline!("move"),
            " sends your opponent to a corresponding ",
            uline!("miniboard"),
            " and vice versa.\nSo, the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m ",
            uline!("move"),
            " in any ",
            uline!("miniboard"),
            " sends your opponent to the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m ",
            uline!("miniboard"),
            ".\n\nNotice how your previous ",
            uline!("move"),
            " sent your opponent to the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m ",
            uline!("miniboard"),
            ".\nThe green \x1b[1;32m_\x1b[0m's, and \x1b[32mcoordinates\x1b[0m on the sides \
            highlight valid moves"
        ),
        (0, 0),
        "",
    ),
    (
        "3. Quiz time! Which coordinates will send your opponent to the \
        \x1b[34mcentre-\x1b[31mleft\x1b[0m \x1b[4mminiboard\x1b[0m?",
        (1, 6),
        "\x1b[3mHint\x1b[0m: in the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m miniboard, look for the \
        coordinates on the side that line up with the \x1b[34mcentre-\x1b[31mleft\x1b[0m cell",
    ),
    (
        concat!(
            "4. Time for the final rule! \x1b[31mO\x1b[0m will now play in the middle-centre cell.\
            \nBut, the \x1b[34mmiddle\x1b[0m-\x1b[31mcentre\x1b[0m ",
            uline!("miniboard"),
            " is won already.\nWhat do you think will happen?"
        ),
        (0, 0),
        "",
    ),
    (
        "Notice how you can make a move anywhere now! Remember, green \x1b[1;32m_\x1b[0m's \
        highlight valid moves.",
        (0, 0),
        "",
    ),
    (
        concat!(
            "This is because you were sent to a won ",
            uline!("miniboard"),
            " and so got redirected to the whole board!"
        ),
        (0, 0),
        "",
    ),
];
