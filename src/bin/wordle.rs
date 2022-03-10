
use druid::widget::prelude::*;
use druid::widget::{Button, Flex, Label, Padding, Painter, SizedBox};
use druid::{
    AppDelegate, AppLauncher, Data, DelegateCtx, Color, Event, LocalizedString, 
    TextAlignment, Widget, WidgetExt, WindowDesc, WindowId
};

use druid::keyboard_types::Key::{Backspace, Character, Enter};

#[derive(Clone, Copy, Data)]
enum GuessResult {
    NotGuessed,
    Entered(char),
    NotInWord(char),
    InWord(char),
    Correct(char)   
}

#[derive(Clone, Data)]
struct AppState {
    guesses: [[GuessResult; 5]; 6],
    current_guess: usize,
    current_letter: usize,
    target: String
}

impl AppState {
    fn new() -> AppState {
        AppState {
            guesses: [[GuessResult::NotGuessed; 5]; 6],
            current_guess: 0,
            current_letter: 0,
            target: String::from("FUBAR")
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

    fn index_in_target(&self, target: &Vec<char>, guess:char) -> Option<usize> {
        target.iter().position(|&item| item == guess)
    }

    fn validate_guess(&mut self) {
       let mut target:Vec<char> = self.target.chars().collect();
        for i in 0..target.len() {
            if let GuessResult::Entered(guess) = self.guesses[self.current_guess][i] {
                if guess == target[i] {
                    self.guesses[self.current_guess][i] = GuessResult::Correct(guess);   
                    target[i] = ' ';   
                } 
            }
        }
        for i in 0..target.len() {
            if let GuessResult::Entered(guess) = self.guesses[self.current_guess][i] {
                if let Some(index) = self.index_in_target(&target, guess) {
                    self.guesses[self.current_guess][i] = GuessResult::InWord(guess);
                    target[index] = ' ';
                } else {
                    self.guesses[self.current_guess][i] = GuessResult::NotInWord(guess);   
                }
            }
        }
    }

    fn enter_pressed(&mut self) {
        if self.current_letter > 4 {
            self.validate_guess();
            if self.current_guess < 6 {
                self.current_guess = self.current_guess + 1;
                self.current_letter = 0;
            }
        }
    }
}

fn guess_letter(guess_result:GuessResult) -> char {
    match guess_result {
        GuessResult::Entered(alpha) => alpha,
        GuessResult::NotInWord(alpha) => alpha,
        GuessResult::InWord(alpha) => alpha,
        GuessResult::Correct(alpha) => alpha,
        _ => ' '
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
                    GuessResult::NotGuessed   => Color::rgb(0.1, 0.1, 0.1),
                    GuessResult::Entered(_)   => Color::rgb(0.0, 0.0, 0.0),
                    GuessResult::NotInWord(_) => Color::rgb(0.2, 0.2, 0.2),
                    GuessResult::InWord(_)    => Color::rgb(0.8 ,0.6, 0.3),
                    GuessResult::Correct(_)   => Color::rgb(0.0 ,0.6, 0.3),
                };
        
                ctx.fill(bounds, &background);
            });            

            row.add_child(
                Padding::new(4.0, 
                    SizedBox::new(
                        Label::dynamic(move |data: &AppState, _| guess_letter(data.guesses[guess][letter]).to_string())
                            .with_text_size(32.0) 
                            .with_text_alignment(TextAlignment::Center)
                            .background(painter)
                        )
                        .width(48.0)
                        .height(48.0)))            
        }
        result.add_child(row);
    }
    result
}

fn key_button(character: char) -> impl Widget<AppState> {
    Button::from_label(Label::new(character.to_string())
        .with_text_size(24.))
        .on_click(move |_ctx, data: &mut AppState, _env| data.character_pressed(character))
        
}

fn build_keyboard()-> Flex<AppState> {
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
    row.add_child(Button::new("ENTER")
        .on_click(move |_ctx, data: &mut AppState, _env| data.enter_pressed()));    
    for key_char in vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'] {
        row.add_child(key_button(key_char));
    }
    row.add_child(Button::new("\u{232B}")
        .on_click(move |_ctx, data: &mut AppState, _env| data.backspace_pressed()));
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
        _env: &Env
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
                    },
                    Backspace => data.backspace_pressed(),
                    Enter => data.enter_pressed(),
                    _ => ()
                }                              
            },
            _ => ()
        };
        Option::from(event)
    }
}

pub fn main() {
    let app_state = AppState::new();

    let guess_grid = build_guess_grid(&app_state);

    AppLauncher::with_window(
        WindowDesc::new(move || Flex::column()
                .with_flex_spacer(0.1)
                .with_child(guess_grid)
                .with_flex_spacer(1.0)
                .with_child(build_keyboard())            
            )
            .window_size((
                640.,
                600.,
            ))
            .resizable(false)
            .title(LocalizedString::new("app-title").with_placeholder("Wordle")),
    )
    .delegate(KeyBoardHandler)
    .use_simple_logger()    
    .launch(app_state)
    .expect("launch failed");
}