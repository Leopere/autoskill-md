#[derive(Clone, Debug, PartialEq)]
pub struct Readability {
    pub grade: f64,
    pub max_grade: f64,
    pub ok: bool,
}

pub fn check_readability(markdown: &str) -> Readability {
    let max_grade = 7.0;
    let grade = flesch_kincaid_grade(markdown);
    Readability {
        grade,
        max_grade,
        ok: grade <= max_grade,
    }
}

pub fn flesch_kincaid_grade(markdown: &str) -> f64 {
    let text = normalize(markdown);
    let sentences = text
        .chars()
        .filter(|char| matches!(char, '.' | '!' | '?'))
        .count()
        .max(1) as f64;
    let words = words(&text);
    if words.is_empty() {
        return 0.0;
    }
    let syllables: usize = words.iter().map(|word| count_syllables(word)).sum();
    let grade = 0.39 * (words.len() as f64 / sentences)
        + 11.8 * (syllables as f64 / words.len() as f64)
        - 15.59;
    round2(grade.max(0.0))
}

fn normalize(markdown: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;

    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        output.push_str(line);
        output.push(' ');
    }

    output
        .split_whitespace()
        .filter(|part| !part.starts_with("http://") && !part.starts_with("https://"))
        .collect::<Vec<_>>()
        .join(" ")
        .replace(['#', '>', '*', '_', '[', ']', '(', ')', '-', '`'], " ")
}

fn words(text: &str) -> Vec<String> {
    text.split(|char: char| !char.is_ascii_alphabetic() && char != '\'')
        .filter(|part| part.chars().any(|char| char.is_ascii_alphabetic()))
        .map(ToString::to_string)
        .collect()
}

fn count_syllables(word: &str) -> usize {
    let clean = word
        .to_ascii_lowercase()
        .chars()
        .filter(|char| char.is_ascii_alphabetic())
        .collect::<String>();
    if clean.is_empty() {
        return 0;
    }
    if clean.len() <= 3 {
        return 1;
    }

    let clean = clean.strip_suffix('e').unwrap_or(&clean);
    let mut count = 0;
    let mut last_was_vowel = false;
    for char in clean.chars() {
        let is_vowel = matches!(char, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
        if is_vowel && !last_was_vowel {
            count += 1;
        }
        last_was_vowel = is_vowel;
    }
    count.max(1)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
