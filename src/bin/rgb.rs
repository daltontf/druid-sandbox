use druid::{AppLauncher, Color, Data, Env, Lens, LensExt, LocalizedString, WidgetExt, WindowDesc};

use druid::widget::prelude::*;
use druid::widget::{CrossAxisAlignment, Flex, Label, Painter, Slider};

#[derive(Clone, Data, Lens)]
struct AppState {
    red: u8,
    green: u8,
    blue: u8,
}

fn add_row<L: Lens<AppState, u8> + 'static>(
    label: &str,
    lens: L,
    f: &'static dyn Fn(&AppState, &Env) -> String,
) -> Flex<AppState> {
    Flex::row()
        .with_child(Label::new(label).fix_width(50.0))
        .with_spacer(5.0)
        .with_flex_child(
            Slider::new()
                .expand_width()
                .lens(lens.map(|x| (*x as f64) / 255.0, |x, y| *x = (y * 255.0) as u8)),
            1.0,
        )
        .with_spacer(5.0)
        .with_child(Label::new(f).fix_width(30.0))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true)
}

pub fn main() {
    let app_state = AppState {
        red: 0,
        green: 0,
        blue: 0,
    };

    AppLauncher::with_window(
        WindowDesc::new(|| {
            Flex::column()
                .must_fill_main_axis(true)
                .with_spacer(10.0)
                .with_child(add_row(
                    "Red",
                    AppState::red,
                    &|data: &AppState, _env: &Env| data.red.to_string(),
                ))
                .with_spacer(10.0)
                .with_child(add_row(
                    "Green",
                    AppState::green,
                    &|data: &AppState, _env: &Env| data.green.to_string(),
                ))
                .with_spacer(10.0)
                .with_child(add_row(
                    "Blue",
                    AppState::blue,
                    &|data: &AppState, _env: &Env| data.blue.to_string(),
                ))
                .with_spacer(10.0)
                .with_flex_child(
                    Painter::new(|ctx, data: &AppState, env| {
                        let bounds = ctx.size().to_rect();
                        let rounded = bounds.to_rounded_rect(5.0);
                        ctx.fill(&rounded, &Color::rgb8(data.red, data.green, data.blue));
                        ctx.stroke(&rounded, &env.get(druid::theme::BORDER_DARK), 2.0);
                    }),
                    1.0,
                )
                .with_spacer(10.0)
        })
        .window_size((500., 300.))
        .resizable(false)
        .title(LocalizedString::new("app-title").with_placeholder("RGB Select")),
    )
    .use_simple_logger()
    .launch(app_state)
    .expect("launch failed");
}
