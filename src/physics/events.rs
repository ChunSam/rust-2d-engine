use crate::ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEvent {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}
