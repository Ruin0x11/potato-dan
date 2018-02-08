use calx_ecs::Entity;
use world::World;

pub mod gui;

mod text {
    make_global!(DEBUG_TEXT, Option<String>, None);
}

pub fn pop_text() -> Option<String> {
    let ret = text::instance::with(|t| t.clone());
    text::instance::with_mut(|t| *t = None);
    ret
}

pub fn add_text(text: String) {
    text::instance::with_mut(|t| {
        let next = match t {
            &mut Some(ref tex) => format!("{}\n{}", tex, text),
            &mut None => text,
        };
        *t = Some(next)
    });
}

mod entity {
    use calx_ecs::Entity;
    make_global!(TARGET_ENTITY, Option<Entity>, None);
}

pub fn follow_entity(entity: Option<Entity>) {
    entity::instance::with_mut(|e| *e = entity);
}

fn entity_info(entity: Entity, world: &World) -> String {
    String::new()
}

pub fn update(world: &World) {
    entity::instance::with_mut(|e| if e.map_or(true, |en| !world.contains(en)) {
                                   *e = None;
                                   add_text(String::new());
                               });
    entity::instance::with(|e| if let &Some(entity) = e {
                               add_text(entity_info(entity, world));
                           });
}
