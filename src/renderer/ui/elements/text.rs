use renderer::ui::*;
use renderer::ui::renderer::*;

pub struct UiText {
    pub text: String,
    pub shadow: bool,
}

impl UiText {
    pub fn new(text: String) -> Self {
        UiText {
            text: text,
            shadow: false,
        }
    }
}

impl UiElement for UiText {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let mut text_lines = renderer.wrap_text(&self.text, renderer.size.1);
        text_lines.reverse();
        for (idx, line) in text_lines.iter().enumerate() {
            let pos = (0, (idx as u32 * renderer.get_font_size()) as i32);
            if self.shadow {
                renderer.add_string_shadow(pos, None, line);
            } else {
                renderer.with_color((0, 0, 0, 255), |r| { r.add_string(pos, None, line); });
            }
        }
    }
}
