use time::format_description::{self, BorrowedFormatItem};

pub fn date() -> Vec<BorrowedFormatItem<'static>> {
    format_description::parse("[year]-[month]-[day]").expect("modifiers are valid")
}

pub fn offset() -> Vec<BorrowedFormatItem<'static>> {
    format_description::parse("[offset_hour sign:mandatory]:[offset_minute]")
        .expect("modifiers are valid")
}

pub fn time() -> Vec<BorrowedFormatItem<'static>> {
    format_description::parse("[hour]:[minute]:[second]").expect("modifiers are valid")
}
