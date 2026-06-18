# UT3 Oxide
A fast minimax ai or local multiplayer for the game *Ultimate* Tic Tac Toe.

## Description
UT3 Oxide is a command-line program in Rust for playing the Ultimate Tic Tac Toe (UTTTT/UT3) game against a Bot or another player locally. The AI is algorithmic using Minimax with AlphaBeta (replace a-b) pruning (link to wiki) and the underlying game representation utilises bitboards, bitpacking, bitwise operations (link) for efficient storage, cache utilisation, and performance.

- Gif here of gameplay

## Motivation
I wanted to practice, and play with my friends offline, however the alternatives all required an internet connection. So I after I first built the game in TypeScript (link to repo), I wanted to make it performant and accessible offline, so I built UT3. Also it was a fantastic opportunity to learn bitpacking and bitwise operations which before this seemed inaccessible to me -- (rewrite)

## Quick Start

Use precompiled binary in releases (link) or build from source

> [!NOTE]
> Prerequisite 
> Rust, Cargo Rustc, use rustup (link)


git clone repo
cargo run --release

./ut3_oxide

## Usage

linux ./ut3_oxide

windows powershell ./ut3_oxide

bench the ais <depth1> <depth2> <repeats>

## Building

git, cargo b t r bench --profile profiling release debug

## Rules && How to Play

## Acknowledgements
AlphaZero Variant -- comment
Zurich lady uttt


