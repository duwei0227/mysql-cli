use std::{time::Instant, vec};

use clap::Parser;
use dialoguer::{BasicHistory, Input, console::style};

mod format;
mod sqlutil;
mod theme;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mysql_config = MySqlConfig::parse();
    println!("{:?}", mysql_config);

    let mut conn = sqlutil::get_connection(&mysql_config).await?;

    sqlutil::test_connection(&mut conn).await?;

    println!("Database connection successful.");
    println!("");
    // 终端文字采用红色，输出内容为：事务会自动提交。
    println!(
        "{}",
        style("使用 ; 或 \\G 作为SQL结束符号。\n注意：更新类操作（insert, update, delete）会自动提交事务\n当前不支持手动控制事务\n输入 exit 或 quit 或 q 或 Q 退出程序\n\n").red()
    );

    let mut history = BasicHistory::new().max_entries(100).no_duplicates(true);
    let theme = theme::CustomTerminalTheme::default();

    loop {
        let input: String = Input::with_theme(&theme)
            .history_with(&mut history)
            .with_prompt("mysql> ")
            .validate_with(|input: &String| -> Result<(), &str> {
                // input 当前限制只能以 use, select, show, explain, update, delete, insert, exit, quit, q, Q 开头
                let start = input.trim_start().split_whitespace().rev().last().unwrap_or("");
                let valid_starts = ["use", "select", "show", "explain", "create", "update", "delete", "insert", "exit", "quit", "q", "Q"];
                if valid_starts.contains(&start) {
                    Ok(())
                } else {
                    Err("Only support SQL starting with use, select, show, explain, update, delete, insert, exit, quit, q, Q")
                }
            })
            .interact_text()?;

        let mut input = input.trim().to_string();
        // 判断 input 是否是 exit, quit, q, Q
        if input == "exit" || input == "quit" || input == "q" || input == "Q" {
            break;
        }

        let mut sqls: Vec<String> = vec![];
        sqls.push(input.clone());

        // 如果 sql 不是以 ; 或 \G 结尾，则继续输入
        while !input.ends_with(';') && !input.ends_with("\\G") {
            input = Input::with_theme(&theme)
                .history_with(&mut history)
                .with_prompt("    -> ")
                .interact_text()?;

            sqls.push(input.clone());
        }

        let execute_sql = sqls.join(" ");

        // ["use", "select", "show", "explain", "update", "delete", "insert", "exit", "quit", "q", "Q"];
        let start = input
            .trim_start()
            .split_whitespace()
            .rev()
            .last()
            .unwrap_or("");
        // 执行查询
        if start == "select" || start == "show" || start == "explain" {
            // 执行查询
            let start = Instant::now();
            let result = sqlutil::select_many(&mut conn, &execute_sql).await;
            let duration_sec = start.elapsed().as_secs_f64();
            // duration_sec 保留2位小数
            let duration_sec = format!("{:.2}", duration_sec);
            if let Ok(result) = result {
                let result_count = result.len();
                if result_count == 0 {
                    println!("Empty set ({} sec)\n", duration_sec);
                    continue;
                }
                format::format_result_print(result, sqls.last().unwrap().ends_with(";"));

                let mut result_msg =
                    format!("{} row in set ({} sec)\n", result_count, duration_sec);
                if result_count != 1 {
                    result_msg = format!("{} rows in set ({} sec)\n", result_count, duration_sec);
                }
                println!("{}", result_msg);
            } else {
                println!("Error: {:?}\n\n", result.err().unwrap());
            }
        } else if start == "use" {
            let result = sqlutil::execute_raw(&mut conn, &execute_sql).await;

            if let Ok(_) = result {
                println!("Database changed.\n");
            } else {
                println!("Error: {:?}\n\n", result.err().unwrap());
            }
        } else {
            // 更新类操作,需要开启事务
            let start = Instant::now();
            let fetch_count = sqlutil::execute_query(&mut conn, &execute_sql).await;
            let duration_sec = start.elapsed().as_secs_f64();
            // duration_sec 保留2位小数
            let duration_sec = format!("{:.2}", duration_sec);
            if let Ok(count) = fetch_count {
                let mut result_msg =
                    format!("Query OK, {} row affected ({} sec)\n", count, duration_sec);
                if count != 1 {
                    result_msg =
                        format!("Query OK, {} rows affected ({} sec)\n", count, duration_sec);
                }
                println!("{}", result_msg);
            } else {
                println!("Error: {:?}\n\n", fetch_count.err().unwrap());
            }
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
#[command(version)]
struct MySqlConfig {
    #[arg(short = 'H', long)]
    host: String,
    #[arg(short = 'P', long, default_value_t = 3306)]
    port: u16,
    #[arg(short, long)]
    user: String,
    #[arg(short, long)]
    password: String,
}
