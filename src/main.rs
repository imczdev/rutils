use chrono::{DateTime, Local, Utc};
use clap::{Arg, ArgAction, Command};
use std::{
    env,
    io::{self, IsTerminal, Write},
};

fn colorize_json(json: &str, enabled: bool) -> String {
    if !enabled {
        return json.to_owned();
    }

    let bytes = json.as_bytes();
    let mut output = String::with_capacity(json.len() + 64);
    let mut index = 0;

    while index < bytes.len() {
        let start = index;
        let color = match bytes[index] {
            b'"' => {
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        b'"' => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }

                let mut next = index;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                    next += 1;
                }
                if next < bytes.len() && bytes[next] == b':' {
                    "36"
                } else {
                    "32"
                }
            }
            b'-' | b'0'..=b'9' | b't' | b'f' | b'n' => {
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !matches!(bytes[index], b',' | b']' | b'}' | b':')
                {
                    index += 1;
                }
                match &json[start..index] {
                    "true" | "false" | "null" => "35",
                    _ => "33",
                }
            }
            _ => {
                output.push(bytes[index] as char);
                index += 1;
                continue;
            }
        };

        output.push_str("\x1b[");
        output.push_str(color);
        output.push('m');
        output.push_str(&json[start..index]);
        output.push_str("\x1b[0m");
    }

    output
}

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
                        let use_color =
                            io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none();
                        println!("{}", colorize_json(&formatted, use_color));
                    }
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

#[cfg(test)]
mod tests {
    use super::colorize_json;

    #[test]
    fn json_output_is_unchanged_when_color_is_disabled() {
        let json = "{\n  \"message\": \"hello world\",\n  \"count\": 2\n}";
        assert_eq!(colorize_json(json, false), json);
    }

    #[test]
    fn json_values_and_keys_receive_distinct_colors() {
        let colored = colorize_json("{\"key\":\"value\",\"count\":2,\"ok\":true}", true);
        assert!(colored.contains("\x1b[36m\"key\"\x1b[0m"));
        assert!(colored.contains("\x1b[32m\"value\"\x1b[0m"));
        assert!(colored.contains("\x1b[33m2\x1b[0m"));
        assert!(colored.contains("\x1b[35mtrue\x1b[0m"));
    }
}
