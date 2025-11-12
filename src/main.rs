use std::env;
use std::io::{self, Write};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Datelike, Utc, TimeZone, LocalResult};
use chrono_tz::Australia::Brisbane;

fn usage() {
    eprintln!("Usage: utc_time HH:MM");
    eprintln!("Usage: utc_time HH:MM dd-mm-yy|YYYY");
    std::process::exit(1);
}

fn map_two_digit_year(y: i32) -> i32 {
    if (0..=68).contains(&y) { 2000 + y }
    else if (69..=99).contains(&y) { 1900 + y }
    else { y }
}

fn fix_two_digit_year(ndt: NaiveDateTime) -> NaiveDateTime {
    let y = ndt.date().year();
    if (0..=99).contains(&y) {
        let ny = map_two_digit_year(y);
        let m = ndt.date().month();
        let d = ndt.date().day();
        let t = ndt.time();
        let date = NaiveDate::from_ymd_opt(ny, m, d).expect("mapped year should be valid");
        NaiveDateTime::new(date, t)
    } else {
        ndt
    }
}

fn parse_input(s: &str) -> Result<NaiveDateTime, String> {
    if let Ok(t) = NaiveTime::parse_from_str(s, "%H:%M") {
        let today_bne = Utc::now().with_timezone(&Brisbane).date_naive();
        return Ok(NaiveDateTime::new(today_bne, t));
    }

    let formats = [
        "%H:%M %d-%m-%Y", // 21:00 22-09-2025
        "%H:%M %d-%m-%y", // 21:00 22-09-25
        "%H:%M %d/%m/%Y", // 21:00 22/09/2025
        "%H:%M %d/%m/%y", // 21:00 22/09/25
    ];

    for fmt in formats {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(fix_two_digit_year(ndt));
        }
    }

    Err("Unrecognized format. Try: HH:MM or HH:MM dd-mm-yy|yyyy or HH:MM dd/mm/yy|yyyy".into())
}

fn main() {
    // Accept: one arg (possibly quoted) or two args (time and date)
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage()
    }
    let input = match args.len() {
        1 => args[0].clone(),
        2 => format!("{} {}", args[0], args[1]),
        _ => {
            eprintln!("Too many arguments; pass either one quoted string or two separate args");
            std::process::exit(2);
        }
    };

    let ndt = match parse_input(&input) {
        Ok(ndt) => ndt,
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(3);
        }
    };

    println!("Select conversion:");
    println!("  1) AEST (Brisbane) -> UTC");
    println!("  2) UTC -> AEST (Brisbane)");
    print!("Choice [1/2]: ");
    io::stdout().flush().expect("flush stdout");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("read choice");
    let choice = choice.trim();

    match choice {
        "1" => {
            // Treat input as Brisbane local -> convert to UTC
            match Brisbane.from_local_datetime(&ndt) {
                LocalResult::Single(local_dt) => {
                    let utc = local_dt.with_timezone(&Utc);
                    println!("UTC: {}", utc.to_rfc3339());
                    println!("Brisbane: {}", local_dt.format("%Y-%m-%d %H:%M %Z"));
                }
                LocalResult::None => {
                    eprintln!("Non-existent local time in Brisbane (unexpected without DST)");
                    std::process::exit(4);
                }
                LocalResult::Ambiguous(_, _) => {
                    eprintln!("Ambiguous local time in Brisbane (rare; no DST in Brisbane)");
                    std::process::exit(5);
                }
            }
        }
        "2" => {
            let utc = Utc.from_utc_datetime(&ndt);
            let bne = utc.with_timezone(&Brisbane);
            println!("Brisbane: {}", bne.format("%Y-%m-%d %H:%M %Z"));
            println!("UTC: {}", utc.to_rfc3339());
        }
        _ => {
            eprintln!("Invalid choice, expected '1' or '2'");
            std::process::exit(6);
        }
    }
}
