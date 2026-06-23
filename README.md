# UT3 Oxide

*you·tee·three ox·ide*

A performant command-line program to play *Ultimate* Tic Tac Toe featuring
Alpha-Beta pruning AI, and local multiplayer for the game.

## Description

UT3 Oxide is a command-line program in Rust for playing the Ultimate Tic Tac Toe
(UT3) game against a bot or another player locally. The AI is algorithmic
and uses Minimax with [Alpha-Beta
pruning](https://en.wikipedia.org/wiki/Alpha-beta_pruning) (α-β). The
engine representation utilises bitfields, bitpacking, and bitwise
operations for efficient storage, cache utilisation, and improved
performance.

Gif here of gameplay

## Motivation

* Bullet points

I wanted to practice, and play with my friends offline, however alternatives
required an internet connection. So after I first built the game in
[TypeScript](https://github.com/cifres/AI-UTTT-V2) to prototype the concept
([playable here](utttai.netlify.app/)), I
wanted to make it performant and accessible offline, so I developed UT3 Oxide.
Finally, it was a fantastic opportunity to deepen my understanding of bitpacking
and bitwise operations in a performance context.

## Quick Start

The easiest way to try out the game is to download a pre-compiled binary for
your system, then running the following:

```bash
./ut3_oxide
```

## Building

> [!NOTE]
> [Rust](https://rust-lang.org/tools/install) and [Git](https://git-scm.com/install) must be installed and on `path`.

```bash
# Clone 
git clone github.com/cifres/ut3-oxide
```

```bash
# Change directory, build and run
cd ut3_oxide
cargo run --release
```

### Benching

```bash
cargo bench --profile release
```

## Usage

### Player versus AI

Play versus the AI at a search depth of `9`:

```bash
# ./ut3_oxide [depth]
./ut3_oxide 9
```

### AI versus AI

Alternatively, you may pit two AIs against each other at specified search depths. Here the both AIs run search at an equal depth and play 10 games total.

```bash
# ./ut3_oxide [depth_ai_x] [depth_ai_o] [repetitions]
./ut3_oxide 7 7 10
```

## Rules && How to Play

## Acknowledgements

AlphaZero Variant -- comment
Zurich lady uttt
