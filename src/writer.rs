use crossbeam_channel::Receiver;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::{BufWriter, Write};
use std::thread;

#[derive(Debug, Eq)]
struct HeapItem {
    index: usize,
    chunk: Vec<u8>,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.index.cmp(&self.index)
    }
}

pub fn start_writer<W: Write + Send + 'static>(
    mut writer: BufWriter<W>,
    write_rx: Receiver<(usize, Vec<u8>)>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut heap = BinaryHeap::new();
        let mut next_index = 0;

        for (index, chunk) in write_rx.iter() {
            heap.push(HeapItem { index, chunk });

            while heap.peek().map_or(false, |item| item.index == next_index) {
                if let Some(item) = heap.pop() {
                    writer.write_all(&item.chunk).unwrap();
                    next_index += 1;
                }
            }
        }

        writer.flush().unwrap();
    })
}
