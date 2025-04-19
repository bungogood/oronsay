use memmap2::Mmap;
use std::{sync::Arc, time::Duration};

pub struct PuzzleChunk {
    pub id: usize,
    pub start: usize,
    pub end: usize,
    pub mmap: Arc<Mmap>,
}

#[derive(Default)]
pub struct ChunkStats {
    pub chunks: usize,
    pub puzzles: usize,
    pub solutions: usize,
    pub no_guesses: usize,
    pub guesses: usize,
    pub elapsed: Duration,
}

impl ChunkStats {
    pub fn add(&mut self, other: &ChunkStats) {
        self.chunks += other.chunks;
        self.puzzles += other.puzzles;
        self.solutions += other.solutions;
        self.no_guesses += other.no_guesses;
        self.guesses += other.guesses;
        self.elapsed += other.elapsed;
    }
}

pub struct SolvedChunk {
    pub id: usize,
    pub data: Vec<u8>,
    pub stats: ChunkStats,
}
