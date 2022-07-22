mod tests;

use {
    regex::Regex,
    std::{
        collections::HashMap,
        fmt,
        fmt::{
            Display,
            Formatter,
        },
        ops::{
            Add,
            BitOr,
        },
    },
};

pub fn str(s: &str) -> Element {
    Element::new(ElementKind::String(s.to_string()))
}

pub fn char(pat: &str) -> Element {
    let regex = match Regex::new(&format!("[{}]", pat.replace("[", "\\[").replace("]", "\\]"))) {
        Ok(v) => v,
        Err(_) => panic!("Regex pattern is invalid."),
    };

    Element::new(ElementKind::CharacterClass(regex))
}

pub struct InternalRuleIdSet {
    pub rule_ids: Vec<String>,
}

pub struct ModuleId(pub String);

pub struct RuleId(pub String);

pub struct Module {
    pub submodules: HashMap<ModuleId, Module>,
    pub rules: HashMap<RuleId, Rule>,
}

pub struct Rule {
}

#[derive(Debug, PartialEq)]
pub struct Element {
    pub kind: ElementKind,
    pub lookahead_kind: LookaheadKind,
    pub loop_count: LoopRange,
}

impl Element {
    pub fn new(kind: ElementKind) -> Element {
        Element {
            kind: kind,
            lookahead_kind: LookaheadKind::None,
            loop_count: LoopRange::validate_new(1, Maxable::Specified(1)),
        }
    }

    fn set_lookahead_kind(mut self, kind: LookaheadKind) -> Element {
        if self.lookahead_kind != LookaheadKind::None {
            panic!("Lookahead kind is already set.");
        }

        self.lookahead_kind = kind;
        self
    }

    pub fn pos(self) -> Element {
        self.set_lookahead_kind(LookaheadKind::Positive)
    }

    pub fn neg(self) -> Element {
        self.set_lookahead_kind(LookaheadKind::Negative)
    }

    pub fn times(self, times: usize) -> Element {
        self.min_to_max(times, times)
    }

    pub fn min(mut self, min: usize) -> Element {
        self.loop_count = LoopRange::validate_new(min, Maxable::Max);
        self
    }

    pub fn min_to_max(mut self, min: usize, max: usize) -> Element {
        self.loop_count = LoopRange::validate_new(min, Maxable::Specified(max));
        self
    }

    fn to_choice_elements(self) -> Vec<Element> {
        if self.is_choice() {
            match self.kind {
                ElementKind::Choice(elems) => elems,
                _ => unreachable!(),
            }
        } else {
            vec![self]
        }
    }

    fn to_sequence_elements(self) -> Vec<Element> {
        if self.is_sequence() {
            match self.kind {
                ElementKind::Sequence(elems) => elems,
                _ => unreachable!(),
            }
        } else {
            vec![self]
        }
    }

    fn is_choice(&self) -> bool {
        match self.kind {
            ElementKind::Choice(_) => true,
            _ => false,
        }
    }

    fn is_sequence(&self) -> bool {
        match self.kind {
            ElementKind::Sequence(_) => true,
            _ => false,
        }
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.lookahead_kind, self.kind, self.loop_count)
    }
}

impl BitOr for Element {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut first = self.to_choice_elements();
        let mut second = rhs.to_choice_elements();
        first.append(&mut second);
        Element::new(ElementKind::Choice(first))
    }
}

impl Add for Element {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut first = self.to_sequence_elements();
        let mut second = rhs.to_sequence_elements();
        first.append(&mut second);
        Element::new(ElementKind::Sequence(first))
    }
}

#[derive(Debug, PartialEq)]
pub enum ElementKind {
    Choice(Vec<Element>),
    Sequence(Vec<Element>),
    // fix: Regex
    String(String),
    CharacterClass(Regex),
    Wildcard,
}

impl Display for ElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ElementKind::Choice(elems) => format!("({})", elems.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(" | ")),
            ElementKind::Sequence(elems) => format!("({})", elems.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(" + ")),
            ElementKind::String(value) => value.clone(),
            ElementKind::CharacterClass(regex) => regex.to_string(),
            ElementKind::Wildcard => "_".to_string(),
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub enum LookaheadKind {
    None,
    Positive,
    Negative,
}

impl Display for LookaheadKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            LookaheadKind::None => "",
            LookaheadKind::Positive => "&",
            LookaheadKind::Negative => "!",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub struct LoopRange {
    min: usize,
    max: Maxable<usize>,
}

impl LoopRange {
    pub fn validate_new(min: usize, max: Maxable<usize>) -> LoopRange {
        match max {
            Maxable::Specified(n) if min > n => panic!("Max of loop range is bigger than min."),
            _ => (),
        }

        LoopRange {
            min: min,
            max: max,
        }
    }
}

impl Display for LoopRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self.max {
            Maxable::Max => format!("{{{}-}}", self.min),
            Maxable::Specified(n) => if self.min == n {
                if self.min != 1 {
                    format!("{{{}}}", self.min)
                } else {
                    String::new()
                }
            } else {
                format!("{{{}-{}}}", self.min, n)
            },
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub enum Maxable<T> {
    Max,
    Specified(T),
}
