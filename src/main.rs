use eframe::egui;
use rand::seq::SliceRandom;

const WINDOW_W: f32 = 480.0;
const WINDOW_H: f32 = 400.0;

static WORDLIST: &str = include_str!("words.txt");

fn words() -> Vec<&'static str> {
    WORDLIST.lines().filter(|l| !l.is_empty()).collect()
}

fn apply_substitutions(word: &str, numbers: bool, symbols: bool) -> String {
    word.chars()
        .map(|c| match c {
            'e' if numbers => '3',
            'o' if numbers => '0',
            'a' if numbers && !symbols => '4',
            'l' if numbers => '1',
            'a' | 'A' if symbols => '@',
            's' | 'S' if symbols => '$',
            'i' | 'I' if symbols => '!',
            _ => c,
        })
        .collect()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn generate_passphrase(
    pool: &[&'static str],
    word_count: usize,
    numbers: bool,
    symbols: bool,
    uppercase: bool,
) -> String {
    let mut rng = rand::thread_rng();
    pool.choose_multiple(&mut rng, word_count)
        .map(|w| {
            let mut word = apply_substitutions(w, numbers, symbols);
            if uppercase {
                word = capitalize(&word);
            }
            word
        })
        .collect::<Vec<_>>()
        .join("-")
}

struct App {
    pool: Vec<&'static str>,
    word_count: usize,
    phrase_count: usize,
    numbers: bool,
    symbols: bool,
    uppercase: bool,
    output: String,
}

impl App {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            pool: words(),
            word_count: 4,
            phrase_count: 5,
            numbers: false,
            symbols: false,
            uppercase: false,
            output: String::new(),
        }
    }

    fn generate(&mut self) {
        self.output = (0..self.phrase_count)
            .map(|_| {
                generate_passphrase(
                    &self.pool,
                    self.word_count,
                    self.numbers,
                    self.symbols,
                    self.uppercase,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PassPhrase Generator");
            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.numbers, "Numbers (leet)");
                ui.checkbox(&mut self.symbols, "Symbols (leet)");
                ui.checkbox(&mut self.uppercase, "Uppercase");
            });

            ui.add(egui::Slider::new(&mut self.word_count, 2..=12).text("Words per phrase"));
            ui.add(egui::Slider::new(&mut self.phrase_count, 1..=20).text("Number of phrases"));

            ui.add_space(4.0);
            if ui.button("Generate").clicked() {
                self.generate();
            }

            if !self.output.is_empty() {
                ui.separator();
                let row_count = self.phrase_count.max(1);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.output)
                            .desired_width(f32::INFINITY)
                            .desired_rows(row_count)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PassPhrase Generator",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([WINDOW_W, WINDOW_H]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_word_count() {
        let pool = words();
        let phrase = generate_passphrase(&pool, 4, false, false, false);
        assert_eq!(phrase.split('-').count(), 4);
    }

    #[test]
    fn wordlist_loaded() {
        assert!(words().len() >= 7000);
    }

    #[test]
    fn number_substitutions_applied() {
        // active rules: e->3, l->1 (i and t subs were removed)
        assert_eq!(apply_substitutions("elite", true, false), "31it3");
    }

    #[test]
    fn symbol_substitutions_applied() {
        assert_eq!(apply_substitutions("sail", false, true), "$@!l");
    }

    #[test]
    fn uppercase_applied() {
        let pool = words();
        let phrase = generate_passphrase(&pool, 4, false, false, true);
        for part in phrase.split('-') {
            let first = part.chars().next().unwrap();
            assert!(first.is_uppercase() || !first.is_alphabetic());
        }
    }

    #[test]
    fn no_duplicate_words_in_phrase() {
        let pool = words();
        for _ in 0..20 {
            let phrase = generate_passphrase(&pool, 6, false, false, false);
            let parts: Vec<_> = phrase.split('-').collect();
            let unique: std::collections::HashSet<_> = parts.iter().collect();
            assert_eq!(parts.len(), unique.len());
        }
    }
}
