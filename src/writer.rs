use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    thread,
};

use crossbeam::channel;
use sha2::{Digest, Sha256};

use crate::types::{ChunkStats, SolvedChunk};

pub struct Writer {
    writer: Option<BufWriter<File>>,
    hasher: Sha256,
    stats: ChunkStats,
    next_id: usize,
    no_hash: bool,
    verbose: bool,
}

impl Writer {
    fn append_chunk(&mut self, chunk: SolvedChunk) {
        self.stats.add(&chunk.stats);
        if self.verbose {
            println!("Processed chunk ID: {}, {}", chunk.id, self.stats.puzzles);
        }
        if let Some(w) = self.writer.as_mut() {
            w.write_all(&chunk.data).expect("Failed to write chunk");
        }
        if !self.no_hash {
            self.hasher.update(&chunk.data);
        }
        self.next_id += 1;
    }

    fn process(&mut self, output_rx: channel::Receiver<SolvedChunk>) {
        let mut pending_chunks = BTreeMap::new();

        for chunk in output_rx.iter() {
            if chunk.id == self.next_id {
                self.append_chunk(chunk);
                while let Some(next) = pending_chunks.remove(&self.next_id) {
                    self.append_chunk(next);
                }
            } else {
                pending_chunks.insert(chunk.id, chunk);
            }
        }

        if let Some(w) = self.writer.as_mut() {
            w.flush().expect("Failed to flush writer");
        }
    }

    pub fn spawn(
        output_rx: channel::Receiver<SolvedChunk>,
        outfile: Option<PathBuf>,
        no_hash: bool,
        verbose: bool,
    ) -> thread::JoinHandle<(Option<String>, ChunkStats)> {
        thread::spawn(move || {
            let writer = outfile
                .as_ref()
                .map(|path| {
                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(path)
                        .expect("Failed to open output file")
                })
                .map(BufWriter::new);

            let mut writer = Writer {
                writer,
                hasher: Sha256::new(),
                stats: ChunkStats::default(),
                next_id: 0,
                no_hash,
                verbose,
            };

            writer.process(output_rx);

            let hash = match no_hash {
                true => None,
                false => Some(format!("{:x}", writer.hasher.finalize())),
            };

            (hash, writer.stats)
        })
    }
}
