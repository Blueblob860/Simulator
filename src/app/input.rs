use gilrs::{GamepadId, Gilrs};
use roboscope_ipc::{SimServices, display::DisplayInput, snapshot::{ControllerInput, ControllerState, JoystickState}};

use crate::{ContPubType, DispInputPubType};

const DEF_CONTROLLER_STATE: ControllerState = ControllerState {
    connection: roboscope_ipc::snapshot::ControllerConnection::Offline,
    battery_level: 100,
    battery_capacity: 100,
    left_stick: JoystickState { x_raw: 0, y_raw: 0 },
    right_stick: JoystickState { x_raw: 0, y_raw: 0 },
    button_a: false, button_b: false,
    button_x: false, button_y: false,
    button_up: false, button_down: false,
    button_left: false, button_right: false,
    button_l1: false, button_l2: false,
    button_r1: false, button_r2: false,
    button_power: false,
};

const DEF_CONTROLLER_INPUT: ControllerInput = ControllerInput {
    primary: DEF_CONTROLLER_STATE,
    partner: DEF_CONTROLLER_STATE
};

pub struct V5InputHandler {
    disp_pub: DispInputPubType,
    mouse_coords: egui::Vec2 = egui::Vec2 { x: 0.0, y: 0.0 },
    pointer_down: bool = false,
    presses: u32 = 0,
    releases: u32 = 0,

    cont_pub: ContPubType,
    gilrs: Gilrs,
    gp1: Option<GamepadId> = None,
    gp2: Option<GamepadId> = None,
    cont_state: ControllerInput = DEF_CONTROLLER_INPUT,
}

impl V5InputHandler {
    pub fn new(sim: &SimServices) -> anyhow::Result<Self> {
        let disp_pub = sim.display_input()?.publisher_builder().create()?;
        let cont_pub = sim.controller_input()?.publisher_builder().create()?;
        Ok(Self {
            disp_pub,
            cont_pub,
            gilrs: Gilrs::new().unwrap(),
            ..
        })
    }

    pub fn update_controllers(&mut self) {
        // Only send an updated state if there was a change
        let mut cont1_updated = false;
        let mut cont2_updated = false;

        // Loop through all the gamepad updates since the last frame
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            // If there's a new gamepad connection,
            // and one of the two controllers are missing,
            // set that gamepad as the missing controller
            if self.gp1.is_none() { self.gp1 = Some(id); }
            else if self.gp2.is_none() { self.gp2 = Some(id); }

            // If either of the two gamepads received an
            // update this event, update their state
            if id == self.gp1.unwrap() {
                self.cont_state.primary = handle_gamepad_event(self.cont_state.primary, event);
                cont1_updated = true;
            } else if id == self.gp2.unwrap() {
                self.cont_state.partner = handle_gamepad_event(self.cont_state.partner, event);
                cont2_updated = true;
            }
        }

        // If one of the two controllers recieved an update then
        // pass it along to the brain
        if cont1_updated || cont2_updated {
            let _ = self.cont_pub.send_copy(self.cont_state);
        }
    }

    pub fn update_touch(&mut self, ui: &mut egui::Ui) {
        let widg_pos = ui.next_widget_position();
        let (clicked, released, held, moved) = ui.input(|i| {
            let interact_in_bounds = if let Some(pos) = i.pointer.press_origin() {
                let adj_mp = pos - widg_pos;
                adj_mp.x >= 0.0 && adj_mp.y >= 32.0 && adj_mp.x <= 480.0 && adj_mp.y <= 272.0
            } else { false };
            let hold = i.pointer.primary_down() && interact_in_bounds;
            let click = !self.pointer_down && hold;
            let release = self.pointer_down && !hold;
            self.pointer_down = hold;
            let moved = if let Some(pos) = i.pointer.latest_pos() && hold {
                let adj_mp = pos - widg_pos;
                let moved = (self.mouse_coords - adj_mp).length_sq() > 1.0;
                self.mouse_coords = adj_mp;
                moved
            } else { false };
            if click { self.presses = self.presses.wrapping_add(1); }
            if release { self.releases = self.releases.wrapping_add(1); }
            (click, release, hold, moved)
        });
        
        if clicked || released || (held && moved) {
            let _ = self.disp_pub.send_copy(DisplayInput {
                kind: if clicked {
                    roboscope_ipc::display::DisplayInputKind::Press
                } else if !clicked && held {
                    roboscope_ipc::display::DisplayInputKind::Hold
                } else {
                    roboscope_ipc::display::DisplayInputKind::Release
                },
                press_count: self.presses,
                release_count: self.releases,
                x: (self.mouse_coords.x) as i16,
                y: (self.mouse_coords.y) as i16,
            });
        }
    }
}

