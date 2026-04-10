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
    Ar,
    Cjk,
    El,
    Fa,
    He,
    Hi,
    Ja,
    Ko,
    KoKp,
    KoKr,
    Ru,
    Th,
    Ur,
    Vi,
    ZhHans,
    ZhHant,
}

impl std::str::FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ar" | "ara" | "arabic" => Ok(Lang::Ar),
            "cjk" => Ok(Lang::Cjk),
            "el" | "ell" | "greek" | "gr" => Ok(Lang::El),
            "fa" | "fas" | "per" | "persian" | "farsi" => Ok(Lang::Fa),
            "he" | "heb" | "hebrew" => Ok(Lang::He),
            "hi" | "hin" | "hindi" => Ok(Lang::Hi),
            "ja" | "jpn" | "japanese" => Ok(Lang::Ja),
            "ko" | "kor" | "korean" => Ok(Lang::Ko),
            "ko-kp" | "kp" | "dprk" | "north-korea" => Ok(Lang::KoKp),
            "ko-kr" | "kr" | "rok" | "south-korea" => Ok(Lang::KoKr),
            "ru" | "rus" | "russian" => Ok(Lang::Ru),
            "th" | "tha" | "thai" => Ok(Lang::Th),
            "ur" | "urd" | "urdu" => Ok(Lang::Ur),
            "vi" | "vie" | "vietnamese" => Ok(Lang::Vi),
            "zh-cn" | "zh-hans" | "zh-hans-cn" | "zh_simplified" | "cn" => Ok(Lang::ZhHans),
            "zh-tw" | "zh-hant" | "zh-hant-tw" | "zh_traditional" | "tw" => Ok(Lang::ZhHant),
            other => Err(format!("unknown lang: {other}")),
        }
    }
}

pub fn lang_label(lang: Lang) -> &'static str {
    match lang {
        Lang::Ar => "ar",
        Lang::Cjk => "cjk",
        Lang::El => "el",
        Lang::Fa => "fa",
        Lang::He => "he",
        Lang::Hi => "hi",
        Lang::Ja => "ja",
        Lang::Ko => "ko",
        Lang::KoKp => "dprk",
        Lang::KoKr => "rok",
        Lang::Ru => "ru",
        Lang::Th => "th",
        Lang::Ur => "ur",
        Lang::Vi => "vi",
        Lang::ZhHans => "cn",
        Lang::ZhHant => "tw",
    }
}

pub fn load_keywords(entries: &[String]) -> io::Result<BTreeMap<Lang, Vec<String>>> {
    let mut map: BTreeMap<Lang, Vec<String>> = BTreeMap::new();

    // 1. Aggregate built-in keywords for each language
    for builtin in [korean::built_in_keywords(), chinese::built_in_keywords()] {
        for (lang, keywords) in builtin.into_iter() {
            map.entry(lang).or_default().extend(keywords);
        }
    }

    // 2. Load external keywords from entries
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
