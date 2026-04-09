use std::collections::BTreeMap;
use std::io;

mod arabic;
mod chinese;
mod cjk;
mod cyrillic;
mod data;
mod greek;
mod hebrew;
mod hindi;
mod japanese;
mod korean;
mod persian;
mod thai;
mod urdu;
mod vietnamese;

pub use arabic::is_arabic;
pub use chinese::{is_simplified_marker, is_traditional_marker};
pub use cjk::is_cjk;
pub use cyrillic::is_cyrillic;
pub use greek::is_greek;
pub use hebrew::is_hebrew;
pub use hindi::is_devanagari;
pub use japanese::is_japanese;
pub use korean::is_hangul;
pub use persian::is_persian;
pub use thai::is_thai;
pub use urdu::is_urdu;
pub use vietnamese::is_vietnamese;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Lang {
    Cjk,
    ZhHans,
    ZhHant,
    Ru,
    Ko,
    KoKp,
    KoKr,
    Ja,
    Vi,
    Th,
    Ar,
    Fa,
    He,
    Hi,
    El,
    Ur,
}

impl std::str::FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cjk" => Ok(Lang::Cjk),
            "zh-cn" | "zh-hans" | "zh-hans-cn" | "zh_simplified" => Ok(Lang::ZhHans),
            "zh-tw" | "zh-hant" | "zh-hant-tw" | "zh_traditional" => Ok(Lang::ZhHant),
            "ru" | "rus" | "russian" => Ok(Lang::Ru),
            "ko" | "kor" | "korean" => Ok(Lang::Ko),
            "ko-kp" | "kp" | "dprk" | "north-korea" => Ok(Lang::KoKp),
            "ko-kr" | "kr" | "rok" | "south-korea" => Ok(Lang::KoKr),
            "ja" | "jpn" | "japanese" => Ok(Lang::Ja),
            "vi" | "vie" | "vietnamese" => Ok(Lang::Vi),
            "th" | "tha" | "thai" => Ok(Lang::Th),
            "ar" | "ara" | "arabic" => Ok(Lang::Ar),
            "fa" | "fas" | "per" | "persian" | "farsi" => Ok(Lang::Fa),
            "he" | "heb" | "hebrew" => Ok(Lang::He),
            "hi" | "hin" | "hindi" => Ok(Lang::Hi),
            "el" | "ell" | "greek" | "gr" => Ok(Lang::El),
            "ur" | "urd" | "urdu" => Ok(Lang::Ur),
            "cn" => Ok(Lang::ZhHans),
            "tw" => Ok(Lang::ZhHant),
            other => Err(format!("unknown lang: {other}")),
        }
    }
}

pub fn lang_label(lang: Lang) -> &'static str {
    match lang {
        Lang::Cjk => "cjk",
        Lang::ZhHans => "cn",
        Lang::ZhHant => "tw",
        Lang::Ru => "ru",
        Lang::Ko => "ko",
        Lang::KoKp => "dprk",
        Lang::KoKr => "rok",
        Lang::Ja => "ja",
        Lang::Vi => "vi",
        Lang::Th => "th",
        Lang::Ar => "ar",
        Lang::Fa => "fa",
        Lang::He => "he",
        Lang::Hi => "hi",
        Lang::El => "el",
        Lang::Ur => "ur",
    }
}

pub fn load_keywords(entries: &[String]) -> io::Result<BTreeMap<Lang, Vec<String>>> {
    let mut map = korean::built_in_keywords();
    for entry in entries {
        let (lang_str, word) = entry.split_once('=').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "keyword expects lang=word")
        })?;
        let lang = lang_str
            .parse::<Lang>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let trimmed = word.trim();
        if trimmed.is_empty() {
            continue;
        }
        map.entry(lang).or_default().push(trimmed.to_string());
    }
    Ok(map)
}
