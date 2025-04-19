use std::{sync::Arc, thread};

use crossbeam::channel;
use memmap2::Mmap;

use crate::types::{ChunkStats, PuzzleChunk, SolvedChunk};

struct ReaderMetadata {
    line_length: usize,
    header_text: Option<Vec<u8>>,
    data_start: usize,
}

pub struct Reader;

impl Reader {
    pub fn spawn(
        mmap: Arc<Mmap>,
        chunk_tx: channel::Sender<PuzzleChunk>,
        output_tx: channel::Sender<SolvedChunk>,
        chunk_size: usize,
    ) -> (usize, thread::JoinHandle<()>) {
        let metadata = Self::analyze_mmap(&mmap);

        let ReaderMetadata {
            line_length,
            header_text,
            data_start,
        } = match metadata {
            Some(metadata) => metadata,
            None => panic!("Failed to analyze mmap"),
        };

        let chunk_size = chunk_size - (chunk_size % line_length);

        // Optionally send header
        let mut next_id = 0;
        if let Some(header_text) = header_text {
            output_tx
                .send(SolvedChunk {
                    id: next_id,
                    data: header_text,
                    stats: ChunkStats::default(),
                })
                .expect("Failed to send header chunk");
            next_id += 1;
        }

        let reader = thread::spawn(move || {
            let mut start = data_start;

            while start < mmap.len() {
                let end = (start + chunk_size).min(mmap.len());
                let chunk = PuzzleChunk {
                    id: next_id,
                    start,
                    end,
                    mmap: Arc::clone(&mmap),
                };
                chunk_tx.send(chunk).expect("Failed to send chunk");
                next_id += 1;
                start += chunk_size;
            }
        });

        (line_length, reader)
    }

    /// Finds line metadata and extracts header (if present)
    fn analyze_mmap(mmap: &Mmap) -> Option<ReaderMetadata> {
        let (first, second) = Self::find_line_bounds(&mmap[..])?;
        let line_length = second - first;

        // Determine if there's a header by looking at pre/post fix newline formatting
        let (pre_fix, post_fix) = if first > 0 && mmap[first - 1] == b'\r' {
            (first - 2, first + 2)
        } else {
            (first - 1, first + 1)
        };

        let (header_text, data_start) = if post_fix != line_length {
            let mut header = mmap[..=pre_fix].iter().copied().collect::<Vec<u8>>();
            header.push(b'\n');
            (Some(header), post_fix)
        } else {
            (None, 0)
        };

        Some(ReaderMetadata {
            line_length,
            header_text,
            data_start,
        })
    }

    fn find_line_bounds(buffer: &[u8]) -> Option<(usize, usize)> {
        let mut newlines = buffer.iter().enumerate().filter(|(_, &b)| b == b'\n');
        let first = newlines.next()?.0;
        let second = newlines.next()?.0;
        Some((first, second))
    }
}
