use glium::glutin::VirtualKeyCode;
use engine::keys::KeyCode;

impl From<VirtualKeyCode> for KeyCode {
    fn from(key: VirtualKeyCode) -> KeyCode {
        match key {
            VirtualKeyCode::A        => KeyCode::A,
            VirtualKeyCode::B        => KeyCode::B,
            VirtualKeyCode::C        => KeyCode::C,
            VirtualKeyCode::D        => KeyCode::D,
            VirtualKeyCode::E        => KeyCode::E,
            VirtualKeyCode::F        => KeyCode::F,
            VirtualKeyCode::G        => KeyCode::G,
            VirtualKeyCode::H        => KeyCode::H,
            VirtualKeyCode::I        => KeyCode::I,
            VirtualKeyCode::J        => KeyCode::J,
            VirtualKeyCode::K        => KeyCode::K,
            VirtualKeyCode::L        => KeyCode::L,
            VirtualKeyCode::M        => KeyCode::M,
            VirtualKeyCode::N        => KeyCode::N,
            VirtualKeyCode::O        => KeyCode::O,
            VirtualKeyCode::P        => KeyCode::P,
            VirtualKeyCode::Q        => KeyCode::Q,
            VirtualKeyCode::R        => KeyCode::R,
            VirtualKeyCode::S        => KeyCode::S,
            VirtualKeyCode::T        => KeyCode::T,
            VirtualKeyCode::U        => KeyCode::U,
            VirtualKeyCode::V        => KeyCode::V,
            VirtualKeyCode::W        => KeyCode::W,
            VirtualKeyCode::X        => KeyCode::X,
            VirtualKeyCode::Y        => KeyCode::Y,
            VirtualKeyCode::Z        => KeyCode::Z,
            VirtualKeyCode::Escape   => KeyCode::Escape,
            VirtualKeyCode::F1       => KeyCode::F1,
            VirtualKeyCode::F2       => KeyCode::F2,
            VirtualKeyCode::F3       => KeyCode::F3,
            VirtualKeyCode::F4       => KeyCode::F4,
            VirtualKeyCode::F5       => KeyCode::F5,
            VirtualKeyCode::F6       => KeyCode::F6,
            VirtualKeyCode::F7       => KeyCode::F7,
            VirtualKeyCode::F8       => KeyCode::F8,
            VirtualKeyCode::F9       => KeyCode::F9,
            VirtualKeyCode::F10      => KeyCode::F10,
            VirtualKeyCode::F11      => KeyCode::F11,
            VirtualKeyCode::F12      => KeyCode::F12,
            VirtualKeyCode::Insert   => KeyCode::Insert,
            VirtualKeyCode::Home     => KeyCode::Home,
            VirtualKeyCode::Delete   => KeyCode::Delete,
            VirtualKeyCode::End      => KeyCode::End,
            VirtualKeyCode::PageDown => KeyCode::PageDown,
            VirtualKeyCode::PageUp   => KeyCode::PageUp,
            VirtualKeyCode::Left     => KeyCode::Left,
            VirtualKeyCode::Up       => KeyCode::Up,
            VirtualKeyCode::Right    => KeyCode::Right,
            VirtualKeyCode::Down     => KeyCode::Down,
            VirtualKeyCode::Return   => KeyCode::Enter,
            VirtualKeyCode::Space    => KeyCode::Space,
            VirtualKeyCode::Numpad0  => KeyCode::NumPad0,
            VirtualKeyCode::Numpad1  => KeyCode::NumPad1,
            VirtualKeyCode::Numpad2  => KeyCode::NumPad2,
            VirtualKeyCode::Numpad3  => KeyCode::NumPad3,
            VirtualKeyCode::Numpad4  => KeyCode::NumPad4,
            VirtualKeyCode::Numpad5  => KeyCode::NumPad5,
            VirtualKeyCode::Numpad6  => KeyCode::NumPad6,
            VirtualKeyCode::Numpad7  => KeyCode::NumPad7,
            VirtualKeyCode::Numpad8  => KeyCode::NumPad8,
            VirtualKeyCode::Numpad9  => KeyCode::NumPad9,
            VirtualKeyCode::Comma    => KeyCode::Comma,
            VirtualKeyCode::Period   => KeyCode::Period,

            _ => KeyCode::Unknown(' '),
        }
    }
}