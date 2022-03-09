use bevy::prelude::*;
use bevy_event_listener_component::*;

#[derive(Component, Debug)]
struct MyEvent(i32);

#[derive(Component, Debug)]
struct MyTotal(i32);

fn main() {
    App::new()
    .add_plugin(EventListenerComponentPlugin)
    .add_event_and_listen::<MyEvent>()
    .add_startup_system(|mut commands: Commands| {
        commands.spawn_bundle((
            MyTotal(0),
            EventListener::mutator(|event: &MyEvent, total: &mut MyTotal| {
                total.0 += event.0;
                println!("Recieved MyEvent, MyTotal = {}", total.0)
            }),
        ));
    })
    .add_startup_system(|mut writer: EventWriter<MyEvent>| { 
        for x in 0..5 { writer.send(MyEvent(x)); }
    })
    .run();
}