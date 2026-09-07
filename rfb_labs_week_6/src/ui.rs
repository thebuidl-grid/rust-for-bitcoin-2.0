//! Terminal output helpers.
//!
//! Every command prints the same shape: a lowercase section heading, then
//! indented `label  value` rows aligned on a common column. Tables get the same
//! treatment. It is plain text on purpose so the output pastes cleanly into a
//! submission document.

use bitcoin::{Amount, FeeRate};

const LABEL_WIDTH: usize = 22;

pub fn heading(text: &str) {
    println!("\n{text}");
}

pub fn row(label: &str, value: impl std::fmt::Display) {
    println!("  {label:<LABEL_WIDTH$}{value}");
}

/// A row whose value is long enough that it should start on the next line.
pub fn block(label: &str, value: impl std::fmt::Display) {
    println!("  {label}");
    println!("    {value}");
}

pub fn note(text: impl std::fmt::Display) {
    println!("\n{text}");
}

pub fn blank() {
    println!();
}

pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect();
    println!("  {}", header_line.join("  ").trim_end());

    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:<width$}", c, width = widths[i]))
            .collect();
        println!("  {}", line.join("  ").trim_end());
    }
}

/// Satoshis with a BTC gloss, e.g. `100000 sat (0.00100000 BTC)`.
pub fn sats(amount: Amount) -> String {
    format!("{} sat ({:.8} BTC)", amount.to_sat(), amount.to_btc())
}

pub fn fee_rate(rate: FeeRate) -> String {
    // A fee rate lands on whole sat/vB only by accident, and rounding it up to
    // the next integer makes a 2.00 sat/vB transaction read as 3.
    let per_kwu = rate.to_sat_per_kwu();
    format!("{:.2} sat/vB ({per_kwu} sat/kwu)", per_kwu as f64 / 250.0)
}

pub fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub fn bytes(len: u64) -> String {
    if len < 1024 {
        format!("{len} B")
    } else {
        format!("{:.1} KiB", len as f64 / 1024.0)
    }
}
