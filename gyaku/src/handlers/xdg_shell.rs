use smithay::{delegate_xdg_shell, desktop::Window, wayland::shell::xdg::XdgShellHandler};

use crate::state::GyakuState;

impl XdgShellHandler for GyakuState {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), true);
    }

    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        todo!()
    }

    fn grab(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        seat: wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
        todo!()
    }

    fn new_client(&mut self, client: smithay::wayland::shell::xdg::ShellClient) {
        // ! Soft TODO
    }

    fn client_pong(&mut self, client: smithay::wayland::shell::xdg::ShellClient) {
        // ! Soft TODO
    }

    fn move_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::ToplevelSurface,
        seat: wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
        // ! Soft TODO
    }

    fn resize_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::ToplevelSurface,
        seat: wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
        edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
        // ! Soft TODO
    }

    fn maximize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // ! Soft TODO
    }

    fn unmaximize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // ! Soft TODO
    }

    fn fullscreen_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::ToplevelSurface,
        output: Option<wayland_server::protocol::wl_output::WlOutput>,
    ) {
        // ! Soft TODO
    }

    fn unfullscreen_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // ! Soft TODO
    }

    fn minimize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // ! Soft TODO
    }

    fn show_window_menu(
        &mut self,
        surface: smithay::wayland::shell::xdg::ToplevelSurface,
        seat: wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
        location: smithay::utils::Point<i32, smithay::utils::Logical>,
    ) {
        // ! Soft TODO
    }

    fn ack_configure(
        &mut self,
        surface: wayland_server::protocol::wl_surface::WlSurface,
        configure: smithay::wayland::shell::xdg::Configure,
    ) {
        // ! Soft TODO
    }

    fn reposition_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
        token: u32,
    ) {
        // ! Soft TODO
    }

    fn toplevel_destroyed(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // ! Soft TODO
    }

    fn popup_destroyed(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface) {
        // ! Soft TODO
    }
}

delegate_xdg_shell!(GyakuState);