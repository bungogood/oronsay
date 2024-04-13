use std::thread;

use crossbeam_channel::{Receiver, Sender};

use crate::{consts::N_CELLS, sudoku::Sudoku};

pub fn start_workers(
    num_threads: usize,
    read_rx: Receiver<(usize, Vec<u8>)>,
    write_tx: Sender<(usize, Vec<u8>)>,
) -> Vec<thread::JoinHandle<usize>> {
    (0..num_threads)
        .map(|_| {
            let rx = read_rx.clone();
            let tx = write_tx.clone();
            thread::spawn(move || {
                let mut num_puzzles = 0;
                for (index, chunk) in rx {
                    let (count, solved_chunk) = process_chunk(chunk);
                    tx.send((index, solved_chunk))
                        .expect("Error sending chunk to writer");
                    num_puzzles += count;
                }
                num_puzzles
            })
        })
        .collect()
}

fn process_chunk(chunk: Vec<u8>) -> (usize, Vec<u8>) {
    let mut solved_chunk = Vec::with_capacity(chunk.len() * 3);

    let mut count = 0;
    for puzzle_slice in chunk.chunks_exact(N_CELLS + 1) {
        assert!(puzzle_slice[N_CELLS] == b'\n', "Invalid chunk format");

        let mut sudoku = Sudoku::new(puzzle_slice[..N_CELLS].try_into().unwrap());

        solved_chunk.extend_from_slice(sudoku.grid.as_ref());
        solved_chunk.push(b',');

        if !sudoku.solve() {
            panic!("Failed to solve sudoku");
        }

        solved_chunk.extend_from_slice(sudoku.grid.as_ref());
        solved_chunk.push(b'\n');

        count += 1;
    }

    (count, solved_chunk)
}
