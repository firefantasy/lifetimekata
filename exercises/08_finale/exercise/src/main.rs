use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

impl<'a> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut text_left = text;
        let mut tokens = Vec::new();
        loop {
            if text_left.is_empty() {
                break;
            }
            if text_left.starts_with('.') {
                tokens.push(MatcherToken::WildCard);
                text_left = &text_left[1..];
            } else if text_left.starts_with('(') {
                let close_basket = text_left.find(')')?;
                let (opt_str, leftover) = text_left.split_at(close_basket);
                tokens.push(MatcherToken::OneOfText(opt_str[1..].split('|').collect()));
                text_left = &leftover[1..];
            } else {
                let first_wc = text_left.find('.').unwrap_or(text_left.len());
                let first_op = text_left.find('(').unwrap_or(text_left.len());
                let first = first_wc.min(first_op);
                tokens.push(MatcherToken::RawText(&text_left[..first]));
                text_left = &text_left[first..];
            }
        }
        Some(Matcher { text: text, tokens: tokens, most_tokens_matched: 0 })
    }
    
    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string<'b, 'c>(&'b mut self, string: &'c str) -> Vec<(&'b MatcherToken<'a>, &'c str)> {
        let mut text_left = string;
        let mut answer = Vec::new();
        'outer_loop: for token in &self.tokens {
            if text_left.is_empty() {
                break;
            }
            match token {
                MatcherToken::WildCard => {
                    let byte_offset = text_left.chars().next().unwrap().len_utf8();
                    answer.push((token, &text_left[..byte_offset]));
                    text_left = &text_left[byte_offset..];
                },
                MatcherToken::OneOfText(opts) => {
                    for opt in opts {
                        if text_left.starts_with(opt) {
                            answer.push((token, &text_left[..opt.len()]));
                            text_left = &text_left[opt.len()..];
                            continue 'outer_loop;
                        }
                    }
                    break;
                },
                MatcherToken::RawText(st) => {
                    if text_left.starts_with(st) {
                        answer.push((token, &text_left[..st.len()]));
                        text_left = &text_left[st.len()..];
                        continue;
                    }
                    break;
                }
            }
        }
        self.most_tokens_matched = answer.len();
        answer
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};
    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcde".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "e") // or '💪'
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
