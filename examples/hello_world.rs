use tempest_core::app::AppBuilder;
use tempest_ecs::{component::Component, registry::RegistryQuery};

#[derive(Component)]
struct AppComponent;

#[derive(Component)]
struct AppComponent2;

#[derive(RegistryQuery)]
#[read_only(AppComponent)]
struct AppQuery;

fn main() {
    AppBuilder::default()
        .with_window("Tempest Sandbox Application")
        .on_app_start(|ctx| {
            let registry = ctx.get_world_mut().entitites_mut();
            let entity = registry.create_entity();
            registry.assign_component(entity, AppComponent {});
        })
        .on_app_update(
            |ctx| {
                for _ in ctx.get_world().entities().query_registry::<AppQuery>() {}
            },
        )
        .on_app_close(|_| ())
        .build()
        .run();
}
