use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::widget::{Flex, IdentityWrapper};
use druid::{
    AppLauncher, BoxConstraints, Color, Command, Data, Env, Event, EventCtx, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, LocalizedString, MenuDesc, MenuItem, MouseButton, PaintCtx, Point,
    Rect, Selector, Size, Target, UpdateCtx, Widget, WindowDesc,
};

use rand::prelude::*;

use std::sync::Arc;

use image;

const GRID_WIDTH: usize = 30;
const GRID_HEIGHT: usize = 20;
const GRID_COUNT: usize = GRID_WIDTH * GRID_HEIGHT;
const MINE_PERCENTAGE_CHANCE: u8 = 15;

struct Square {
    grid_index: usize,
}

#[derive(Clone, Lens, Data)]
struct AppState {
    mines: Arc<[bool; GRID_COUNT]>,
    flags: Arc<[bool; GRID_COUNT]>,
    neighbors: Arc<[Option<usize>; GRID_COUNT]>,
    widget_ids: Arc<[Option<WidgetId>; GRID_COUNT]>,
    game_over: bool,
    mine_image: Arc<Vec<u8>>,
    flag_image: Arc<Vec<u8>>,
}

const RESET: Selector = Selector::new("RESET");
const REPAINT: Selector = Selector::new("REPAINT");

impl AppState {
    fn init(&mut self) {
        let mut rng = rand::thread_rng();

        let mines = Arc::make_mut(&mut self.mines);
        let flags = Arc::make_mut(&mut self.flags);
        let neighbors = Arc::make_mut(&mut self.neighbors);
        let widget_ids = Arc::make_mut(&mut self.widget_ids);
        for i in 0..mines.len() {
            mines[i] = rng.gen_range(0, 100) < MINE_PERCENTAGE_CHANCE;
        }
        for i in 0..flags.len() {
            flags[i] = false;
        }
        for i in 0..neighbors.len() {
            neighbors[i] = None;
        }
        for i in 0..widget_ids.len() {
            if widget_ids[i].is_none() {
                widget_ids[i] = Some(WidgetId::next());
            }
        }
        self.game_over = false;
    }

    fn compute_neighbors(&mut self, grid_index: usize, ctx: &mut EventCtx) {
        let mut count: usize = 0;

        let x = grid_index % GRID_WIDTH;
        let y = grid_index / GRID_WIDTH;

        let neighbors = [
            if y > 0 && x > 0 {
                Some((y - 1) * GRID_WIDTH + (x - 1))
            } else {
                None
            },
            if y > 0 {
                Some((y - 1) * GRID_WIDTH + x)
            } else {
                None
            },
            if y > 0 && x < GRID_WIDTH - 1 {
                Some((y - 1) * GRID_WIDTH + (x + 1))
            } else {
                None
            },
            if x > 0 {
                Some(y * GRID_WIDTH + (x - 1))
            } else {
                None
            },
            if x < GRID_WIDTH - 1 {
                Some(y * GRID_WIDTH + (x + 1))
            } else {
                None
            },
            if y < GRID_HEIGHT - 1 && x > 0 {
                Some((y + 1) * GRID_WIDTH + (x - 1))
            } else {
                None
            },
            if y < GRID_HEIGHT - 1 {
                Some((y + 1) * GRID_WIDTH + x)
            } else {
                None
            },
            if y < GRID_HEIGHT - 1 && x < GRID_WIDTH - 1 {
                Some((y + 1) * GRID_WIDTH + (x + 1))
            } else {
                None
            },
        ];

        for neighbor in neighbors.iter() {
            if let Some(neighbor_index) = neighbor {
                if self.mines[*neighbor_index] {
                    count += 1
                }
            }
        }

        Arc::make_mut(&mut self.neighbors)[grid_index] = Some(count);

        ctx.submit_command(Command::new(
            REPAINT,
            (),
            Target::Widget(self.widget_ids[grid_index].unwrap()),
        ));

        if !self.mines[grid_index] && count == 0 {
            for neighbor in neighbors.iter() {
                if let Some(neighbor_index) = neighbor {
                    if self.neighbors[*neighbor_index].is_none() {
                        self.compute_neighbors(*neighbor_index, ctx)
                    }
                }
            }
        }
    }

