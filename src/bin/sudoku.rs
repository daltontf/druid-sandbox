use druid::piet::{FontFamily, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::widget::Container;
use druid::widget::{Flex, IdentityWrapper};
use druid::AppDelegate;
use druid::Command;
use druid::DelegateCtx;
use druid::ExtEventSink;
use druid::FileDialogOptions;
use druid::FileSpec;
use druid::Handled;
use druid::MenuDesc;
use druid::MenuItem;
use druid::Target;
use druid::{
    AppLauncher, BoxConstraints, Code, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Point, Rect, Selector, Size, UpdateCtx,
    Widget, WidgetExt, WindowDesc,
};

use std::sync::Arc;

const SOLVE: Selector = Selector::new("SOLVE");
const REQUEST_FOCUS: Selector = Selector::new("REQUEST_FOCUS");
const REPAINT: Selector = Selector::new("REPAINT");
const SOLVE_COMPLETE: Selector<[u8; 81]> = Selector::new("SOLVE_COMPLETE");

struct Square {
    grid_index: usize,
}

#[derive(Clone, Lens, Data)]
struct AppState {
    widget_ids: Arc<[Option<WidgetId>; 81]>,
    values: Arc<[u8; 81]>,
    is_legal: Arc<[bool; 81]>,
    solving: bool,
}

impl AppState {
    fn init(&mut self) {
        let widget_ids = Arc::make_mut(&mut self.widget_ids);
        for i in 0..widget_ids.len() {
            if widget_ids[i].is_none() {
                widget_ids[i] = Some(WidgetId::next());
            }
        }
    }

    fn set_value(&mut self, index: usize, value: u8) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        Arc::make_mut(&mut self.values)[index] = value;
        result.push(index);
        if value != 0 {
            result.append(&mut self.calculate_legality(index, value));
        }
        let row = index / 9;
        let col = index % 9;
        for other in 0..9 {
            let row_index = row * 9 + other;
            let row_value = self.values[row_index];
            if value == 0 || row_value != 0 {
                result.append(&mut self.calculate_legality(row_index, row_value));
            }
            let col_index = other * 9 + col;
            let col_value = self.values[col_index];
            if value == 0 || col_value != 0 {
                result.append(&mut self.calculate_legality(col_index, col_value));
            }
            let grid_index = (((row / 3) * 3) + other / 3) * 9 + (((col / 3) * 3) + other % 3);
            let grid_value = self.values[grid_index];
            if value == 0 || grid_value != 0 {
                result.append(&mut self.calculate_legality(grid_index, grid_value));
            }
        }
        result
    }

    fn is_legal_move(board: &[u8; 81], index: usize, value: u8) -> bool {
        let row = index / 9;
        let col = index % 9;
        for other in 0..9 {
            let row_index = row * 9 + other;
            let col_index = other * 9 + col;
            let grid_index = (((row / 3) * 3) + other / 3) * 9 + (((col / 3) * 3) + other % 3);
            if row_index != index && board[row_index] == value
                || col_index != index && board[col_index] == value
                || grid_index != index && board[grid_index] == value
            {
                return false;
            }
        }
        true
    }

    fn solve_board(&mut self, sink: ExtEventSink) {
        let mut board: [u8; 81] = [0; 81];
        for i in 0..81 {
            board[i] = self.values[i];
        }
        self.solving = true;
        std::thread::spawn(move || {
            AppState::solve(&mut board);
            sink.submit_command(SOLVE_COMPLETE, board, Target::Global)
                .unwrap()
        });
    }

    fn solve(board: &mut [u8; 81]) -> bool {
        for i in 0..81 {
            if board[i] == 0 {
                for guess in 1..10 {
                    if AppState::is_legal_move(board, i, guess) {
                        board[i] = guess;
                        if AppState::solve(board) {
                            return true;
                        }
                    }
                }
                board[i] = 0;
                return false;
            }
        }
        true
    }

    fn calculate_legality(&mut self, index: usize, value: u8) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        let was_legal = self.is_legal[index];
        if value > 0 {
            let is_legal_now = AppState::is_legal_move(&self.values, index, value);
            let is_legal = Arc::make_mut(&mut self.is_legal);
            is_legal[index] = is_legal_now;
            if was_legal != is_legal_now {
                result.push(index);
            }
            return result;
        }
        let is_legal = Arc::make_mut(&mut self.is_legal);
        is_legal[index] = true;
        if !was_legal {
            result.push(index);
        }
        result
    }
}

