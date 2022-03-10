#[macro_use]
extern crate lazy_static;

use druid::widget::prelude::*;
use druid::widget::{Button, Flex, Label, Padding, Painter, SizedBox};
use druid::{
    AppDelegate, AppLauncher, Color, Data, DelegateCtx, Event, LocalizedString, TextAlignment,
    Widget, WidgetExt, WindowDesc, WindowId,
};

use druid::keyboard_types::Key::{Backspace, Character, Enter};

use rand::prelude::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Copy, Data)]
enum GuessResult {
    NotGuessed,
    Entered(char),
    NotInWord(char),
    InWord(char),
    Correct(char),
}

type KeyState = u8;

const CHAR_NOT_TRIED: KeyState = 0;
const CHAR_NOT_IN_WORD: KeyState = 1;
const CHAR_IN_WORD: KeyState = 2;
const CHAR_CORRECT: KeyState = 3;

lazy_static! {
    static ref LEGAL_GUESSES: HashSet<String> = {
        let mut it = HashSet::new();

        let file = File::open("resources/legal_guesses.txt").unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            it.insert(line.unwrap());
        }

        it
    };
}

lazy_static! {
    static ref TARGET_WORDS: HashSet<String> = {
        let mut it = HashSet::new();

        let file = File::open("resources/target_words.txt").unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            it.insert(line.unwrap());
        }

        it
    }; 
}

#[derive(Clone, Data)]
struct AppState {
    guesses: [[GuessResult; 5]; 6],
    current_guess: usize,
    current_letter: usize,
    target: String,
    guessed_letters: [[KeyState; 13]; 2],
}

impl AppState {
    fn new() -> AppState {
        let mut rng = rand::thread_rng();        

        let target_vec:Vec<&String> = TARGET_WORDS.iter().collect();

        let random_idx = rng.gen_range(0, target_vec.len());

        AppState {
            guesses: [[GuessResult::NotGuessed; 5]; 6],
            current_guess: 0,
            current_letter: 0,
            target: target_vec[random_idx].to_string(),
            guessed_letters: [[CHAR_NOT_TRIED; 13]; 2], // Druid max is 13, have to do silly things
        }
    }

    fn character_pressed(&mut self, key: char) {
        if self.current_guess < 6 && self.current_letter < 5 {
            self.guesses[self.current_guess][self.current_letter] = GuessResult::Entered(key);
            self.current_letter = self.current_letter + 1;
        }
    }

    fn backspace_pressed(&mut self) {
        if self.current_guess < 6 && self.current_letter > 0 {
            self.current_letter = self.current_letter - 1;
            self.guesses[self.current_guess][self.current_letter] = GuessResult::NotGuessed;
        }
    }

    fn index_in_target(&self, target: &Vec<char>, guess: char) -> Option<usize> {
        target.iter().position(|&item| item == guess)
    }

    fn key_state_indices(&self, character: char) -> (usize, usize) {
        let char_index = character as u8 - 65;
        ((char_index / 13) as usize, (char_index % 13) as usize)
    }

    fn update_key_state(&mut self, character: char, key_state: KeyState) {
        let (outer_index, inner_index) = self.key_state_indices(character);
        if key_state > self.guessed_letters[outer_index][inner_index] {
            self.guessed_letters[outer_index][inner_index] = key_state;
        }
    }

    fn get_key_state(&self, character: char) -> KeyState {
        let (outer_index, inner_index) = self.key_state_indices(character);
        self.guessed_letters[outer_index][inner_index]
    }

    fn process_guess(&mut self) {
        let mut target: Vec<char> = self.target.chars().collect();
        for i in 0..target.len() {
            if let GuessResult::Entered(guess) = self.guesses[self.current_guess][i] {
                if guess == target[i] {
                    self.guesses[self.current_guess][i] = GuessResult::Correct(guess);
                    self.update_key_state(guess, CHAR_CORRECT);
                    target[i] = ' ';
                }
            }
        }
        for i in 0..target.len() {
            if let GuessResult::Entered(guess) = self.guesses[self.current_guess][i] {
                if let Some(index) = self.index_in_target(&target, guess) {
                    self.guesses[self.current_guess][i] = GuessResult::InWord(guess);
                    self.update_key_state(guess, CHAR_IN_WORD);
                    target[index] = ' ';
                } else {
                    self.guesses[self.current_guess][i] = GuessResult::NotInWord(guess);
                    self.update_key_state(guess, CHAR_NOT_IN_WORD);
                }
            }
        }
    }

    fn enter_pressed(&mut self) {
        if self.current_letter > 4 {
            let guess_word: String = self.guesses[self.current_guess]
                .iter()
                .map(|&g| guess_letter(g))
                .collect();

            if LEGAL_GUESSES.contains(&guess_word) || TARGET_WORDS.contains(&guess_word) {
                self.process_guess();
                if self.current_guess < 6 {
                    self.current_guess = self.current_guess + 1;
                    self.current_letter = 0;
                }
            }
        }
    }
}

