use clap::{App, Arg};
use libc;
use num_cpus;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, LineWriter, Seek, SeekFrom, Write};
use std::path::Path;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;
use walkdir::WalkDir;

pub fn is_positive_number(v: String) -> Result<(), String> {
    if v.parse::<u64>().is_ok() {
        return Ok(());
    }

    Err(format!("{} isn't a positive number", &*v))
}

#[derive(Debug)]
struct GenInput {
    reader: BufReader<File>,
    writer: LineWriter<File>,
}

impl GenInput {
    fn read(&mut self) -> io::Result<Option<String>> {
        let mut buf = String::new();
        match self.reader.read_line(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    Ok(None)
                } else {
                    Ok(Some(buf))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn rewind(&mut self) -> io::Result<()> {
        self.reader.seek(SeekFrom::Start(0)).map(|_| ())
    }

    fn write(&mut self, line: &str) -> io::Result<()> {
        self.writer.write_all(line.as_bytes())
    }
}

fn generate(mut items: Vec<GenInput>, interval: Duration) {
    loop {
        for item in items.iter_mut() {
            match item.read() {
                Ok(Some(line)) => {
                    item.write(&line)
                        .map_err(|err| eprintln!("Error: {:?}", err))
                        .ok();
                }
                Ok(None) => {
                    item.rewind()
                        .map_err(|err| eprintln!("Error: {:?}", err))
                        .ok();
                }
                Err(error) => {
                    eprintln!("Error reading: {:?}", error);
                }
            }

            sleep(interval);
        }
    }
}

fn run(
    in_dir: &str,
    out_dir: &str,
    interval: Duration,
    parallelism_num: usize,
) -> io::Result<Vec<JoinHandle<()>>> {
    let in_path = Path::new(in_dir);
    let out_path = Path::new(out_dir);
    let mut workers_data: Vec<Vec<GenInput>> = Vec::with_capacity(parallelism_num);
    let mut counter: usize = 0;

    for _i in 0..parallelism_num {
        workers_data.push(vec![]);
    }

    println!(
        "{} -> {} (threads: {}, interval: {:?})",
        in_dir, out_dir, parallelism_num, interval
    );

    for entry in WalkDir::new(in_dir).into_iter().filter_map(|e| e.ok()) {
        let path_in = entry.path();
        if path_in.is_file() {
            if let Ok(rel_dir) = path_in.strip_prefix(in_path) {
                let path_out = out_path.join(rel_dir);
                let dir_to_create = path_out.parent().unwrap();
                fs::create_dir_all(dir_to_create)?;
                let read_file = File::open(path_in)?;
                let write_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(path_out)?;

                let index: usize = counter % parallelism_num;
                let reader = BufReader::new(read_file);
                let writer = LineWriter::new(write_file);
                let gen_input = GenInput { reader, writer };
                workers_data[index].push(gen_input);

                counter += 1;
            }
        }
    }

    let mut join_handles = vec![];
    for worker_data in workers_data.into_iter() {
        if worker_data.len() > 0 {
            join_handles.push(thread::spawn(move || {
                generate(worker_data, interval);
            }));
        }
    }

    Ok(join_handles)
}

fn main() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let matches = App::new("loggen")
        .version("0.1.0")
        .author("Mariano Guerra <mariano@marianoguerra.org>")
        .about("Generates log lines from a folder structure of samples")
        .arg(
            Arg::with_name("in-base-dir")
                .short("i")
                .long("in-base-dir")
                .value_name("FILE")
                .required(true)
                .help("Input base directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("out-base-dir")
                .short("o")
                .long("out-base-dir")
                .value_name("FILE")
                .required(true)
                .help("Output base directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .short("t")
                .long("interval")
                .value_name("MS")
                .help("Time in milliseconds between reads")
                .validator(is_positive_number)
                .default_value("250")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("parallelism")
                .short("p")
                .long("parallelism")
                .value_name("COUNT")
                .help("Number of parallel generators")
                .validator(is_positive_number)
                .default_value("2")
                .takes_value(true),
        )
        .get_matches();

    let in_dir = matches.value_of("in-base-dir").unwrap();
    let out_dir = matches.value_of("out-base-dir").unwrap();
    let interval_str = matches.value_of("interval").unwrap_or("0");
    let interval_num = interval_str.parse::<u64>().unwrap();
    let parallelism_str = matches.value_of("parallelism").unwrap_or("0");
    let parallelism_num_0 = parallelism_str.parse::<usize>().unwrap();
    let parallelism_num = if parallelism_num_0 == 0 {
        num_cpus::get() as usize
    } else {
        parallelism_num_0
    };

    match run(
        in_dir,
        out_dir,
        Duration::from_millis(interval_num),
        parallelism_num,
    ) {
        Ok(join_handles) => {
            for join_handle in join_handles {
                match join_handle.join() {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Error in thread: {:?}", error);
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    }
}
