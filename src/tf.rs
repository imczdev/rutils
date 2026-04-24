mod now;

use chrono::{DateTime, NaiveDateTime, Local};
use std::env;

fn main() {
     // 从命令行参数获取时间戳
     let args: Vec<String> = env::args().collect();

     if args.len() != 2 {
         eprintln!("timestamp not found");
         return;
     }
 
     // 尝试将参数解析为 u64
     let mut timestamp: u64 = match args[1].parse() {
         Ok(ts) => ts,
         Err(_) => {
             eprintln!("timestamp invalid");
             return;
         }
     };

     if timestamp > 9_999_999_999 {
        timestamp = timestamp / 1000
     }

 
     // 将时间戳转换为 NaiveDateTime
     let naive_datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0);
     
     // 转换为 DateTime
     let datetime: DateTime<Local> = DateTime::from_utc(naive_datetime, Local::now().offset().clone());
 
     // 格式化并输出
     let formatted = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
     println!("{}", formatted);
}
