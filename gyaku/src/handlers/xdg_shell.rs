use crate::shell::grabs::move_grab::MoveSurfaceGrab;
use crate::state::GyakuState;
use color_eyre::eyre::Context;
use color_eyre::Result;
use smithay::desktop::PopupKind;
use smithay::input::pointer::{Focus, GrabStartData};
use smithay::input::Seat;
use smithay::utils::Serial;
use smithay::wayland::compositor::with_states;
use smithay::wayland::seat::WaylandFocus;
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
use smithay::{delegate_xdg_shell, desktop::Window, wayland::shell::xdg::XdgShellHandler};
use tracing::{instrument, trace, trace_span};
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::Resource;

impl XdgShellHandler for GyakuState {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), false);
    }
    #[instrument(skip(self))]
    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        trace!("new_popup");
        trace_span!("new_popup").in_scope(|| {
            self.popup_manager
                .track_popup(PopupKind::from(surface))
                .unwrap();
        });

        // ! Soft TODO
    }

    fn grab(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        seat: wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
        // todo!()
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
        // TODO: From smallvil
        let seat = Seat::from_resource(&seat).unwrap();

        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let pointer = seat.get_pointer().unwrap();

            let window = self
                .space
                .elements()
                .find(|w| w.toplevel().wl_surface() == wl_surface)
                .unwrap()
                .clone();
            let initial_window_location = self.space.element_location(&window).unwrap();

            let grab = MoveSurfaceGrab {
                start_data,
                window,
                initial_window_location,
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
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

impl GyakuState {
    pub fn commit_xdg_shell_surface(&mut self, surface: &WlSurface) -> Option<()> {
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;

        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .ok()
                .map(|v| v.initial_configure_sent)
        })?;

        if !initial_configure_sent {
            window.toplevel().send_configure();
        }

        Some(())
    }
}

delegate_xdg_shell!(GyakuState);

fn check_grab(
    seat: &Seat<GyakuState>,
    surface: &WlSurface,
    serial: Serial,
) -> Option<GrabStartData<GyakuState>> {
    let pointer = seat.get_pointer()?;

    // Check that this surface has a click grab.
    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;

    let (focus, _) = start_data.focus.as_ref()?;
    // If the focus was for a different surface, ignore the request.
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }

    Some(start_data)
}