impl Square {
    fn new(grid_index: usize) -> Self {
        Square { grid_index }
    }
}

impl Widget<AppState> for Square {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.request_focus();
            }
            Event::KeyDown(e) => {
                let digit = match e.code {
                    Code::Space => Some(0),
                    Code::Digit1 => Some(1),
                    Code::Digit2 => Some(2),
                    Code::Digit3 => Some(3),
                    Code::Digit4 => Some(4),
                    Code::Digit5 => Some(5),
                    Code::Digit6 => Some(6),
                    Code::Digit7 => Some(7),
                    Code::Digit8 => Some(8),
                    Code::Digit9 => Some(9),
                    _ => Option::None,
                };
                if !data.solving {
                    if let Some(digit_value) = digit {
                        let modfied = data.set_value(self.grid_index, digit_value);
                        for index in modfied {
                            ctx.submit_command(Command::new(
                                REPAINT,
                                (),
                                Target::Widget(data.widget_ids[index as usize].unwrap()),
                            ));
                        }
                    } else {
                        let delta: Option<i8> = match e.code {
                            Code::ArrowDown => Some(9),
                            Code::ArrowUp => Some(-9),
                            Code::ArrowLeft => Some(-1),
                            Code::ArrowRight => Some(1),
                            _ => Option::None,
                        };
                        if let Some(delta_value) = delta {
                            let mut new_index: i8 = self.grid_index as i8 + delta_value;
                            if new_index < 0 {
                                new_index = 81 + new_index;
                            } else if new_index > 80 {
                                new_index = new_index - 81;
                            }
                            ctx.submit_command(Command::new(
                                REQUEST_FOCUS,
                                (),
                                Target::Widget(data.widget_ids[new_index as usize].unwrap()),
                            ));
                        }
                    }
                }
            }
            Event::Command(c) => {
                if let Some(_) = c.get(REQUEST_FOCUS) {
                    ctx.request_focus();
                } else if let Some(_) = c.get(REPAINT) {
                    ctx.request_paint();
                } else if let Some(_) = c.get(SOLVE) {
                    ctx.set_handled();
                    if data.is_legal.iter().all(|&x| x) {
                        data.solve_board(ctx.get_external_handle());
                        ctx.submit_command(Command::new(REPAINT, (), Target::Global));
                    }
                } else if let Some(board) = c.get(SOLVE_COMPLETE) {
                    let values = Arc::make_mut(&mut data.values);
                    for i in 0..81 {
                        values[i] = board[i];
                    }
                    data.solving = false;
                    ctx.submit_command(Command::new(REPAINT, (), Target::Global));
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus();
            }
            LifeCycle::FocusChanged(_) => {
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        Size::new(48., 48.)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        let size = ctx.size();
        let rect = Rect::from_origin_size(Point::ORIGIN, size);

        let color = if data.solving {
            &Color::GRAY
        } else {
            if ctx.has_focus() {
                &Color::AQUA
            } else {
                &Color::WHITE
            }
        };

        ctx.fill(&rect, color);
        ctx.stroke(&rect, &Color::BLACK, 1.0);

        let value = data.values[self.grid_index];
        if value > 0 {
            let layout = ctx
                .text()
                .new_text_layout(format!("{}", value))
                .font(FontFamily::MONOSPACE, 48.0)
                .text_color(if data.is_legal[self.grid_index] {
                    Color::BLACK
                } else {
                    Color::RED
                })
                .build()
                .unwrap();
            ctx.draw_text(&layout, (10.0, -2.0));
        }
    }
}

