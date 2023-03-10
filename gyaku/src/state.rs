use slog::Logger;
use smithay::{
    delegate_output,
    desktop::{PopupManager, Space, Window},
    input::{Seat, SeatState},
    wayland::{
        compositor::CompositorState, data_device::DataDeviceState, shell::xdg::XdgShellState,
        shm::ShmState,
    }
};
use tracing::instrument;
use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    Display,
};

/// State of the compositor
// GOD
// todo: refactor this whole ass thing, this is a god object
pub struct GyakuState {
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) shm_state: ShmState,
    pub(crate) seat_state: SeatState<Self>,
    pub(crate) data_device_state: DataDeviceState,

    pub(crate) space: Space<Window>,
    // TODO: periodically cleanup popup state, see https://smithay.github.io/smithay/smithay/desktop/struct.PopupManager.html#method.cleanup
    pub(crate) popup_manager: PopupManager,
    pub(crate) log: Logger,
    pub(crate) seat: Seat<Self>,
}

impl GyakuState {
    pub fn new(display: &mut Display<Self>, log: Logger) -> Self {
        let display_handle = display.handle();
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "seat-0", log.clone());
        // todo: add hook into event loop, see `event_loop.rs` for code

        Self {
            compositor_state: CompositorState::new::<Self, _>(&display_handle, log.clone()),
            xdg_shell_state: XdgShellState::new::<Self, _>(&display_handle, log.clone()),
            shm_state: ShmState::new::<Self, _>(&display_handle, vec![], log.clone()),
            seat_state,
            data_device_state: DataDeviceState::new::<Self, _>(&display_handle, log.clone()),

            space: Space::new(log.clone()),
            popup_manager: PopupManager::new(log.clone()),
            log: log.clone(),
            seat,
        }
    }
}

delegate_output!(GyakuState);

// Client state... might want to move this to another file for cleaniness later

#[derive(Debug, Default)]
pub struct ClientState;
impl ClientData for ClientState {
    /// Notification that a client was initialized
    #[instrument(skip(self))]
    fn initialized(&self, _client_id: ClientId) {}
    /// Notification that a client is disconnected
    #[instrument(skip(self))]
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
