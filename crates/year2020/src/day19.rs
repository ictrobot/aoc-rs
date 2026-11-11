use utils::array::ArrayVec;
use utils::prelude::*;

/// Matching messages against recursive rules.
///
/// The key optimization is that both sequences in a rule have the same length, so if the first
/// matches, the second doesn't need checking as it would end at the same index.
#[derive(Clone, Debug)]
pub struct Day19<'a> {
    rules: Vec<Option<Rule>>,
    messages: Vec<&'a [u8]>,
}

#[derive(Clone, Debug)]
enum Rule {
    Letter(u8),
    Sequence(Sequence),
    Or(Sequence, Sequence),
}

#[derive(Clone, Copy, Debug)]
enum State {
    Unvisited,
    CurrentlyVisiting,
    Visited(u32),
}

const MAX_RULES: usize = 256;
const MAX_SEQUENCE_LENGTH: usize = 3;
type Sequence = ArrayVec<u8, MAX_SEQUENCE_LENGTH>;

impl<'a> Day19<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let sequence = parser::u8().repeat_arrayvec(b' ', 1);
        let rule = parser::parse_tree!(
            (b'"', letter @ parser::byte_range(b'a'..=b'b'), b'"') => Rule::Letter(letter),
            (l @ sequence) =>> {
                (" | ", r @ sequence) => Rule::Or(l, r),
                (parser::noop()) => Rule::Sequence(l),
            },
        );
        let (parsed, messages) = parser::u8()
            .with_suffix(": ")
            .then(rule)
            .repeat(parser::eol(), 1)
            .with_eol()
            .with_eol()
            .then(parser::take_while1(|&b| matches!(b, b'a' | b'b')).repeat(parser::eol(), 1))
            .parse_complete(input)?;

        let mut rules = vec![None; MAX_RULES];
        for (id, rule) in parsed {
            let id = usize::from(id);
            if rules[id].is_some() {
                return Err(InputError::new(input, 0, "duplicate rule"));
            }
            rules[id] = Some(rule);
        }
        if rules[0].is_none() {
            return Err(InputError::new(input, 0, "missing rule 0"));
        }

        let mut states = [State::Unvisited; MAX_RULES];
        for id in (0..MAX_RULES).filter(|&id| rules[id].is_some()) {
            Self::check(&rules, id, &mut states)
                .map_err(|error| InputError::new(input, 0, error))?;
        }

        Ok(Self { rules, messages })
    }

    fn check(rules: &[Option<Rule>], id: usize, states: &mut [State]) -> Result<u32, &'static str> {
        match states[id] {
            State::Unvisited => states[id] = State::CurrentlyVisiting,
            State::CurrentlyVisiting => return Err("rules contain a cycle"),
            State::Visited(length) => return Ok(length),
        }

        let mut sequence_length = |ids| {
            let mut total = 0u32;
            for &child in ids {
                let child = usize::from(child);
                if rules[child].is_none() {
                    return Err("undefined rule");
                }
                let length = Self::check(rules, child, states)?;
                total = total.checked_add(length).ok_or("rule too long")?;
            }
            Ok(total)
        };

        let result = match &rules[id] {
            Some(Rule::Letter(_)) => 1,
            Some(Rule::Sequence(s)) => sequence_length(s)?,
            Some(Rule::Or(l, r)) => match (sequence_length(l)?, sequence_length(r)?) {
                (l, r) if l == r => l,
                _ => return Err("expected consistent sequence lengths"),
            },
            None => unreachable!("input already validated"),
        };

        states[id] = State::Visited(result);
        Ok(result)
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.messages
            .iter()
            .filter(|message| self.match_rule(0, message, 0) == Some(message.len()))
            .count() as u64
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        assert!(
            matches!(&self.rules[0], Some(Rule::Sequence(s)) if s[..] == [8, 11])
                && matches!(&self.rules[8], Some(Rule::Sequence(s)) if s[..] == [42])
                && matches!(&self.rules[11], Some(Rule::Sequence(s)) if s[..] == [42, 31]),
            "part 2 requires rules '0: 8 11', '8: 42' and '11: 42 31'"
        );

        // Rule 8 is one or more 42s and rule 11 is N 42s followed by N 31s, so messages match if
        // they are 42s followed by fewer, but at least one, 31s
        self.messages
            .iter()
            .filter(|message| {
                let (mut start, mut count42) = (0, 0);
                while let Some(next) = self.match_rule(42, message, start) {
                    start = next;
                    count42 += 1;

                    let (mut index, mut count31) = (start, 0);
                    while let Some(next) = self.match_rule(31, message, index) {
                        index = next;
                        count31 += 1;
                    }
                    if index == message.len() && (1..count42).contains(&count31) {
                        return true;
                    }
                }
                false
            })
            .count() as u64
    }

    fn match_rule(&self, id: u8, message: &[u8], index: usize) -> Option<usize> {
        let match_sequence = |ids| {
            let mut index = index;
            for &id in ids {
                index = self.match_rule(id, message, index)?;
            }
            Some(index)
        };

        match &self.rules[usize::from(id)] {
            Some(Rule::Letter(letter)) => (message.get(index) == Some(letter)).then_some(index + 1),
            Some(Rule::Sequence(s)) => match_sequence(s),
            Some(Rule::Or(l, r)) => match_sequence(l).or_else(|| match_sequence(r)),
            None => unreachable!("input already validated"),
        }
    }
}

examples!(Day19<'_> -> (u64, u64) [
    {file: "day19_example0.txt", part1: 2},
    {file: "day19_example1.txt", part1: 3, part2: 12},
]);
