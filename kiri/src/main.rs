use std::{os::unix::io::OwnedFd, sync::Arc};

use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent, Axis, PointerAxisEvent, AxisSource, PointerButtonEvent, ButtonState, Event},
        renderer::{
            element::surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
            gles2::Gles2Renderer,
            utils::{draw_render_elements, on_commit_buffer_handler},
            Frame, Renderer,
        },
        winit::{self, WinitEvent},
    },
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{keyboard::FilterResult, Seat, SeatHandler, SeatState, pointer::{AxisFrame, ButtonEvent}},
    reexports::wayland_server::{protocol::wl_seat, Display},
    utils::{Rectangle, Serial, Transform, SERIAL_COUNTER},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            with_surface_tree_downward, CompositorHandler, CompositorState, SurfaceAttributes,
            TraversalAction,
        },
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
    },
};
use wayland_protocols::xdg::shell::server::xdg_toplevel;
use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::{
        wl_buffer,
        wl_surface::{self, WlSurface},
    },
    ListeningSocket,
};

use tracing::{debug, error, info, trace, warn};

impl BufferHandler for App {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for App {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        // Handle popup creation here
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Handle popup grab here
    }
}

impl DataDeviceHandler for App {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for App {}
impl ServerDndGrabHandler for App {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd) {}
}

impl CompositorHandler for App {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler(surface);
    }
}

impl ShmHandler for App {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for App {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
}

struct App {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,
    data_device_state: DataDeviceState,

    seat: Seat<Self>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::config::HookBuilder::default()
        .add_default_filters()
        .panic_section("It's not that I won't do it, I just can't!")
        .install()?;

    pretty_env_logger::formatted_builder()
        .filter_level(tracing::log::LevelFilter::Debug)
        .init();
    run_winit()
}

fn log() -> ::slog::Logger {
    use tracing_slog::TracingSlogDrain;
    let drain = TracingSlogDrain;
    ::slog::Logger::root(drain, slog::o!())
}

pub fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    let log = log();

    let mut display: Display<App> = Display::new()?;
    let dh = display.handle();

    let compositor_state = CompositorState::new::<App, _>(&dh, None);
    let shm_state = ShmState::new::<App, _>(&dh, vec![], None);
    let mut seat_state = SeatState::new();
    let seat = seat_state.new_wl_seat(&dh, "winit", None);

    let mut state = {
        App {
            compositor_state,
            xdg_shell_state: XdgShellState::new::<App, _>(&dh, None),
            shm_state,
            seat_state,
            data_device_state: DataDeviceState::new::<App, _>(&dh, None),
            seat,
        }
    };

    let listener = ListeningSocket::bind("wayland-5").unwrap();
    let mut clients = Vec::new();

    let (mut backend, mut winit) = winit::init::<Gles2Renderer, _>(None)?;

    let start_time = std::time::Instant::now();

    let keyboard = state
        .seat
        .add_keyboard(Default::default(), 200, 200)
        .unwrap();

    std::env::set_var("WAYLAND_DISPLAY", "wayland-5");
    std::process::Command::new("weston-terminal").spawn().ok();

