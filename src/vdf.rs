#[cfg(test)] // "sorted" hashmap for testing
use std::collections::BTreeMap;
#[cfg(not(test))]
use std::collections::HashMap;

//
#[cfg(not(test))]
pub type Vdf<'s> = HashMap<&'s str, VdfValue<'s>>;
#[cfg(test)] // "sorted" hashmap for testing
pub type Vdf<'s> = BTreeMap<&'s str, VdfValue<'s>>;

#[derive(Debug, Clone)]
pub enum VdfValue<'s> {
    Value(&'s str),
    Map(Vdf<'s>),
}

pub struct VdfParser<'a> {
    s: &'a str,
}

//

impl<'a> VdfValue<'a> {
    pub fn as_value(&self) -> Option<&'a str> {
        match self {
            VdfValue::Value(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&'_ Vdf<'a>> {
        match self {
            VdfValue::Map(map) => Some(map),
            _ => None,
        }
    }
}

impl<'a> VdfParser<'a> {
    pub fn from_str(s: &'a str) -> Self {
        Self { s }
    }

    pub fn parse_entries(&mut self) -> Option<Vdf<'a>> {
        Some(std::iter::from_fn(|| self.parse_entry()).collect())
    }

    pub fn parse_entry(&mut self) -> Option<(&'a str, VdfValue<'a>)> {
        Some((self.parse_lit_str()?, self.parse_value()?))
    }

    pub fn parse_value(&mut self) -> Option<VdfValue<'a>> {
        if let Some(val) = self.parse_lit_str() {
            Some(VdfValue::Value(val))
        } else {
            self.parse_map().map(VdfValue::Map)
        }
    }

    pub fn parse_map(&mut self) -> Option<Vdf<'a>> {
        let initial = self.s;
        if let Some(vdf) = (|| {
            self.parse_char('{')?;
            let vdf = self.parse_entries()?;
            self.parse_char('}')?;
            Some(vdf)
        })() {
            Some(vdf)
        } else {
            self.s = initial;
            None
        }
    }

    pub fn parse_lit_str(&mut self) -> Option<&'a str> {
        self.parse_char('"')?;

        let (lhs, rhs) = self.s.split_once('"')?;
        // the " is consumed by split_once
        self.s = rhs;

        Some(lhs)
    }

    pub fn parse_char(&mut self, ch: char) -> Option<()> {
        let (lhs, rhs) = self.split_char()?;
        if lhs == ch {
            self.s = rhs;
            Some(())
        } else {
            None
        }
    }

    pub fn split_char(&self) -> Option<(char, &'a str)> {
        let mut chars = self.s.trim_start().chars();
        let ch = chars.next()?;
        Some((ch, chars.as_str()))
    }
}

//

#[cfg(test)]
mod tests {
    use super::VdfParser;

    #[test]
    fn simple_vdf_parse_test() {
        let s = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/home"
		"label"		""
		"contentid"		"0"
		"totalsize"		"0"
		"update_clean_bytes_tally"		"0"
		"time_last_update_corruption"		"0"
		"apps"
		{
        }
    }
}"#;

        let vdf = VdfParser::from_str(s).parse_entries();
        insta::assert_debug_snapshot!(vdf);
    }
}
