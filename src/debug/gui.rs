use std::collections::{HashMap, VecDeque};
use imgui;
use toml::Value;

use util;

pub struct Variable {
    pub val: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for Variable {
    fn default() -> Self {
        Variable {
            val: 12.3456789,
            min: 0.0,
            max: 100.0,
        }
    }
}

pub type Variables = HashMap<String, Variable>;

pub struct UiState {
    pub show_log: bool,

    pub fps: VecDeque<f32>,
    pub vars: Variables,
    pub log: LogWindow,
}

pub fn vars_from_toml() -> Variables {
    let val = util::toml::toml_value_from_file("data/debug.toml");
    let mut vars = HashMap::new();

    let keys = match util::toml::expect_value_in_table(&val, "keys") {
        Value::Array(array) => array,
        _                   => {
            println!("No keys array found!");
            return vars;
        }
    };

    for key in keys.iter() {
        let name: String = util::toml::expect_value_in_table(key, "name");
        let default: f32 = util::toml::expect_value_in_table(key, "default");
        let min: f32 = util::toml::expect_value_in_table(key, "min");
        let max: f32 = util::toml::expect_value_in_table(key, "max");

        vars.insert(name, Variable { val: default, min: min, max: max });
    }
    vars
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            show_log: false,

            fps: VecDeque::new(),
            vars: vars_from_toml(),
            log: LogWindow::new(),
        }
    }

    pub fn reset_vars(&mut self) {
        self.vars = vars_from_toml();
    }
}

pub struct LogWindow {
    buf: String,
    scroll_to_bottom: bool,
}

impl LogWindow {
    pub fn new() -> Self {
        LogWindow {
            buf: String::new(),
            scroll_to_bottom: true,
        }
    }
    pub fn run(&mut self, ui: &imgui::Ui, opened: &mut bool) {
        ui.window(im_str!("Log"))
            .opened(opened)
            .size((500.0, 400.0), imgui::ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.small_button(im_str!("Clear")) {
                    self.clear();
                }
                //ui.same_line(0.0);
                //let copy = ui.small_button(im_str!("Copy"));
                ui.same_line(0.0);
                ui.separator();
                ui.child_frame(im_str!("scrolling"), (0.0, 0.0))
                    .scrollbar_horizontal(true)
                    .build(|| {
                        ui.text(&self.buf);
                        // if self.scroll_to_bottom {
                        //
                        // }
                        self.scroll_to_bottom = false;
                    });
            });

    }

    pub fn add(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn clear(&mut self) {
        self.buf.clear();
    }
}
