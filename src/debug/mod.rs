use calx_ecs::Entity;
use world::World;
use imgui::{self, ImGui, Ui};

pub mod gui;

use self::gui::*;

mod text {
    make_global!(DEBUG_TEXT, Option<String>, None);
}

pub fn pop_text() -> Option<String> {
    let ret = text::instance::with(|t| t.clone());
    text::instance::with_mut(|t| *t = None);
    ret
}

pub fn add_text(text: String) {
    text::instance::with_mut(|t| {
        let next = match t {
            &mut Some(ref tex) => format!("{}\n{}", tex, text),
            &mut None => text,
        };
        *t = Some(next)
    });
}

mod entity {
    use calx_ecs::Entity;
    make_global!(TARGET_ENTITY, Option<Entity>, None);
}

pub fn follow_entity(entity: Option<Entity>) {
    entity::instance::with_mut(|e| *e = entity);
}

fn entity_info(entity: Entity, world: &World) -> String {
    String::new()
}

pub fn update(world: &World) {
    entity::instance::with_mut(|e| if e.map_or(true, |en| !world.contains(en)) {
                                   *e = None;
                                   add_text(String::new());
                               });
    entity::instance::with(|e| if let &Some(entity) = e {
                               add_text(entity_info(entity, world));
                           });
}

pub fn get(key: &str) -> f32 {
    instance::with_mut(|state| state.vars.entry(key.to_string()).or_insert(Variable::default()).val)
}

pub fn set_fps(fps: f32) {
    instance::with_mut(|state| {
        if state.fps.len() > 15 {
            state.fps.pop_front();
        }
        state.fps.push_back(fps);
    })
}

pub fn log(s: &str) {
    instance::with_mut(|state| {
        state.log.add(&format!("{}\n", s));
    })
}

make_global!(UI_STATE, gui::UiState, gui::UiState::new());

pub fn run_ui(ui: &imgui::Ui) {
    instance::with_mut(|state| {
        if state.show_log {
            state.log.run(ui, &mut state.show_log);
        }
        ui.window(im_str!("Hello world"))
            .size((300.0, 800.0), imgui::ImGuiCond::FirstUseEver)
            .menu_bar(true)
            .build(|| {
                ui.text(im_str!("This...is...imgui-rs!"));

                ui.menu_bar(|| {
                    ui.menu(im_str!("View")).build(|| {
                        ui.menu_item(im_str!("Log window"))
                            .selected(&mut state.show_log)
                            .build();
                    });
                });

                imgui::PlotLines::new(ui, im_str!("FPS"), &Vec::from(state.fps.clone()))
                    .overlay_text(im_str!("{}", state.fps.back().cloned().unwrap_or(0.0)))
                    .build();
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));

                ui.separator();
                if ui.small_button(im_str!("Reload")) {
                    state.reset_vars();
                }

                for (key, mut var) in state.vars.iter_mut() {
                    ui.slider_float(im_str!("{}", key), &mut var.val, var.min, var.max)
                        .build();
                }
            });

    })
}
