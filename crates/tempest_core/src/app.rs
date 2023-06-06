use std::collections::{hash_map::RandomState, HashMap};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};

use tempest_ecs::world::World;
use tempest_render::renderer::Renderer;

pub type ApplicationStartCallback = for<'a> fn(ctx: &mut AppContext<'a>);
pub type ApplicationUpdateCallback = for<'a> fn(ctx: &mut AppContext<'a>);
pub type ApplicationStopCallback = for<'a> fn(ctx: &mut AppContext<'a>);

struct WindowInfo {
    name: String,
}

pub struct App {
    world: World,
    on_start: Vec<Box<ApplicationStartCallback>>,
    on_update: Vec<Box<ApplicationUpdateCallback>>,
    on_stop: Vec<Box<ApplicationStopCallback>>,
    window_titles: Vec<WindowInfo>,
}

#[derive(Default)]
pub struct AppBuilder {
    on_start: Vec<Box<ApplicationStartCallback>>,
    on_update: Vec<Box<ApplicationUpdateCallback>>,
    on_stop: Vec<Box<ApplicationStopCallback>>,
    windows: Vec<WindowInfo>,
}

pub struct AppContext<'a> {
    world: &'a mut World,
    events: &'a EventLoopWindowTarget<()>,
    renderers: &'a mut HashMap<WindowId, Renderer>,
}

impl<'a> AppContext<'a> {
    pub fn new(
        world: &'a mut World,
        events: &'a EventLoopWindowTarget<()>,
        renderers: &'a mut HashMap<WindowId, Renderer>,
    ) -> Self {
        Self {
            world: world,
            events: events,
            renderers: renderers,
        }
    }

    pub fn get_world(&self) -> &World {
        self.world
    }

    pub fn get_world_mut(&mut self) -> &mut World {
        self.world
    }

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
}

impl AppBuilder {
    pub fn on_app_start(&mut self, start: ApplicationStartCallback) -> &mut Self {
        self.on_start.push(Box::new(start));
        self
    }

    pub fn on_app_close(&mut self, close: ApplicationStopCallback) -> &mut Self {
        self.on_stop.push(Box::new(close));
        self
    }

    pub fn on_app_update(&mut self, update: ApplicationUpdateCallback) -> &mut Self {
        self.on_update.push(Box::new(update));
        self
    }

    pub fn with_window(&mut self, name: &str) -> &mut Self {
        assert!(!self.windows.iter().any(|info| info.name == name));

        self.windows.push(WindowInfo {
            name: name.to_owned(),
        });
        self
    }

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

                    renderers.iter_mut().for_each(|(_, renderer)| {
                        renderer.window().request_redraw();
                    });
                }
            }
            _ => {}
        });
    }
}
