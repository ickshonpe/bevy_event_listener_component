use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use bevy::app::Events;
use bevy::app::ManualEventReader;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::utils::HashMap;


pub trait HandlesEvent<E> 
where
    E: 'static + Send + Sync
{
    fn on_event(&mut self, world: &mut World, event: &E, entity: Entity);
}

pub struct EventHandler<E, F>
where 
    F: FnMut(&mut World, &E, Entity)
{
    closure: F,
    phantom: PhantomData<dyn Fn() -> E + 'static + Send + Sync>
}

impl <E, F> EventHandler<E, F> 
where 
    F: FnMut(&mut World, &E, Entity)
{
    pub fn new(f: F) -> Self {
        Self {
            closure: f,
            phantom: PhantomData::default()
        }
    }
}

impl <E, F> HandlesEvent<E> for EventHandler<E, F> 
where 
    E: 'static + Send + Sync,
    F: FnMut(&mut World, &E, Entity) + 'static + Send + Sync
{
    fn on_event(&mut self, world: &mut World, event: &E, entity: Entity) {
        (self.closure)(world, event, entity);
    }
}

#[derive(Component)]
pub struct EventListener<E> 
where
    E: 'static + Send + Sync
{
    list: Arc<Mutex<Vec<Box<dyn HandlesEvent<E> + 'static + Send + Sync>>>>,
}

impl <E> Default for EventListener<E>
where
    E: 'static + Send + Sync
{
    fn default() -> Self {
        Self { list: Default::default() }
    }
}

impl <E> EventListener<E> 
where
    E: 'static + Send + Sync
{
    pub fn new() -> Self {
        EventListener::default()
    }

    pub fn mutator<C: Component>(
        mut f: impl FnMut(&E, &mut C) + 'static + Sync + Send) -> Self {
        let listener = Self::default();
        let h = EventHandler::new(move |world: &mut World, event: &E, entity: Entity| {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut component) = entity_mut.get_mut::<C>() {
                    f(event, &mut component);
                }
            }
        });
        listener.list.lock().unwrap().push(Box::new(h));
        listener
    }

    pub fn add<F>(&mut self, f: F) -> &mut Self
    where 
        F: FnMut(&mut World, &E, Entity) + 'static + Send + Sync
    {
        let handler = EventHandler::new(f);
        self.list.lock().unwrap().push(Box::new(handler));
        self
    }
}

impl <E, F> From<F> for EventListener<E> 
where
    F: FnMut() + 'static + Send + Sync,
    E: 'static + Send + Sync,
{
    fn from(mut f: F) -> Self {
        let mut l = Self::new();
        l.add(move |_, _, _| f());
        l
    }
}

pub trait EventProcessor : 'static + Send + Sync {
    fn process_event(&mut self, world: &mut World);
}

pub struct Processor<E> 
where 
    E: 'static + Send + Sync
{
    manual_event_reader: ManualEventReader<E>
}

impl <E> Default for Processor<E> 
where
    E: 'static + Send + Sync
{        
    fn default() -> Self {
        Self { manual_event_reader: Default::default() }
    }
}


impl <E> EventProcessor for Processor<E> 
where 
    E: 'static + Send + Sync
{
    fn process_event(&mut self, world: &mut World) {
        world.resource_scope(|world, events: Mut<Events<E>>| {
            let mut system_state = SystemState::<Query<Entity, With<EventListener<E>>>>::new(world);
            let entities: Vec<Entity> = system_state.get(world).iter().collect();
            for event in self.manual_event_reader.iter(&events) {
                for &entity in entities.iter() {
                    if let Some(listener) = world.get_mut::<EventListener<E>>(entity) {
                        let arc = listener.list.clone();
                        let mut list = arc.lock().unwrap();
                        for handler in list.iter_mut() {
                            handler.on_event(world, event, entity);
                        }
                    }

                }   
            }
        });
    }
}

#[derive(Default)]
pub struct EventProcessors {
    map: HashMap<TypeId, Box<dyn EventProcessor>>,
}

pub fn update_event_listeners(
    world: &mut World,
) {
    world.resource_scope(|world, mut processors: Mut<EventProcessors>| {
        for processor in processors.map.values_mut() {
            processor.process_event(world);
        }
    });
}

pub trait AddEventListenerExt {
    fn add_event_and_listen<E>(&mut self) -> &mut Self
    where
        E: 'static + Send + Sync;
}

impl AddEventListenerExt for App {
    fn add_event_and_listen<E>(&mut self) -> &mut Self
    where
        E: 'static + Send + Sync
    {   
        self.add_event::<E>();     
        self.world.get_resource_or_insert_with(EventProcessors::default)
        .map.insert(TypeId::of::<E>(), Box::new(Processor::<E>::default()));
        self
    }
}

pub struct EventListenerComponentPlugin;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, SystemLabel)]
pub struct UpdateGenericEventListeners;

impl Plugin for EventListenerComponentPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_event_listeners.exclusive_system().at_end()
            .label(UpdateGenericEventListeners)
        )
        ;
    }
}