    fn toggle_flag(&mut self, grid_index: usize) {
        Arc::make_mut(&mut self.flags)[grid_index] = !self.flags[grid_index];
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
            Event::MouseDown(e) => {
                if !data.game_over {
                    if e.button == MouseButton::Left {
                        if data.neighbors[self.grid_index].is_none() {
                            data.compute_neighbors(self.grid_index, ctx);
                        }
                        if data.mines[self.grid_index] {
                            data.game_over = true;
                        }
                    } else if e.button == MouseButton::Right {
                        if data.neighbors[self.grid_index].is_none() {
                            data.toggle_flag(self.grid_index);
                            ctx.request_paint();
                        }
                    }
                }
            }
            Event::Command(c) => {
                if let Some(_) = c.get(RESET) {
                    data.init();
                    ctx.set_handled();
                    ctx.submit_command(Command::new(REPAINT, (), Target::Global));
                } else if let Some(_) = c.get(REPAINT) {
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
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
        Size::new(24., 24.)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        let size = ctx.size();
        let rect = Rect::from_origin_size(Point::ORIGIN, size);
        if data.flags[self.grid_index] {
            ctx.fill(&rect, &Color::rgb(0.5, 0.5, 0.5));
            let image = ctx
                .make_image(24, 24, &data.flag_image, ImageFormat::RgbaSeparate)
                .unwrap();
            ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor)
        } else {
            if let Some(nc) = data.neighbors[self.grid_index] {
                if data.mines[self.grid_index] {
                    ctx.fill(&rect, &Color::rgb(1., 0., 0.));
                    let image = ctx
                        .make_image(24, 24, &data.mine_image, ImageFormat::RgbaSeparate)
                        .unwrap();
                    ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor)
                } else if nc > 0 {
                    let layout = ctx
                        .text()
                        .new_text_layout(format!("{}", nc))
                        .font(FontFamily::MONOSPACE, 24.0)
                        .text_color(Color::rgb8(255, 255, 255))
                        .build()
                        .unwrap();
                    ctx.draw_text(&layout, (6.0, -2.0));
                }
            } else {
                ctx.fill(&rect, &Color::rgb(0.5, 0.5, 0.5));
            }
        }
        ctx.stroke(&rect, &Color::BLACK, 1.0);
    }
}

fn build_grid(widget_ids: [Option<WidgetId>; GRID_COUNT]) -> Flex<AppState> {
    let mut result = Flex::column();
    for y in 0..GRID_HEIGHT {
        let mut row = Flex::row();
        for x in 0..GRID_WIDTH {
            row.add_child(IdentityWrapper::wrap(
                Square::new(y * GRID_WIDTH + x),
                widget_ids[y * GRID_WIDTH + x].unwrap(),
            ))
        }
        result.add_child(row);
    }
    result
}

pub fn main() {
    let mut app_state = AppState {
        mines: Arc::new([false; GRID_COUNT]),
        flags: Arc::new([false; GRID_COUNT]),
        neighbors: Arc::new([None; GRID_COUNT]),
        widget_ids: Arc::new([None; GRID_COUNT]),
        game_over: false,
        mine_image: Arc::new(
            image::load_from_memory(include_bytes!("../../resources/mine.png"))
                .unwrap()
                .to_rgba8()
                .into_raw(),
        ),
        flag_image: Arc::new(
            image::load_from_memory(include_bytes!("../../resources/flag.png"))
                .unwrap()
                .to_rgba8()
                .into_raw(),
        ),
    };

    app_state.init();

    let widget_ids = app_state.widget_ids.as_ref().clone();

    AppLauncher::with_window(
        WindowDesc::new(move || Flex::column().with_child(build_grid(widget_ids)))
            .menu(MenuDesc::empty().append(MenuItem::new(
                LocalizedString::new("Reset"),
                Command::new(RESET, (), Target::Global),
            )))
            .window_size((
                2.0 + GRID_WIDTH as f64 * 25.,
                54.0 + GRID_HEIGHT as f64 * 25.,
            ))
            .resizable(false)
            .title(LocalizedString::new("app-title").with_placeholder("Minesweeper")),
    )
    .use_simple_logger()
    .launch(app_state)
    .expect("launch failed");
}