fn guess_letter(guess_result: GuessResult) -> char {
    match guess_result {
        GuessResult::Entered(alpha) => alpha,
        GuessResult::NotInWord(alpha) => alpha,
        GuessResult::InWord(alpha) => alpha,
        GuessResult::Correct(alpha) => alpha,
        _ => ' ',
    }
}

fn build_guess_grid(_app_state: &AppState) -> Flex<AppState> {
    let mut result = Flex::column();
    for guess in 0..6 {
        let mut row = Flex::row();
        for letter in 0..5 {
            let painter = Painter::<AppState>::new(move |ctx, data, _env| {
                let bounds = ctx.size().to_rect();

                let background = match data.guesses[guess][letter] {
                    GuessResult::NotGuessed => Color::rgb(0.1, 0.1, 0.1),
                    GuessResult::Entered(_) => Color::rgb(0.0, 0.0, 0.0),
                    GuessResult::NotInWord(_) => Color::rgb(0.2, 0.2, 0.2),
                    GuessResult::InWord(_) => Color::rgb(0.8, 0.6, 0.3),
                    GuessResult::Correct(_) => Color::rgb(0.0, 0.6, 0.3),
                };
                ctx.fill(bounds, &background);
            });

            row.add_child(Padding::new(
                4.0,
                SizedBox::new(
                    Label::dynamic(move |data: &AppState, _| {
                        guess_letter(data.guesses[guess][letter]).to_string()
                    })
                    .with_text_size(32.0)
                    .with_text_alignment(TextAlignment::Center)
                    .background(painter),
                )
                .width(48.0)
                .height(48.0),
            ))
        }
        result.add_child(row);
    }
    result
}

fn key_button(character: char) -> impl Widget<AppState> {
    let painter = Painter::<AppState>::new(move |ctx, data, _env| {
        let bounds = ctx.size().to_rect();

        let background = match data.get_key_state(character) {
            CHAR_NOT_IN_WORD => Color::rgb(0.2, 0.2, 0.2),
            CHAR_IN_WORD => Color::rgb(0.8, 0.6, 0.3),
            CHAR_CORRECT => Color::rgb(0.0, 0.6, 0.3),
            _ => Color::rgb(0.1, 0.1, 0.1),
        };

        ctx.fill(bounds, &background);
    });

    Padding::new(
        4.0,
        SizedBox::new(
            Label::new(character.to_string())
                .with_text_size(24.)
                .background(painter)
                .on_click(move |_ctx, data: &mut AppState, _env| data.character_pressed(character)),
        )
        .width(32.0)
        .height(32.0),
    )
}

fn build_keyboard() -> Flex<AppState> {
    let mut result = Flex::column();
    let mut row = Flex::row();
    for key_char in vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'] {
        row.add_child(key_button(key_char));
    }
    result.add_child(row);

    row = Flex::row();
    for key_char in vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'] {
        row.add_child(key_button(key_char));
    }
    result.add_child(row);

    row = Flex::row();
    row.add_child(
        Button::new("ENTER").on_click(move |_ctx, data: &mut AppState, _env| data.enter_pressed()),
    );
    for key_char in vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'] {
        row.add_child(key_button(key_char));
    }
    row.add_child(
        Button::new("\u{232B}")
            .on_click(move |_ctx, data: &mut AppState, _env| data.backspace_pressed()),
    );
    result.add_child(row);

    result
}

struct KeyBoardHandler;

impl AppDelegate<AppState> for KeyBoardHandler {
    fn event(
        &mut self,
        _ctx: &mut DelegateCtx<'_>,
        _window_id: WindowId,
        event: Event,
        data: &mut AppState,
        _env: &Env,
    ) -> Option<Event> {
        match &event {
            Event::KeyDown(e) => {
                let digit = &e.key;
                match digit {
                    Character(payload) => {
                        if let Some(upper) = payload.to_ascii_uppercase().chars().next() {
                            if upper.is_alphabetic() {
                                data.character_pressed(upper)
                            }
                        }
                    }
                    Backspace => data.backspace_pressed(),
                    Enter => data.enter_pressed(),
                    _ => (),
                }
            }
            _ => (),
        };
        Option::from(event)
    }
}

pub fn main() {
    let app_state = AppState::new();

    let guess_grid = build_guess_grid(&app_state);

    AppLauncher::with_window(
        WindowDesc::new(move || {
            Flex::column()
                .with_flex_spacer(0.1)
                .with_child(guess_grid)
                .with_flex_spacer(1.0)
                .with_child(build_keyboard())
        })
        .window_size((640., 600.))
        .resizable(false)
        .title(LocalizedString::new("app-title").with_placeholder("Wordle")),
    )
    .delegate(KeyBoardHandler)
    .use_simple_logger()
    .launch(app_state)
    .expect("launch failed");
}
