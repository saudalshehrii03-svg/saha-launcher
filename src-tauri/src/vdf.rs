//! Minimal Valve KeyValues (text VDF) reader/writer for loginusers.vdf,
//! libraryfolders.vdf, appmanifest_*.acf and localconfig.vdf.
//! Keeps key order so files are written back unchanged except for our edits.

#[derive(Debug, Clone)]
pub enum Vdf {
    Str(String),
    Obj(Vec<(String, Vdf)>),
}

impl Vdf {
    /// Case-insensitive child lookup (Steam is inconsistent with "apps"/"Apps").
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        self.entries().iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Vdf> {
        match self {
            Vdf::Obj(v) => v.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }
    pub fn path(&self, keys: &[&str]) -> Option<&Vdf> {
        keys.iter().try_fold(self, |node, k| node.get(k))
    }
    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            Vdf::Obj(_) => None,
        }
    }
    pub fn entries(&self) -> &[(String, Vdf)] {
        match self {
            Vdf::Obj(v) => v,
            Vdf::Str(_) => &[],
        }
    }
    pub fn entries_mut(&mut self) -> Option<&mut Vec<(String, Vdf)>> {
        match self {
            Vdf::Obj(v) => Some(v),
            Vdf::Str(_) => None,
        }
    }
    /// Set (or append) a string value.
    pub fn set_str(&mut self, key: &str, val: &str) {
        if let Some(v) = self.get_mut(key) {
            *v = Vdf::Str(val.to_string());
        } else if let Vdf::Obj(v) = self {
            v.push((key.to_string(), Vdf::Str(val.to_string())));
        }
    }
}

enum Tok {
    S(String),
    Open,
    Close,
}

fn tokenize(src: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut it = src.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            c if c.is_whitespace() => {
                it.next();
            }
            '/' => {
                it.next();
                if it.peek() == Some(&'/') {
                    for c in it.by_ref() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
            }
            '{' => {
                it.next();
                out.push(Tok::Open);
            }
            '}' => {
                it.next();
                out.push(Tok::Close);
            }
            '"' => {
                it.next();
                let mut s = String::new();
                while let Some(c) = it.next() {
                    match c {
                        '"' => break,
                        '\\' => match it.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(o) => s.push(o),
                            None => {}
                        },
                        o => s.push(o),
                    }
                }
                out.push(Tok::S(s));
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = it.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                        break;
                    }
                    s.push(c);
                    it.next();
                }
                // Platform conditionals like [$WIN32] are ignored.
                if !s.starts_with('[') {
                    out.push(Tok::S(s));
                }
            }
        }
    }
    out
}

type Toks = std::iter::Peekable<std::vec::IntoIter<Tok>>;

fn parse_obj(toks: &mut Toks) -> Vec<(String, Vdf)> {
    let mut v = Vec::new();
    while let Some(t) = toks.next() {
        match t {
            Tok::Close => break,
            Tok::Open => {
                parse_obj(toks); // stray block, skip
            }
            Tok::S(key) => match toks.peek() {
                Some(Tok::Open) => {
                    toks.next();
                    v.push((key, Vdf::Obj(parse_obj(toks))));
                }
                Some(Tok::S(_)) => {
                    if let Some(Tok::S(val)) = toks.next() {
                        v.push((key, Vdf::Str(val)));
                    }
                }
                _ => {}
            },
        }
    }
    v
}

pub fn parse(src: &str) -> Vdf {
    let mut toks = tokenize(src).into_iter().peekable();
    Vdf::Obj(parse_obj(&mut toks))
}

pub fn read(path: &std::path::Path) -> Option<Vdf> {
    std::fs::read_to_string(path).ok().map(|s| parse(&s))
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_obj(entries: &[(String, Vdf)], depth: usize, out: &mut String) {
    let ind = "\t".repeat(depth);
    for (k, v) in entries {
        match v {
            Vdf::Str(s) => out.push_str(&format!("{ind}\"{}\"\t\t\"{}\"\n", esc(k), esc(s))),
            Vdf::Obj(c) => {
                out.push_str(&format!("{ind}\"{}\"\n{ind}{{\n", esc(k)));
                write_obj(c, depth + 1, out);
                out.push_str(&format!("{ind}}}\n"));
            }
        }
    }
}

pub fn to_string(root: &Vdf) -> String {
    let mut out = String::new();
    write_obj(root.entries(), 0, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let src = "\"users\"\n{\n\t\"7656\"\n\t{\n\t\t\"AccountName\"\t\t\"a\\\\b\"\n\t}\n}\n";
        let v = parse(src);
        assert_eq!(v.path(&["users", "7656"]).unwrap().str("accountname"), Some("a\\b"));
        assert_eq!(to_string(&v), src);
    }
}