    loop {
        winit.dispatch_new_events(|event| match event {
            WinitEvent::Resized { .. } => {}
            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    keyboard.input::<(), _>(
                        &mut state,
                        event.key_code(),
                        event.state(),
                        0.into(),
                        0,
                        |_, _, _| {
                            //
                            FilterResult::Forward
                        },
                    );
                }
                InputEvent::PointerMotionAbsolute { .. } => {
                    if let Some(surface) = state
                        .xdg_shell_state
                        .toplevel_surfaces(|surfaces| surfaces.iter().next().cloned())
                    {
                        let surface = surface.wl_surface().clone();
                        keyboard.set_focus(&mut state, Some(surface), 0.into());
                    };
                }
                InputEvent::PointerButton { event, .. } => {
                    let pointer = self.seat.get_pointer().unwrap();
                    let keyboard = self.seat.get_keyboard().unwrap();

                    let serial = SERIAL_COUNTER.next_serial();

                    let button = event.button_code();

                    let button_state = event.state();

                    if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
                        if let Some((window, _loc)) = self
                            .space
                            .element_under(pointer.current_location())
                            .map(|(w, l)| (w.clone(), l))
                        {
                            self.space.raise_element(&window, true);
                            keyboard.set_focus(
                                self,
                                Some(window.toplevel().wl_surface().clone()),
                                serial,
                            );
                            self.space.elements().for_each(|window| {
                                window.toplevel().send_configure();
                            });
                        } else {
                            self.space.elements().for_each(|window| {
                                window.set_activated(false);
                                window.toplevel().send_configure();
                            });
                            keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                        }
                    };

                    pointer.button(
                        self,
                        &ButtonEvent {
                            button,
                            state: button_state,
                            serial,
                            time: event.time(),
                        },
                    );
                }
                InputEvent::PointerAxis { event, .. } => {
                    let source = event.source();

                    let horizontal_amount = event
                        .amount(Axis::Horizontal)
                        .unwrap_or_else(|| event.amount_discrete(Axis::Horizontal).unwrap() * 3.0);
                    let vertical_amount = event
                        .amount(Axis::Vertical)
                        .unwrap_or_else(|| event.amount_discrete(Axis::Vertical).unwrap() * 3.0);
                    let horizontal_amount_discrete = event.amount_discrete(Axis::Horizontal);
                    let vertical_amount_discrete = event.amount_discrete(Axis::Vertical);

                    let mut frame = AxisFrame::new(event.time()).source(source);
                    if horizontal_amount != 0.0 {
                        frame = frame.value(Axis::Horizontal, horizontal_amount);
                        if let Some(discrete) = horizontal_amount_discrete {
                            frame = frame.discrete(Axis::Horizontal, discrete as i32);
                        }
                    } else if source == AxisSource::Finger {
                        frame = frame.stop(Axis::Horizontal);
                    }
                    if vertical_amount != 0.0 {
                        frame = frame.value(Axis::Vertical, vertical_amount);
                        if let Some(discrete) = vertical_amount_discrete {
                            frame = frame.discrete(Axis::Vertical, discrete as i32);
                        }
                    } else if source == AxisSource::Finger {
                        frame = frame.stop(Axis::Vertical);
                    }

                    self.seat.get_pointer().unwrap().axis(self, frame);
                }
                _ => {}
            },

            _ => (),
        })?;

        backend.bind().unwrap();

        let size = backend.window_size().physical_size;
        let damage = Rectangle::from_loc_and_size((0, 0), size);

        let elements = state.xdg_shell_state.toplevel_surfaces(|surfaces| {
            surfaces
                .iter()
                .flat_map(|surface| {
                    render_elements_from_surface_tree(
                        backend.renderer(),
                        surface.wl_surface(),
                        (0, 0),
                        1.0,
                        log.clone(),
                    )
                })
                .collect::<Vec<WaylandSurfaceRenderElement<Gles2Renderer>>>()
        });

        let mut frame = backend
            .renderer()
            .render(size, Transform::Flipped180)
            .unwrap();
        frame.clear([0.1, 0.0, 0.0, 1.0], &[damage]).unwrap();
        draw_render_elements(&mut frame, 1.0, &elements, &[damage], &log).unwrap();
        frame.finish().unwrap();

        state.xdg_shell_state.toplevel_surfaces(|surfaces| {
            for surface in surfaces {
                send_frames_surface_tree(
                    surface.wl_surface(),
                    start_time.elapsed().as_millis() as u32,
                );
            }
        });

        if let Some(stream) = listener.accept()? {
            println!("Got a client: {:?}", stream);

            let client = display
                .handle()
                .insert_client(stream, Arc::new(ClientState))
                .unwrap();
            clients.push(client);
        }

        display.dispatch_clients(&mut state)?;
        display.flush_clients()?;

        // It is important that all events on the display have been dispatched and flushed to clients before
        // swapping buffers because this operation may block.
        backend.submit(Some(&[damage])).unwrap();
    }
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been commited
            for callback in states
                .cached_state
                .current::<SurfaceAttributes>()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

struct ClientState;
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}

// Macros used to delegate protocol handling to types in the app state.
delegate_xdg_shell!(App);
delegate_compositor!(App);
delegate_shm!(App);
delegate_seat!(App);
delegate_data_device!(App);