fn build_grid(widget_ids: [Option<WidgetId>; 81]) -> Flex<AppState> {
    let mut result = Flex::column();
    for gridy in 0..3 {
        let mut grid_row = Flex::row();
        for gridx in 0..3 {
            let mut grid_column = Flex::column();
            for y in 0..3 {
                let mut row = Flex::row();
                for x in 0..3 {
                    let index = (gridy * 3 + y) * 9 + (gridx * 3 + x);
                    row.add_child(IdentityWrapper::wrap(
                        Square::new(index),
                        widget_ids[index].unwrap(),
                    ))
                }
                grid_column.add_child(row);
            }
            grid_row.add_child(Container::new(grid_column).border(Color::BLACK, 1.0));
        }
        result.add_child(grid_row);
    }
    result
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(druid::commands::SAVE_FILE_AS) {
            if let Err(e) = std::fs::write(
                file_info.path(),
                data.values[..].iter().map(|&x| x + 48).collect::<Vec<u8>>(),
            ) {
                println!("Error writing file: {}", e);
            }
            return Handled::Yes;
        }
        if let Some(file_info) = cmd.get(druid::commands::OPEN_FILE) {
            match std::fs::read_to_string(file_info.path()) {
                Ok(s) => {
                    let first_line = s.lines().next().unwrap_or("").as_bytes();
                    if first_line.len() > 80 {
                        let values = Arc::make_mut(&mut data.values);
                        for i in 0..81 {
                            values[i] = first_line[i] - 48;
                        }
                        ctx.submit_command(Command::new(REPAINT, (), Target::Global));
                    }
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
            return Handled::Yes;
        }
        Handled::No
    }
}

pub fn main() {
    let mut app_state = AppState {
        widget_ids: Arc::new([None; 81]),
        values: Arc::new([0u8; 81]),
        is_legal: Arc::new([true; 81]),
        solving: false,
    };

    app_state.init();

    let widget_ids = app_state.widget_ids.as_ref().clone();

    let txt = FileSpec::new("Text file", &["txt"]);
    // The options can also be generated at runtime,
    // so to show that off we create a String for the default save name.
    let default_save_name = String::from("sudodu_puzzle.txt");
    let save_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![txt])
        .default_type(txt)
        .default_name(default_save_name)
        .name_label("Target")
        .title("Choose a target for this lovely file")
        .button_text("Export");

    let open_dialog_options = save_dialog_options
        .clone()
        .default_name("MySavedFile.txt")
        .name_label("Source")
        .title("Where did you put that file?")
        .button_text("Import");

    AppLauncher::with_window(
        WindowDesc::new(move || Flex::column().with_child(build_grid(widget_ids)))
            .window_size((460., 502.))
            .resizable(false)
            .title(LocalizedString::new("app-title").with_placeholder("Sudoku"))
            .menu(
                MenuDesc::empty()
                    .append(MenuItem::new(
                        LocalizedString::new("Solve"),
                        Command::new(SOLVE, (), Target::Global),
                    ))
                    .append(MenuItem::new(
                        LocalizedString::new("Load"),
                        Command::new(
                            druid::commands::SHOW_OPEN_PANEL,
                            open_dialog_options.clone(),
                            Target::Auto,
                        ),
                    ))
                    .append(MenuItem::new(
                        LocalizedString::new("Save"),
                        Command::new(
                            druid::commands::SHOW_SAVE_PANEL,
                            save_dialog_options.clone(),
                            Target::Auto,
                        ),
                    )),
            ),
    )
    .use_simple_logger()
    .delegate(Delegate)
    .launch(app_state)
    .expect("launch failed");
}
