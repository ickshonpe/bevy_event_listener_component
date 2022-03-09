use bevy::prelude::*;
use bevy_event_listener_component::*;

#[derive(Component, Debug)]
struct MyEvent(String);

fn spawn_listener(
    mut commands: Commands
) {
    let mut event_listener = EventListener::new();
    event_listener
        .add(|_world: &mut World, my_event: &MyEvent, _this_entity: Entity| {
            println!("recieved event: {:?}", my_event);
        });
    commands.spawn().insert(event_listener);
}

fn send_event(
    mut writer: EventWriter<MyEvent>,
) {
    writer.send(MyEvent("Hello, world!".to_string()));
}

fn main() {
    App::new()
    .add_plugins(MinimalPlugins)
    .add_plugin(EventListenerComponentPlugin)
    .add_event_and_listen::<MyEvent>()
    .add_startup_system(spawn_listener)
    .add_startup_system(send_event)
    .run();
}