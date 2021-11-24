//! Character-classes
type CharClassRange = std::ops::RangeInclusive<char>;

/// Representation of a character-class
#[derive(Clone, PartialEq)]
pub struct CharClass {
    ranges: Vec<CharClassRange>,
}

impl CharClass {
    /** Create new empty character class. */
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /** Retrieve total number of characters in class */
    pub fn len(&self) -> u32 {
        self.ranges
            .iter()
            .map(|r| *r.end() as u32 - *r.start() as u32 + 1)
            .sum()
    }

    /** Dump the character ranges */
    pub fn dump(&self) {
        println!("{:p} ranges={}", self, self.ranges.len());

        for i in 0..self.ranges.len() {
            println!("  {:?}", self.ranges[i]);
        }
    }

    /** Normalize character-class by removing intersections and coherent ranges. */
    pub fn normalize(&mut self) {
        let mut prev_count: usize = 0;

        while self.ranges.len() != prev_count {
            prev_count = self.ranges.len();

            // First sort all ranges
            self.ranges.sort_by(|a, b| a.start().cmp(b.start()));

            // Then look for intersections
            for i in 0..self.ranges.len() - 1 {
                let a = &self.ranges[i];
                let b = &self.ranges[i + 1];

                // Remove intersections
                if b.start() <= a.end() && b.end() >= a.start() {
                    if b.end() > a.end() {
                        self.ranges[i] = *a.start()..=*b.end();
                    }

                    self.ranges.remove(i + 1);
                    break;
                }
                // Merge coherent ranges
                else if *a.end() as u32 + 1 == *b.start() as u32 {
                    self.ranges[i] = *a.start()..=*b.end();
                    self.ranges.remove(i + 1);
                    break;
                }
            }
        }
    }

    /** Negate entire character class */
    pub fn negate(mut self) -> CharClass {
        let mut prev_count: usize = 0;
        let mut start = '\0';
        let mut end = '\0';

        while self.ranges.len() != prev_count {
            prev_count = self.ranges.len();

            for i in 0..self.ranges.len() {
                let irange = self.ranges[i].clone();

                if end < *irange.start() {
                    end = if *irange.start() > '\0' {
                        std::char::from_u32(*irange.start() as u32 - 1).unwrap()
                    } else {
                        '\0'
                    };

                    self.ranges[i] = start..=end;

                    start = if *irange.end() < std::char::MAX {
                        std::char::from_u32(*irange.end() as u32 + 1).unwrap()
                    } else {
                        std::char::MAX
                    };

                    end = start;
                } else {
                    end = if *irange.end() < std::char::MAX {
                        std::char::from_u32(*irange.end() as u32 + 1).unwrap()
                    } else {
                        std::char::MAX
                    };

                    self.ranges.remove(i);
                    break;
                }
            }
        }

        if end < std::char::MAX {
            self.ranges.push(end..=std::char::MAX);
        }

        self.normalize();
        self
    }

    /** Add range to character class. */
    pub fn add(&mut self, range: CharClassRange) -> u32 {
        let len = self.len();
        self.ranges.push(range);
        self.normalize();
        self.len() - len
    }

    /** Clears entire range to be empty. */
    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    /** Test */
    pub fn test(&self, range: &CharClassRange) -> bool {
        self.ranges
            .binary_search_by(|r| {
                if r.start() > range.end() {
                    std::cmp::Ordering::Greater
                } else if r.end() < range.start() {
                    std::cmp::Ordering::Less
                } else {
                    if range.start() >= r.start() && range.end() <= r.end() {
                        std::cmp::Ordering::Equal
                    } else {
                        std::cmp::Ordering::Less // fixme: Is here also a Greater-case?
                    }
                }
            })
            .is_ok()
    }

    /** Does this range fit all chars? */
    fn is_any(&self) -> bool {
        self.ranges.len() == 1
            && *self.ranges[0].start() == 0 as char
            && *self.ranges[0].end() == std::char::MAX
    }
}

impl std::fmt::Debug for CharClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn escape(ch: char) -> String {
            match ch {
                '\x07' => "\\a".to_string(),
                '\x08' => "\\b".to_string(),
                '\x0c' => "\\f".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\x0b' => "\\v".to_string(),
                '\\' => "\\\\".to_string(),
                _ => format!("{}", ch),
            }
        }

        if self.is_any() {
            write!(f, ".")?;
        } else {
            write!(f, "[")?;
            for range in &self.ranges {
                if range.start() < range.end() {
                    write!(f, "{}-{}", escape(*range.start()), escape(*range.end()))?;
                } else {
                    write!(f, "{}", escape(*range.start()))?;
                }
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

impl PartialOrd for CharClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.ranges.len() == other.ranges.len() {
            for (mine, other) in self.ranges.iter().zip(other.ranges.iter()) {
                if other.end() > mine.end() || other.start() > mine.start() {
                    return Some(std::cmp::Ordering::Less);
                } else if other.end() < mine.end() {
                    return Some(std::cmp::Ordering::Greater);
                }
            }

            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

impl std::ops::Add for CharClass {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        for range in &other.ranges {
            self.ranges.push(range.clone());
        }

        self.normalize();
        self
    }
}

impl std::ops::AddAssign for CharClass {
    fn add_assign(&mut self, other: Self) {
        for range in &other.ranges {
            self.ranges.push(range.clone());
        }

        self.normalize();
    }
}

// todo: std::ops::Sub is not implemented yet but might be interesting ;)

/** Character-class construction helper-macro

Example:
```
use charclass::charclass;

let ccl = charclass!['A' => 'Z', 'a' => 'z'] + charclass!['_'];
```
*/
#[macro_export]
macro_rules! charclass {
    ( $( $from:expr => $to:expr ),+ ) => {
        {
            let mut ccl = $crate::CharClass::new();
            $( ccl.add($from..=$to); )*
            ccl
        }
    };

    ( $( $chr:expr ),+ ) => {
        {
            let mut ccl = $crate::CharClass::new();
            $( ccl.add($chr..=$chr); )*
            ccl
        }
    };
}

pub fn ccl_test() {
    let mut ccl = CharClass::new();
    ccl.add('a'..='c');
    ccl.add('€'..='€');
    ccl.add('k'..='v');
    ccl.normalize();
    //ccl.dump();
    //ccl.negate();
    ccl.dump();

    //ccl.add('a'..='z');
    ccl.normalize();
    ccl.dump();

    for c in b'a'..=b'z' {
        let c = char::from(c);
        println!("{}: {}", c, ccl.test(&(c..=c)));
    }

    for rg in vec!['k'..='v', 'l'..='o', 'a'..='d', 'a'..='b', 'k'..='x'] {
        println!("{:?} {}", &rg, ccl.test(&rg));
    }

    let mut t = CharClass::new();
    t.add('A'..='D');

    ccl += t;
    ccl.dump();
}
