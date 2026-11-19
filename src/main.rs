use chrono::{DateTime, Local, Utc};
use clap::{Arg, ArgAction, Command};
use std::io::{self, Write};

fn main() {
    let matches = Command::new("cz")
        .about("cz tools")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("now").about("显示当前时间").arg(
                Arg::new("local")
                    .short('l')
                    .long("local")
                    .help("以本地时间格式输出")
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("tf")
                .about("将 Unix 时间戳转换为本地时间")
                .arg(
                    Arg::new("timestamp")
                        .help("Unix 时间戳（秒或毫秒）")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("jf").about("格式化 JSON").arg(
                Arg::new("json")
                    .help("要格式化的 JSON 文本")
                    .num_args(1..)
                    .trailing_var_arg(true),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("now", matches)) => {
            if matches.get_flag("local") {
                println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
            } else {
                println!("{}", Local::now().timestamp_millis() as u64);
            }
        }
        Some(("tf", matches)) => {
            let mut timestamp = *matches
                .get_one::<u64>("timestamp")
                .expect("required timestamp is validated by clap");

            if timestamp > 9_999_999_999 {
                timestamp /= 1000;
            }

            let datetime = i64::try_from(timestamp)
                .ok()
                .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0));

            match datetime {
                Some(datetime) => {
                    println!(
                        "{}",
                        datetime.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")
                    );
                }
                None => {
                    eprintln!("error: timestamp is out of range");
                    std::process::exit(2);
                }
            }
        }
        Some(("jf", matches)) => {
            let json = if let Some(args) = matches.get_many::<String>("json") {
                args.cloned().collect::<Vec<_>>().join(" ")
            } else {
                println!("JSON: ");
                io::stdout().flush().expect("failed to flush prompt");

                let mut input = String::new();
                if let Err(error) = io::stdin().read_line(&mut input) {
                    eprintln!("error: failed to read JSON input: {error}");
                    std::process::exit(2);
                }
                input
            };

            match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(value) => match serde_json::to_string_pretty(&value) {
                    Ok(formatted) => {
                        println!("=======================formatted======================");
                        println!("{formatted}");
                    },
                    Err(error) => {
                        eprintln!("error: failed to format JSON: {error}");
                        std::process::exit(2);
                    }
                },
                Err(error) => {
                    eprintln!("error: invalid JSON: {error}");
                    std::process::exit(2);
                }
            }
        }
        _ => unreachable!("clap requires a recognized subcommand"),
    }
}
