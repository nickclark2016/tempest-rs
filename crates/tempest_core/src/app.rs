use std::collections::{hash_map::RandomState, HashMap};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};

use tempest_ecs::world::World;
use tempest_render::renderer::Renderer;

/// Callback to be invoked on the start of an application.
pub type ApplicationStartCallback = for<'a> fn(ctx: &mut AppContext<'a>);

/// Callback to be invoked at every tick of an application.
pub type ApplicationUpdateCallback = for<'a> fn(ctx: &mut AppContext<'a>);

/// Callback to be invoked on the exitt of an application.
pub type ApplicationStopCallback = for<'a> fn(ctx: &mut AppContext<'a>);

struct WindowInfo {
    name: String,
}

/// Struct representing an application and its current state.
pub struct App {
    world: World,
    on_start: Vec<Box<ApplicationStartCallback>>,
    on_update: Vec<Box<ApplicationUpdateCallback>>,
    on_stop: Vec<Box<ApplicationStopCallback>>,
    window_titles: Vec<WindowInfo>,
}

/// Builder used to create an application.
/// 
/// # Example
/// ```rust
/// use tempest_core::app::AppBuilder;
/// 
/// AppBuilder::default().with_window("Hello, World!")
///     .on_app_start(|_| { println!("Started!"); })
///     .on_app_update(|ctx| {
///         println!("Updating!"); ctx.request_shutdown();
///     })
///     .on_app_close(|_| { println!("Closing!"); })
///     .build()
///     .run();
/// ```
#[derive(Default)]
pub struct AppBuilder {
    on_start: Vec<Box<ApplicationStartCallback>>,
    on_update: Vec<Box<ApplicationUpdateCallback>>,
    on_stop: Vec<Box<ApplicationStopCallback>>,
    windows: Vec<WindowInfo>,
}

/// Struct wrapping application context.  This is used to provide data from the app to user-defined callbacks.
/// 
/// ## Lifetimes
/// - `<'a>` - Lifetime of the [App](App) created from
pub struct AppContext<'a> {
    world: &'a mut World,
    events: &'a EventLoopWindowTarget<()>,
    renderers: &'a mut HashMap<WindowId, Renderer>,
    shutdown_requested: bool,
}

impl<'a> AppContext<'a> {
    /// Constructs a new instance of a context from a world, an event loop, and a set of renderers.
    pub fn new(
        world: &'a mut World,
        events: &'a EventLoopWindowTarget<()>,
        renderers: &'a mut HashMap<WindowId, Renderer>,
    ) -> Self {
        Self {
            world: world,
            events: events,
            renderers: renderers,
            shutdown_requested: false
        }
    }

    /// Fetches an immutable reference to the world
    pub fn get_world(&self) -> &World {
        self.world
    }

    /// Fetches a mutable reference to the world
    pub fn get_world_mut(&mut self) -> &mut World {
        self.world
    }

    /// Creates a new window for the application with the provided name
    pub fn create_window(&mut self, name: &str) {
        assert!(!self
            .renderers
            .iter()
            .any(|(_, renderer)| renderer.window().title() == name));

        let win = WindowBuilder::new().with_title(name).build(self.events);
        let window_create = || async { Renderer::new(win.unwrap()).await };
        let renderer = pollster::block_on(window_create());
        self.renderers.insert(renderer.window().id(), renderer);
    }

    /// Closes the window with the provided name
    pub fn close_window(&mut self, name: &str) {
        let id_opt = self
            .renderers
            .iter()
            .find(|it| it.1.window().title() == name)
            .map(|(id, _)| *id);

        if let Some(id) = id_opt {
            self.renderers.remove(&id);
        }
    }

    /// Requests the application to shut down
    pub fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }
}

impl AppBuilder {
    /// Adds a callback to the built application to be invoked before entering the main application loop, but after the initialization of the application
    pub fn on_app_start(&mut self, start: ApplicationStartCallback) -> &mut Self {
        self.on_start.push(Box::new(start));
        self
    }

    /// Adds a callback to the built application to be invoked after leaving the main application loop, but before destruction of the application
    pub fn on_app_close(&mut self, close: ApplicationStopCallback) -> &mut Self {
        self.on_stop.push(Box::new(close));
        self
    }

    /// Adds a callback to the built application to be invoked on every tick of the application
    pub fn on_app_update(&mut self, update: ApplicationUpdateCallback) -> &mut Self {
        self.on_update.push(Box::new(update));
        self
    }

    /// Adds a window to the built application with the provided name
    pub fn with_window(&mut self, name: &str) -> &mut Self {
        assert!(!self.windows.iter().any(|info| info.name == name));

        self.windows.push(WindowInfo {
            name: name.to_owned(),
        });
        self
    }

    /// Builds an application from the contents of the builder
    pub fn build(&mut self) -> App {
        App {
            world: World::default(),
            on_start: self.on_start.drain(..).collect(),
            on_update: self.on_update.drain(..).collect(),
            on_stop: self.on_stop.drain(..).collect(),
            window_titles: self.windows.drain(..).collect(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            world: World::default(),
            on_start: Vec::default(),
            on_update: Vec::default(),
            on_stop: Vec::default(),
            window_titles: Vec::default(),
        }
    }
}

impl App {
    /// Runs the application.  This call assumes indefinite control the calling thread until destruction of the thread of application.
    pub fn run(self) {
        pollster::block_on(self.run_internal());
    }

    async fn run_internal(mut self) {
        let event_loop = EventLoop::new();

        let mut renderers = HashMap::<WindowId, Renderer, RandomState>::default();
        for winfo in &self.window_titles {
            let win = WindowBuilder::new()
                .with_title(&winfo.name)
                .build(&event_loop)
                .unwrap();
            let win_id = win.id();
            let renderer = Renderer::new(win).await;
            renderers.insert(win_id, renderer);
        }

        let mut ctx = AppContext::new(&mut self.world, &event_loop, &mut renderers);

        for cb in &self.on_start {
            cb(&mut ctx);
        }

        // make sure we have at least one window before getting here
        assert!(!renderers.is_empty());

        event_loop.run(move |event, event_loop, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if let Some(renderer) = renderers.get_mut(&window_id) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => {
                            let mut ctx =
                                AppContext::new(&mut self.world, &event_loop, &mut renderers);

                            for cb in &self.on_stop {
                                cb(&mut ctx);
                            }
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) => {
                if let Some(renderer) = renderers.get_mut(&window_id) {
                    match renderer.draw(&self.world.entities()) {
                        Ok(_) => {}
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            Event::MainEventsCleared => {
                if *control_flow != ControlFlow::Exit {
                    let mut ctx = AppContext::new(&mut self.world, &event_loop, &mut renderers);

                    for cb in &self.on_update {
                        cb(&mut ctx);
                    }

                    if ctx.shutdown_requested {
                        *control_flow = ControlFlow::Exit;
                    }

                    renderers.iter_mut().for_each(|(_, renderer)| {
                        renderer.window().request_redraw();
                    });
                }
            }
            _ => {}
        });
    }
}
