use std::time::Duration;

use amane::Service;

// the title of the focused window, from niri's event stream
pub struct Focus {
    pub title: String,
}

/// the string value of `"key":"..."` in a line of JSON, with its escapes undone
fn json_string(text: &str, key: &str) -> Option<String> {
    let start = text.find(&format!("\"{key}\":\""))? + key.len() + 4;

    let mut result = String::new();
    let mut characters = text[start..].chars();

    while let Some(letter) = characters.next() {
        match letter {
            '"' => return Some(result),
            '\\' => match characters.next()? {
                // control characters have no place in a one-line title
                'n' | 't' | 'r' | 'b' | 'f' => result.push(' '),
                'u' => {
                    let code = unicode(&mut characters)?;

                    // a character outside the basic plane arrives as two \u escapes
                    if (0xD800..0xDC00).contains(&code) {
                        let low = (characters.next() == Some('\\') && characters.next() == Some('u')).then(|| unicode(&mut characters)).flatten()?;

                        result.push(char::from_u32(0x10000 + ((code - 0xD800) << 10) + (low.wrapping_sub(0xDC00) & 0x3FF))?);
                    } else {
                        result.push(char::from_u32(code)?);
                    }
                }
                other => result.push(other),
            },
            other => result.push(other),
        }
    }

    None
}

fn unicode(characters: &mut std::str::Chars) -> Option<u32> {
    let digits: String = characters.take(4).collect();

    u32::from_str_radix(&digits, 16).ok()
}

fn read() -> String {
    // `"title":null` (a window without one) doesn't match, which is an empty title
    json_string(&amane::output("niri msg --json focused-window"), "title").unwrap_or_default()
}

impl Service for Focus {
    fn new() -> Self {
        Self { title: String::new() }
    }

    /*
     * niri says a window changed far more often than the focused one's title does (every terminal prompt, every
     * loading tab), so a reader thread only notes the events that can matter, and this thread sleeps until it hears
     * one, waits a tenth of a second for the rest of a burst, and reads the title once. nothing wakes while nothing
     * happens. the stream is started again if it ends (niri restarted)
     */
    fn listen() {
        let (noted, events) = std::sync::mpsc::channel::<()>();

        // the first look
        let _ = noted.send(());

        std::thread::spawn(move || {
            loop {
                for line in amane::lines("niri msg --json event-stream") {
                    // a changed window matters only if it's the focused one
                    let relevant = line.contains("WindowFocusChanged")
                        || line.contains("WindowClosed")
                        || line.contains("WorkspaceActivated")
                        || (line.contains("WindowOpenedOrChanged") && line.contains("\"is_focused\":true"));

                    if relevant && noted.send(()).is_err() {
                        return;
                    }
                }

                // the stream ended: wait for niri, then read the title again
                std::thread::sleep(Duration::from_secs(2));

                if noted.send(()).is_err() {
                    return;
                }
            }
        });

        while events.recv().is_ok() {
            std::thread::sleep(Duration::from_millis(100));

            // everything that arrived meanwhile is the same burst
            while events.try_recv().is_ok() {}

            let title = read();

            if title != Self::read().title {
                Self::write().title = title;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::json_string;

    #[test]
    fn finds_the_key() {
        let line = r#"{"WindowFocusChanged":{"id":3,"title":"Terminal","app_id":"foot"}}"#;

        assert_eq!(json_string(line, "title").as_deref(), Some("Terminal"));
        assert_eq!(json_string(line, "app_id").as_deref(), Some("foot"));
        assert_eq!(json_string(line, "missing"), None);
    }

    #[test]
    fn null_is_not_a_string() {
        assert_eq!(json_string(r#"{"title":null}"#, "title"), None);
    }

    #[test]
    fn undoes_escapes() {
        assert_eq!(json_string(r#"{"title":"say \"hi\" \\ ok \/"}"#, "title").as_deref(), Some(r#"say "hi" \ ok /"#));
    }

    #[test]
    fn control_characters_become_spaces() {
        assert_eq!(json_string(r#"{"title":"a\nb\tc"}"#, "title").as_deref(), Some("a b c"));
    }

    #[test]
    fn unicode_escapes() {
        assert_eq!(json_string(r#"{"title":"caf\u00e9"}"#, "title").as_deref(), Some("caf\u{e9}"));
        // outside the basic plane: a surrogate pair
        assert_eq!(json_string(r#"{"title":"\ud83d\ude00!"}"#, "title").as_deref(), Some("\u{1f600}!"));
    }

    #[test]
    fn broken_input_is_none_not_a_panic() {
        assert_eq!(json_string(r#"{"title":"unfinished"#, "title"), None);
        assert_eq!(json_string(r#"{"title":"bad \u12"}"#, "title"), None);
        assert_eq!(json_string(r#"{"title":"lone \ud83d"}"#, "title"), None);
        assert_eq!(json_string(r#"{"title":"cut \"#, "title"), None);
    }
}