fn handle_gamepad_event(mut state: ControllerState, event: gilrs::EventType) -> ControllerState {
    match event {
        gilrs::EventType::ButtonPressed(button, _) => {
            match button {
                gilrs::Button::South => { state.button_b = true; },
                gilrs::Button::East => { state.button_a = true; },
                gilrs::Button::North => { state.button_x = true; },
                gilrs::Button::West => { state.button_y = true; },
                gilrs::Button::LeftTrigger => { state.button_l1 = true; },
                gilrs::Button::LeftTrigger2 => { state.button_l2 = true; },
                gilrs::Button::RightTrigger => { state.button_r1 = true; },
                gilrs::Button::RightTrigger2 => { state.button_r2 = true; },
                gilrs::Button::Start => { state.button_power = true; },
                gilrs::Button::DPadUp => { state.button_up = true; },
                gilrs::Button::DPadDown => { state.button_down = true; },
                gilrs::Button::DPadLeft => { state.button_left = true; },
                gilrs::Button::DPadRight => { state.button_right = true; },
                _ => {},
            }
        },
        gilrs::EventType::ButtonReleased(button, _) => {
            match button {
                gilrs::Button::South => { state.button_b = false; },
                gilrs::Button::East => { state.button_a = false; },
                gilrs::Button::North => { state.button_x = false; },
                gilrs::Button::West => { state.button_y = false; },
                gilrs::Button::LeftTrigger => { state.button_l1 = false; },
                gilrs::Button::LeftTrigger2 => { state.button_l2 = false; },
                gilrs::Button::RightTrigger => { state.button_r1 = false; },
                gilrs::Button::RightTrigger2 => { state.button_r2 = false; },
                gilrs::Button::Start => { state.button_power = false; },
                gilrs::Button::DPadUp => { state.button_up = false; },
                gilrs::Button::DPadDown => { state.button_down = false; },
                gilrs::Button::DPadLeft => { state.button_left = false; },
                gilrs::Button::DPadRight => { state.button_right = false; },
                _ => {},
            }
        },
        gilrs::EventType::AxisChanged(axis, v, _) => {
            match axis {
                gilrs::Axis::LeftStickX => { state.left_stick.x_raw = (v * 127.0) as i8; },
                gilrs::Axis::LeftStickY => { state.left_stick.y_raw = (v * 127.0) as i8; },
                gilrs::Axis::RightStickX => { state.right_stick.x_raw = (v * 127.0) as i8; },
                gilrs::Axis::RightStickY => { state.right_stick.y_raw = (v * 127.0) as i8; },
                gilrs::Axis::DPadX => { 
                    if v < -0.5 { state.button_left = true; state.button_right = false; }
                    else if v > 0.5 { state.button_left = false; state.button_right = true; }
                    else { state.button_left = false; state.button_right = false; }
                },
                gilrs::Axis::DPadY => { 
                    if v < -0.5 { state.button_down = true; state.button_up = false; }
                    else if v > 0.5 { state.button_down = false; state.button_up = true; }
                    else { state.button_down = false; state.button_up = false; }
                },
                _ => {},
            }
        },
        gilrs::EventType::Connected => {
            state.connection = roboscope_ipc::snapshot::ControllerConnection::Vexnet;
        },
        gilrs::EventType::Disconnected => {
            state.connection = roboscope_ipc::snapshot::ControllerConnection::Offline;
        },
        _ => {},
    }
    state
}