use sqlx::mysql::{MySqlColumn, MySqlRow};
use sqlx::{Column, Row, TypeInfo};

pub fn format_result_print(datas: Vec<MySqlRow>, is_semicolon: bool) {
    if is_semicolon {
        SemicolonFormat::format_print(datas);
    } else {
        GFormat::format_print(datas);
    }
}

trait Format {
    fn format_print(datas: Vec<MySqlRow>);
}

struct SemicolonFormat;
impl Format for SemicolonFormat {
    fn format_print(datas: Vec<MySqlRow>) {
        // 循环 datas ，计算每一列值的最大长度或者列名的长度，选择较大值作为该列的打印宽度
        let mut column_widths: Vec<usize> = vec![];
        for row in datas.iter() {
            for (i, column) in row.columns().iter().enumerate() {
                let value = get_string_value(column, row);
                let width = value.len().max(column.name().len());
                if column_widths.len() <= i {
                    column_widths.push(width);
                } else if column_widths[i] < width {
                    column_widths[i] = width;
                }
            }
        }

        // 打印分隔符
        print!("+");
        for width in &column_widths {
            print!("{:-^width$}+", "", width = *width + 2);
        }
        println!();
        // 打印列名
        print!("|");
        for (i, col) in datas[0].columns().iter().enumerate() {
            let col_name = col.name();
            print!(" {:^width$} |", col_name, width = column_widths[i]);
        }
        println!();
        // 打印分隔符
        print!("+");
        for width in &column_widths {
            print!("{:-^width$}+", "", width = *width + 2);
        }
        println!();
        // 打印数据行
        for row in datas.iter() {
            print!("|");
            for (i, col) in row.columns().iter().enumerate() {
                let value = get_string_value(col, row);

                print!(" {:^width$} |", value, width = column_widths[i])
            }
            println!();
        }
        // 打印分隔符
        print!("+");
        for width in &column_widths {
            print!("{:-^width$}+", "", width = *width + 2);
        }
        println!();
    }
}

struct GFormat;
impl Format for GFormat {
    fn format_print(datas: Vec<MySqlRow>) {
        // 计算列名的最大长度
        let max_column_name_length = datas
            .iter()
            .flat_map(|data| data.columns())
            .map(|column| column.name().len())
            .max()
            .unwrap_or(0);

        for (i, row) in datas.iter().enumerate() {
            println!(
                "*************************** {}. row ***************************",
                i + 1
            );
            for column in row.columns() {
                let column_name = column.name().to_uppercase();
                // 右对齐打印列名和值
                println!(
                    "{:>width$} : {}",
                    column_name,
                    get_string_value(column, row),
                    width = max_column_name_length
                );
            }
        }
    }
}

fn get_string_value(column: &MySqlColumn, row: &MySqlRow) -> String {
    // println!("Column Type: {}", column.type_info().name());
    // println!("Column Name: {}", column.name());
    // println!("row: {:?}", row);

    match column.type_info().name() {
        "INT" | "TINYINT" | "SMALLINT" | "MEDIUMINT" | "BIGINT" => row
            .try_get::<i64, usize>(column.ordinal())
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string()),
        "BIGINT UNSIGNED" | "INT UNSIGNED" | "TINYINT UNSIGNED" | "SMALLINT UNSIGNED"
        | "MEDIUMINT UNSIGNED" => row
            .try_get::<u64, usize>(column.ordinal())
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string()),
        "FLOAT" | "DOUBLE" | "DECIMAL" => row
            .try_get::<f64, usize>(column.ordinal())
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string()),
        "VARCHAR" | "TEXT" | "CHAR" | "LONGTEXT" => row
            .try_get::<String, usize>(column.ordinal())
            .unwrap_or("NULL".to_string()),
        // .unwrap_or("NULL".to_string()),
        "DATE" | "DATETIME" | "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, usize>(column.ordinal())
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string()),
        "BLOB" | "LONGBLOB" | "MEDIUMBLOB" | "TINYBLOB" => {
            row.try_get::<Vec<u8>, usize>(column.ordinal())
                // 转为hex字符串
                .map(|v| bytes_to_hex_string(&v))
                .unwrap_or("NULL".to_string())
        }
        "VARBINARY" | "BINARY" => row
            .try_get::<Vec<u8>, usize>(column.ordinal())
            .map(|v| String::from_utf8_lossy(&v).to_string())
            .unwrap_or("NULL".to_string()),

        _ => "NULL".to_string(),
    }
}

/// byte 转为 hex
fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}
