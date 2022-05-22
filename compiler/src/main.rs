mod lib;

use lib::*;
use std::process;

#[derive(Default)]
struct Options {
	exe: String,
	file: String,
	watch: bool,
	web: bool,
}

fn process_args() -> Options {
	let mut args = std::env::args();
	let exe = args.next().unwrap();
	let mut file = None;
	let mut watch = None;
	let mut web = None;
	let mut fail = false;

	for arg in args {
		match arg.as_str() {
			"--watch" =>  {
				if watch.is_some() {
					fail = true;
				}
				watch = Some(true);
			},
			"--web" =>  {
				if web.is_some() {
					fail = true;
				}
				web = Some(true);
			},
			_ => {
				if file.is_some() {
					fail = true;
				}
				file = Some(arg);
			}
		}
	}

	if fail || file.is_none() {
		eprintln!("usage: {} FILE [--watch] [--web]", exe);
		process::exit(1);
	}

	Options {
		exe,
		file: file.unwrap(),
		watch: watch.unwrap_or_default(),
		web: web.unwrap_or_default(),
	}
}

fn main() {
	let options = process_args();
	if options.watch {
		watch(&options.exe, &options.file, options.web);
	} else if let Err(message) = build(&options.exe, &options.file, options.web) {
		eprintln!("{}", message)
	}
}