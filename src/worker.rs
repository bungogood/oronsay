use std::thread;

use crossbeam::channel;

use crate::{
    solver::Solver,
    sudoku::{Puzzle, N_CELLS},
    types::{ChunkStats, PuzzleChunk, SolvedChunk},
};

pub struct Worker<S: Solver> {
    solver: S,
    line_length: usize,
}

impl<S: Solver> Worker<S> {
    fn process_chunk(&self, chunk: PuzzleChunk, state: &mut S::State) -> SolvedChunk {
        let start = std::time::Instant::now();
        let data = &chunk.mmap[chunk.start..chunk.end];
        let mut solved = SolvedChunk {
            id: chunk.id,
            data: Vec::with_capacity(data.len() * 3),
            stats: ChunkStats::default(),
        };

        for puzzle_slice in data.chunks_exact(self.line_length) {
            let puzzle = Puzzle::new(&puzzle_slice[..N_CELLS]);
            solved.stats.puzzles += 1;

            solved.data.extend_from_slice(puzzle.grid.as_ref());
            solved.data.push(b',');

            let solution = match self.solver.solve(&puzzle, state) {
                Some(solution) => solution,
                None => {
                    let sudoku = puzzle.sudoku();
                    panic!(
                        "Failed to solve sudoku {}: {}\n{}",
                        solved.stats.puzzles,
                        sudoku.to_string(),
                        sudoku.pretty()
                    )
                }
            };

            solved.data.extend_from_slice(solution.grid.as_ref());
            solved.data.push(b'\n');
        }

        solved.stats.chunks += 1;
        solved.stats.elapsed = start.elapsed();
        solved
    }

    pub fn spawn(
        solver: S,
        line_length: usize,
        chunk_rx: channel::Receiver<PuzzleChunk>,
        output_tx: channel::Sender<SolvedChunk>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut state = solver.make_state();
            let worker = Worker {
                solver,
                line_length,
            };
            for chunk in chunk_rx.iter() {
                let solved = worker.process_chunk(chunk, &mut state);
                output_tx.send(solved).expect("Failed to send output chunk");
            }
        })
    }

    pub fn spawn_multiple(
        solver: S,
        line_length: usize,
        chunk_rx: channel::Receiver<PuzzleChunk>,
        output_tx: channel::Sender<SolvedChunk>,
        num_workers: usize,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            let handle = Worker::spawn(
                solver.clone(),
                line_length,
                chunk_rx.clone(),
                output_tx.clone(),
            );
            handles.push(handle);
        }
        handles
    }
}
