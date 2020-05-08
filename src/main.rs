extern crate regex;

use druid::widget::Slider;
use druid::widget::{CrossAxisAlignment, Flex, Label};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use regex::Regex;
use std::process::Command;

#[derive(Clone, Debug, Data, Lens)]
struct Brightness {
    #[lens(name = "lightroom")]
    term: String,
    scale: f64,
    output: String,
}

trait Lightroom {
    fn new() -> Self;
    fn default_brightness() -> f64;
    fn get_brightness_level(stdout: String) -> f64;
    fn get_brightness() -> f64;
    fn set_brightness(&self, f: f64);
    fn set_output(self) -> Self;
}

impl Lightroom for Brightness {
    fn new() -> Self {
        Self {
            term: "Brightness".into(),
            scale: Self::get_brightness(),
            output: "HDMI-0".to_string(),
        }
    }

    fn default_brightness() -> f64 {
        0.5
    }

    fn set_output(mut self) -> Self {
        let pattern = Regex::new(r"(?P<output>[0-9A-Z-]+) connected primary")
            .expect("failed to compile regex");

        match Command::new("xrandr").output() {
            Ok(output) => unsafe {
                let stdout = String::from_utf8_unchecked(output.stdout);
                let s = pattern
                    .captures(&stdout)
                    .and_then(|cap| cap.name("output").map(|out| out.as_str()))
                    .expect("failed to find the output");

                self.output = s.to_string();
            },
            Err(error) => {
                eprintln!("ERROR: {}", error);
            }
        };

        self
    }

    fn get_brightness_level(stdout: String) -> f64 {
        const KEY: &str = "Brightness:";
        for s in stdout.split("\n") {
            if s.contains(KEY) {
                let mut nstr = s.replace(KEY, "");
                nstr.retain(|c| !c.is_whitespace());
                return nstr.parse().unwrap();
            }
        }
        return Self::default_brightness();
    }

    fn get_brightness() -> f64 {
        let mut cmd = Command::new("xrandr");
        cmd.arg("--verbose");

        match cmd.output() {
            Ok(output) => unsafe {
                let stderr = String::from_utf8_unchecked(output.stderr);
                if stderr == "" {
                    let stdout = String::from_utf8_unchecked(output.stdout);
                    return Self::get_brightness_level(stdout);
                }
                eprintln!("{}", stderr);
            },
            Err(error) => {
                eprintln!("ERROR: {}", error);
            }
        }
        Self::default_brightness()
    }

    fn set_brightness(&self, f: f64) {
        let f = if f > 0.4 && f <= 1.0 { f } else { 0.4 };
        Command::new("xrandr")
            .arg("--output")
            .arg(self.output.as_str())
            .arg("--brightness")
            .arg(f.to_string())
            .spawn()
            .expect("failed to set brightness");
    }
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .window_size((240., 80.))
        .with_min_size((240., 80.))
        .title(LocalizedString::new("light-room").with_placeholder("Lightroom"));

    let br = Brightness::new().set_output();
    AppLauncher::with_window(main_window)
        .launch(br)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<Brightness> {
    // `Slider` is of type `Widget<f64>`
    // via `.lens` we get it to be of type `Widget<Brightness>`
    let slider = Slider::new().lens(Brightness::scale);

    let label = Label::new(|d: &Brightness, _: &Env| {
        d.set_brightness(d.scale);
        format!("{}: {:.2}", d.term, d.scale)
    });

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(label)
        .with_spacer(8.0)
        .with_child(
            Flex::row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .with_spacer(8.0)
                .with_child(slider),
        )
        .center()
}
