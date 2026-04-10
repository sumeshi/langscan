use crate::lang::Lang;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchMode {
    Matched,
    Unmatched,
}

#[derive(Default, Debug)]
pub struct Flags {
    pub has_cjk: bool,
    pub has_cyr: bool,
    pub has_hangul: bool,
    pub has_vietnamese: bool,
    pub has_thai: bool,
    pub has_turkish: bool,
    pub has_ukrainian: bool,
    pub has_arabic: bool,
    pub has_persian: bool,
    pub has_hebrew: bool,
    pub has_devanagari: bool,
    pub has_greek: bool,
    pub has_polish: bool,
    pub has_urdu: bool,
    pub has_japanese: bool,
    pub has_simplified_marker: bool,
    pub has_traditional_marker: bool,
}

#[derive(Default, Debug)]
pub struct ScanPlan {
    pub need_cjk: bool,
    pub need_cyr: bool,
    pub need_hangul: bool,
    pub need_vietnamese: bool,
    pub need_thai: bool,
    pub need_turkish: bool,
    pub need_ukrainian: bool,
    pub need_arabic: bool,
    pub need_persian: bool,
    pub need_hebrew: bool,
    pub need_devanagari: bool,
    pub need_greek: bool,
    pub need_polish: bool,
    pub need_urdu: bool,
    pub need_japanese: bool,
    pub need_simplified_marker: bool,
    pub need_traditional_marker: bool,
}

impl ScanPlan {
    pub fn from_langs(langs: &[Lang]) -> Self {
        let mut plan = Self::default();
        for lang in langs {
            match lang {
                Lang::Cjk => plan.need_cjk = true,
                Lang::ZhHans => {
                    plan.need_cjk = true;
                    plan.need_simplified_marker = true;
                }
                Lang::ZhHant => {
                    plan.need_cjk = true;
                    plan.need_traditional_marker = true;
                }
                Lang::Ru => plan.need_cyr = true,
                Lang::Ko | Lang::KoKp | Lang::KoKr => plan.need_hangul = true,
                Lang::Ja => {
                    plan.need_cjk = true;
                    plan.need_japanese = true;
                }
                Lang::Vi => plan.need_vietnamese = true,
                Lang::Th => plan.need_thai = true,
                Lang::Tr => plan.need_turkish = true,
                Lang::Uk => plan.need_ukrainian = true,
                Lang::Ar => plan.need_arabic = true,
                Lang::Fa => plan.need_persian = true,
                Lang::He => plan.need_hebrew = true,
                Lang::Hi => plan.need_devanagari = true,
                Lang::El => plan.need_greek = true,
                Lang::Pl => plan.need_polish = true,
                Lang::Ur => plan.need_urdu = true,
            }
        }
        plan
    }

    pub fn is_complete(&self, flags: &Flags) -> bool {
        (!self.need_cjk || flags.has_cjk)
            && (!self.need_cyr || flags.has_cyr)
            && (!self.need_hangul || flags.has_hangul)
            && (!self.need_japanese || flags.has_japanese)
            && (!self.need_vietnamese || flags.has_vietnamese)
            && (!self.need_thai || flags.has_thai)
            && (!self.need_turkish || flags.has_turkish)
            && (!self.need_ukrainian || flags.has_ukrainian)
            && (!self.need_arabic || flags.has_arabic)
            && (!self.need_persian || flags.has_persian)
            && (!self.need_hebrew || flags.has_hebrew)
            && (!self.need_devanagari || flags.has_devanagari)
            && (!self.need_greek || flags.has_greek)
            && (!self.need_polish || flags.has_polish)
            && (!self.need_urdu || flags.has_urdu)
            && (!self.need_simplified_marker || flags.has_simplified_marker)
            && (!self.need_traditional_marker || flags.has_traditional_marker)
    }
}
