use utils::array::ArrayVec;
use utils::bit::BitIterator;
use utils::prelude::*;

/// Matching messages against recursive rules.
///
/// The key optimization is that both sequences in a rule have the same length, so if the first
/// matches, the second doesn't need checking as it would end at the same index. Rules of up to 8
/// characters also store which patterns they match, so messages can be checked in a single lookup.
#[derive(Clone, Debug)]
pub struct Day19 {
    rules: Vec<Rule>,
    messages: Vec<(u128, u32)>,
}

#[derive(Clone, Debug, Default)]
struct Rule {
    kind: Option<Kind>,
    length: u32,
    patterns: [u64; 4],
}

#[derive(Clone, Debug)]
enum Kind {
    Letter(u8),
    Sequence(Sequence),
    Or(Sequence, Sequence),
}

#[derive(Clone, Copy, Debug)]
enum State {
    Unvisited,
    CurrentlyVisiting,
    Visited,
}

const MAX_RULES: usize = 256;
const MAX_SEQUENCE_LENGTH: usize = 3;
const MAX_PATTERN_LENGTH: u32 = 8;
const MAX_MESSAGE_LENGTH: usize = 128;
type Sequence = ArrayVec<u8, MAX_SEQUENCE_LENGTH>;

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let sequence = parser::u8().repeat_arrayvec(b' ', 1);
        let rule = parser::parse_tree!(
            (b'"', letter @ parser::byte_range(b'a'..=b'b'), b'"') => Kind::Letter(letter),
            (l @ sequence) =>> {
                (" | ", r @ sequence) => Kind::Or(l, r),
                (parser::noop()) => Kind::Sequence(l),
            },
        );
        let message = parser::take_while1(|&b| matches!(b, b'a' | b'b')).map_res(|message| {
            if message.len() > MAX_MESSAGE_LENGTH {
                return Err("message too long");
            }
            let bits = message
                .iter()
                .rev()
                .fold(0, |bits, &b| (bits << 1) | u128::from(b - b'a'));
            Ok((bits, message.len() as u32))
        });
        let (parsed, messages) = parser::u8()
            .with_suffix(": ")
            .then(rule)
            .repeat(parser::eol(), 1)
            .with_eol()
            .with_eol()
            .then(message.repeat(parser::eol(), 1))
            .parse_complete(input)?;

        let mut rules = vec![Rule::default(); MAX_RULES];
        for (id, kind) in parsed {
            let id = usize::from(id);
            if rules[id].kind.is_some() {
                return Err(InputError::new(input, 0, "duplicate rule"));
            }
            rules[id].kind = Some(kind);
        }
        if rules[0].kind.is_none() {
            return Err(InputError::new(input, 0, "missing rule 0"));
        }

        let mut states = [State::Unvisited; MAX_RULES];
        for id in 0..MAX_RULES {
            if rules[id].kind.is_some() {
                Self::check(&mut rules, id, &mut states)
                    .map_err(|error| InputError::new(input, 0, error))?;
            }
        }

        Ok(Self { rules, messages })
    }

    fn check(rules: &mut [Rule], id: usize, states: &mut [State]) -> Result<u32, &'static str> {
        match states[id] {
            State::Unvisited => states[id] = State::CurrentlyVisiting,
            State::CurrentlyVisiting => return Err("rules contain a cycle"),
            State::Visited => return Ok(rules[id].length),
        }

        let kind = rules[id].kind.clone();
        let mut sequence_length = |ids: &Sequence| {
            let (mut total, mut sequence_patterns) = (0u32, [0; 4]);
            for &child in ids {
                let child = usize::from(child);
                if rules[child].kind.is_none() {
                    return Err("undefined rule");
                }
                let length = Self::check(rules, child, states)?;
                let next = total.checked_add(length).ok_or("rule too long")?;
                let child_patterns = rules[child].patterns;
                sequence_patterns = Self::concat(sequence_patterns, total, child_patterns, length);
                total = next;
            }
            Ok((total, sequence_patterns))
        };

        let (length, patterns) = match kind {
            Some(Kind::Letter(letter)) => (1, [1 << (letter - b'a'), 0, 0, 0]),
            Some(Kind::Sequence(s)) => sequence_length(&s)?,
            Some(Kind::Or(l, r)) => match (sequence_length(&l)?, sequence_length(&r)?) {
                ((l, a), (r, b)) if l == r => (l, std::array::from_fn(|i| a[i] | b[i])),
                _ => return Err("expected consistent sequence lengths"),
            },
            None => unreachable!("input already validated"),
        };

        states[id] = State::Visited;
        rules[id].length = length;
        rules[id].patterns = patterns;
        Ok(length)
    }

    // Concatenating a suffix to every pattern adds suffix << length to each, shifting the whole set
    fn concat(patterns: [u64; 4], length: u32, suffixes: [u64; 4], suffix_length: u32) -> [u64; 4] {
        if length == 0 {
            return suffixes;
        }
        if length + suffix_length > MAX_PATTERN_LENGTH {
            return [0; 4];
        }

        // Patterns are >= 1 character, so suffixes are <= 7 characters and only use suffixes[..2]
        let mut result = [0; 4];
        for (i, &word) in suffixes[..2].iter().enumerate() {
            for (bit, _) in BitIterator::ones(word) {
                let offset = (i * 64 + bit as usize) << length;
                // Patterns fill a 2^length bit block, and offset is a whole number of blocks
                if length < 7 {
                    // Blocks are <= 2^6/64 bits, so moving a whole number of blocks keeps
                    // patterns[0] within a single u64
                    result[offset / 64] |= patterns[0] << (offset % 64);
                } else {
                    // Blocks are 2^7/128 bits, so moving a whole number of blocks (0 or 1) copies
                    // patterns[0] and patterns[1] without shifting
                    result[offset / 64] |= patterns[0];
                    result[offset / 64 + 1] |= patterns[1];
                }
            }
        }
        result
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.messages
            .iter()
            .filter(|&&(bits, len)| {
                len == self.rules[0].length && self.match_rule(0, bits, 0).is_some()
            })
            .count() as u64
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        assert!(
            matches!(&self.rules[0].kind, Some(Kind::Sequence(s)) if s[..] == [8, 11])
                && matches!(&self.rules[8].kind, Some(Kind::Sequence(s)) if s[..] == [42])
                && matches!(&self.rules[11].kind, Some(Kind::Sequence(s)) if s[..] == [42, 31]),
            "part 2 requires rules '0: 8 11', '8: 42' and '11: 42 31'"
        );
        let (rule42, rule31) = (&self.rules[42], &self.rules[31]);
        assert!(
            rule42.length <= MAX_PATTERN_LENGTH && rule31.length <= MAX_PATTERN_LENGTH,
            "part 2 requires rules 42 and 31 to match at most 8 characters"
        );

        // Rule 8 is one or more 42s and rule 11 is N 42s followed by N 31s, so messages match if
        // they are 42s followed by fewer, but at least one, 31s
        self.messages
            .iter()
            .filter(|&&(bits, len)| {
                let (mut start, mut count42) = (0, 0);
                while start < len && Self::match_pattern(rule42, bits, start) {
                    start += rule42.length;
                    count42 += 1;

                    let (mut index, mut count31) = (start, 0);
                    while index < len && Self::match_pattern(rule31, bits, index) {
                        index += rule31.length;
                        count31 += 1;
                    }
                    if index == len && (1..count42).contains(&count31) {
                        return true;
                    }
                }
                false
            })
            .count() as u64
    }

    fn match_rule(&self, id: u8, bits: u128, index: u32) -> Option<u32> {
        let rule = &self.rules[usize::from(id)];
        if rule.length <= MAX_PATTERN_LENGTH {
            return Self::match_pattern(rule, bits, index).then_some(index + rule.length);
        }

        let match_sequence = |ids| {
            let mut index = index;
            for &id in ids {
                index = self.match_rule(id, bits, index)?;
            }
            Some(index)
        };

        match &rule.kind {
            Some(Kind::Sequence(s)) => match_sequence(s),
            Some(Kind::Or(l, r)) => match_sequence(l).or_else(|| match_sequence(r)),
            Some(Kind::Letter(_)) | None => unreachable!("input already validated"),
        }
    }

    fn match_pattern(rule: &Rule, bits: u128, index: u32) -> bool {
        let pattern = (bits >> index) as usize & ((1 << rule.length) - 1);
        rule.patterns[pattern / 64] & (1 << (pattern % 64)) != 0
    }
}

examples!(Day19 -> (u64, u64) [
    {file: "day19_example0.txt", part1: 2},
    {file: "day19_example1.txt", part1: 3, part2: 12},
]);
