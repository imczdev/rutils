use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("now")
        .about("时间工具")
        .version("1.0.0")
        //
        // Only a few of its arguments are implemented below.
        .subcommand(
            Command::new("local")
                .short_flag('l')
                .long_flag("local")
                .about("本地时间格式")
                .arg(
                    Arg::new("local")
                        .action(ArgAction::Set)
                        .num_args(1..),
                )
        )
        // Sync subcommand
        .get_matches();

    match matches.subcommand() {
        Some(("local", sync_matches)) => {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("{}",now);
        }
        _ => {
            let now = chrono::Local::now().timestamp_millis() as u64;
            println!("{}",now);
        },
    }
}