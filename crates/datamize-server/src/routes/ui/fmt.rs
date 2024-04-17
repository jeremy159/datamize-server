use currency_rs::{Currency, CurrencyOpts};

pub fn num_to_currency(num: i64) -> String {
    let opts = CurrencyOpts::new().set_negative_pattern("(!#)");
    Currency::new_float(num as f64 / 1000_f64, Some(opts)).format()
}

pub fn num_to_percentage(num: f64) -> String {
    format!("{:.2}%", num * 100_f64)
}
