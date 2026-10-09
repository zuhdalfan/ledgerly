//! Text measurement and wrapping from the embedded font's own metrics, so the
//! PDF looks the same on every machine.

use ttf_parser::Face;

pub struct TextMeasure {
    face: Face<'static>,
    units_per_em: f32,
}

impl TextMeasure {
    pub fn new(data: &'static [u8]) -> Option<Self> {
        let face = Face::parse(data, 0).ok()?;
        let units_per_em = face.units_per_em() as f32;
        Some(Self {
            face,
            units_per_em: if units_per_em > 0.0 {
                units_per_em
            } else {
                1000.0
            },
        })
    }

    pub fn width_pt(&self, text: &str, size_pt: f32) -> f32 {
        let units: f32 = text
            .chars()
            .map(|character| {
                self.face
                    .glyph_index(character)
                    .and_then(|glyph| self.face.glyph_hor_advance(glyph))
                    .unwrap_or(0) as f32
            })
            .sum();
        units / self.units_per_em * size_pt
    }

    pub fn width_mm(&self, text: &str, size_pt: f32) -> f32 {
        self.width_pt(text, size_pt) * 25.4 / 72.0
    }

    /// Break text to fit `max_mm`, honouring explicit newlines. Words longer than
    /// the line are kept whole rather than split.
    pub fn wrap(&self, text: &str, size_pt: f32, max_mm: f32) -> Vec<String> {
        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            if paragraph.trim().is_empty() {
                lines.push(String::new());
                continue;
            }

            let mut current = String::new();
            for word in paragraph.split_whitespace() {
                let candidate = if current.is_empty() {
                    word.to_string()
                } else {
                    format!("{current} {word}")
                };

                if current.is_empty() || self.width_mm(&candidate, size_pt) <= max_mm {
                    current = candidate;
                } else {
                    lines.push(std::mem::take(&mut current));
                    current = word.to_string();
                }
            }

            if !current.is_empty() {
                lines.push(current);
            }
        }

        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");

    #[test]
    fn measures_wider_text_as_wider() {
        let measure = TextMeasure::new(FONT).unwrap();
        assert!(measure.width_mm("hello", 12.0) > 0.0);
        assert!(measure.width_mm("hello world", 12.0) > measure.width_mm("hello", 12.0));
    }

    #[test]
    fn wraps_to_fit_the_available_width() {
        let measure = TextMeasure::new(FONT).unwrap();
        let lines = measure.wrap(
            "Consulting services for the redesign of the marketing website",
            9.0,
            40.0,
        );

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(
                measure.width_mm(line, 9.0) <= 40.0,
                "line too wide: {line:?}"
            );
        }
    }

    #[test]
    fn keeps_long_words_whole() {
        let measure = TextMeasure::new(FONT).unwrap();
        let lines = measure.wrap("supercalifragilisticexpialidocious", 9.0, 10.0);
        assert_eq!(
            lines,
            vec!["supercalifragilisticexpialidocious".to_string()]
        );
    }
}
