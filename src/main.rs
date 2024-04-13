use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
    thread,
};

use clap::Parser;
use crossbeam_channel;

mod reader;
mod worker;
mod writer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    infile: PathBuf,
    outfile: Option<PathBuf>,

    num_threads: Option<usize>,
}

fn create_reader(infile: PathBuf) -> io::Result<BufReader<File>> {
    let infile = File::open(infile)?;
    Ok(BufReader::new(infile))
}

fn create_writer(outfile: Option<PathBuf>) -> io::Result<BufWriter<Box<dyn io::Write + Send>>> {
    let writer: Box<dyn io::Write + Send> = match outfile {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::sink()),
    };
    Ok(BufWriter::new(writer))
}

fn get_num_threads() -> usize {
    thread::available_parallelism()
        .map(|num| num.get())
        .unwrap_or(1)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let num_workers = args.num_threads.unwrap_or(get_num_threads());

    let (read_tx, read_rx) = crossbeam_channel::unbounded();
    let (write_tx, write_rx) = crossbeam_channel::unbounded();

    let buf_reader = create_reader(args.infile)?;
    let buf_writer = create_writer(args.outfile)?;

    let reader_thread = reader::start_reader(buf_reader, read_tx);
    let worker_threads = worker::start_workers(num_workers, read_rx, write_tx);
    let writer_thread = writer::start_writer(buf_writer, write_rx);

    // Join threads
    reader_thread.join().unwrap();
    for worker in worker_threads {
        worker.join().unwrap();
    }
    writer_thread.join().unwrap();

    Ok(())
}
