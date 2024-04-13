use std::thread;

use crossbeam_channel::{Receiver, Sender};

pub fn start_workers(
    num_threads: usize,
    read_rx: Receiver<(usize, Vec<u8>)>,
    write_tx: Sender<(usize, Vec<u8>)>,
) -> Vec<thread::JoinHandle<()>> {
    (0..num_threads)
        .map(|_| {
            let rx = read_rx.clone();
            let tx = write_tx.clone();
            thread::spawn(move || {
                for (index, chunk) in rx {
                    // let lines = chunk[..]
                    //     .split(|&b| b == b'\n')
                    //     .map(|line| String::from_utf8_lossy(line).to_string())
                    //     .collect();
                    // let processed_line = process_chunk(lines); // Define your processing
                    // tx.send((index, processed_line)).unwrap();
                    tx.send((index, chunk)).unwrap();
                }
            })
        })
        .collect()
}

// fn process_chunk(lines: Vec<String>) -> Vec<String> {
//     lines
// }
