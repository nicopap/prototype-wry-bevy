use std::fmt;

use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct PrintHierarchy<'w, 's> {
    query: Query<'w, 's, (DebugName, Option<&'static Children>)>,
}
impl<'w, 's> PrintHierarchy<'w, 's> {
    fn print<'a>(&'a self, root: Entity) -> PrintEntityHierarchy<'a, 'w, 's> {
        PrintEntityHierarchy(root, self)
    }
}
struct PrintEntityHierarchy<'a, 'w, 's>(Entity, &'a PrintHierarchy<'w, 's>);
impl fmt::Debug for PrintEntityHierarchy<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // unwrap: This query always is Ok for a living Entity
        let (n, children) = self.1.query.get(self.0).unwrap();
        let name = n
            .name
            .map_or_else(|| format!("Entity({:?})", n.entity), |n| n.to_string());
        if let Some(children) = children.filter(|c| !c.is_empty()) {
            let mut s = f.debug_tuple(&name);
            for &entry in children.iter() {
                s.field(&PrintEntityHierarchy(entry, self.1));
            }
            s.finish()
        } else {
            f.write_str(&name)
        }
    }
}
