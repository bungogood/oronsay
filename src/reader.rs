use std::{
    io::{BufReader, Read, Result},
    thread,
};

use crossbeam_channel::Sender;

fn find_newline_positions(buffer: &Vec<u8>) -> Option<(usize, usize)> {
    let mut positions = Vec::new();

    for (i, &byte) in buffer.iter().enumerate() {
        if byte == b'\n' {
            positions.push(i);
            if positions.len() == 2 {
                break;
            }
        }
    }

    if positions.len() == 2 {
        Some((positions[0], positions[1]))
    } else {
        None
    }
}

pub fn start_reader<R: Read + Send + 'static>(
    mut reader: BufReader<R>,
    read_tx: Sender<(usize, Vec<u8>)>,
    write_tx: Sender<(usize, Vec<u8>)>,
) -> Result<(usize, thread::JoinHandle<()>)> {
    // let buf_size = 65536; // 64 KiB
    // let buf_size = 32768; // 32 KiB
    let buf_size = 16368; // 16 KiB

    let mut current_buffer = vec![0u8; buf_size];
    let mut chunk_index = 0;

    let initial_read_size = reader.read(&mut current_buffer)?;

    let (first, second) = find_newline_positions(&current_buffer).expect("Error finding newlines");
    let line_length = second - first;

    let reader = thread::spawn(move || {
        let (pre_fix, post_fix) = if current_buffer[first - 1] == b'\r' {
            (first - 2, first + 2)
        } else {
            (first - 1, first + 1)
        };

        let mut next_buffer = if post_fix != line_length {
            let mut header = current_buffer[..=pre_fix].to_vec();
            header.push(b'\n');
            write_tx
                .send((chunk_index, header))
                .expect("Error sending header to writer");
            chunk_index += 1;
            current_buffer[first + 1..initial_read_size].to_vec()
        } else {
            current_buffer[..initial_read_size].to_vec()
        };

        loop {
            let read_size = match reader.read(&mut current_buffer) {
                Ok(0) => break, // End of file reached.
                Ok(size) => size,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return;
                }
            };

            // Append any carry-over from the previous iteration
            let mut data = if !next_buffer.is_empty() {
                let mut temp = std::mem::take(&mut next_buffer);
                temp.extend_from_slice(&current_buffer[..read_size]);
                temp
            } else {
                current_buffer[..read_size].to_vec()
            };

            // Find the last newline to determine the carry-over for the next chunk
            if let Some(last_newline_pos) = data.iter().rposition(|&b| b == b'\n') {
                next_buffer = data[last_newline_pos + 1..].to_vec();
                data.truncate(last_newline_pos + 1);
            } else {
                next_buffer.clear();
            }

            if read_tx.send((chunk_index, data)).is_err() {
                eprintln!("Error sending chunk to workers");
                return;
            }
            chunk_index += 1;

            // Prepare the current buffer for the next read, adjusting the size if necessary
            current_buffer.resize(buf_size, 0);
        }

        // Handle any remaining data as carry-over
        if !next_buffer.is_empty() {
            if read_tx.send((chunk_index, next_buffer)).is_err() {
                eprintln!("Error sending final carry-over chunk to workers");
            }
        }
    });

    Ok((line_length, reader))
}
