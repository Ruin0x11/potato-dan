use std::collections::{HashMap, VecDeque};
use imgui;

pub struct Variable {
    val: f32,
    min: f32,
    max: f32,
}

pub struct UiState {
    pub fps: VecDeque<f32>,
    pub vars: HashMap<&'static str, Variable>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            fps: VecDeque::new(),
            vars: HashMap::new(),
        }
    }
}

pub fn get(key: &'static str, default: f32, min: f32, max: f32) -> f32 {
    instance::with_mut(|state| state.vars.entry(key).or_insert(Variable {
        val: default,
        min: min,
        max: max,
    }).val)
}

pub fn set_fps(fps: f32) {
    instance::with_mut(|state| {
        if state.fps.len() > 15 {
            state.fps.pop_front();
        }
        state.fps.push_back(fps);
    })
}

make_global!(UI_STATE, UiState, UiState::new());

pub fn run(ui: &imgui::Ui) {
    instance::with_mut(|state| {
    ui.window(im_str!("Hello world"))
        .size((300.0, 800.0), imgui::ImGuiCond::FirstUseEver)
        .build(|| {
            ui.text(im_str!("This...is...imgui-rs!"));
            imgui::PlotLines::new(ui, im_str!("FPS"), &Vec::from(state.fps.clone()))
                .overlay_text(im_str!("{}", state.fps.back().cloned().unwrap_or(0.0)))
                .build();
            ui.separator();
            let mouse_pos = ui.imgui().mouse_pos();
            ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));

            for (key, mut var) in state.vars.iter_mut() {
                ui.slider_float(im_str!("{}", key), &mut var.val, var.min, var.max)
                    .build();
            }
        });

    })
}
