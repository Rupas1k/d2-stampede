use crate::class::Class;
use crate::field::{FieldPath, FieldState};
use crate::field_value::FieldValue;
use crate::serializer::SerializerError;
use prettytable::{row, Table};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityEvents {
    Created,
    Updated,
    Deleted,
}

impl EntityEvents {
    #[inline]
    pub(crate) fn from_cmd(cmd: u32) -> Self {
        match cmd {
            0 => EntityEvents::Updated,
            2 => EntityEvents::Created,
            3 => EntityEvents::Deleted,
            _ => unreachable!(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EntityError {
    #[error("No entities found at index {0}")]
    IndexNotFound(usize),

    #[error("No entities found for handle {0}")]
    HandleNotFound(usize),

    #[error("No entities found for class with id {0}")]
    ClassIdNotFound(i32),

    #[error("No entities found for class with name {0}")]
    ClassNameNotFound(String),

    #[error("No property found for name {0} (Class: {1}, FieldPath: {2})")]
    PropertyNameNotFound(String, String, String),

    #[error(transparent)]
    FieldPathNotFound(#[from] SerializerError),
}

/// Container for entities.
pub struct Entities {
    pub(crate) entities_vec: Vec<Option<Entity>>,
}

impl Default for Entities {
    fn default() -> Self {
        Entities {
            entities_vec: vec![None; 8192],
        }
    }
}

impl Entities {
    /// Iterator over all entities.
    /// # Examples
    ///
    /// ```
    /// use d2_stampede::prelude::*;
    /// use d2_stampede::proto::*;
    ///
    /// #[derive(Default)]
    /// struct MyObs;
    ///
    /// impl Observer for MyObs {
    ///     fn on_tick_start(&mut self, ctx: &Context) -> ObserverResult {
    ///         let dire_heroes = ctx
    ///             .entities()
    ///             .iter()
    ///             .filter(|&e| {
    ///                 e.class().name().starts_with("CDOTA_Hero_Unit")
    ///                     && try_property!(e, u32, "m_iTeamNum") == Some(3)
    ///                     && try_property!(e, u32, "m_hReplicatingOtherHeroModel") == Some(u32::MAX)
    ///             })
    ///             .collect::<Vec<_>>();
    ///         Ok(())
    ///     }
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities_vec.iter().flatten()
    }

    /// Returns [`Entity`] for given index.
    pub fn get_by_index(&self, index: usize) -> Result<&Entity, EntityError> {
        self.entities_vec
            .get(index)
            .and_then(|x| x.as_ref())
            .ok_or(EntityError::IndexNotFound(index))
    }

    /// Returns [`Entity`] for given handle.
    pub fn get_by_handle(&self, handle: usize) -> Result<&Entity, EntityError> {
        self.get_by_index(handle & 0x3fff)
            .map_err(|_| EntityError::HandleNotFound(handle))
    }

    /// Returns [`Entity`] for given class id.
    pub fn get_by_class_id(&self, id: i32) -> Result<&Entity, EntityError> {
        self.iter()
            .find(|&entity| entity.class().id() == id)
            .ok_or(EntityError::ClassIdNotFound(id))
    }

    /// Returns [`Entity`] for given class name.
    pub fn get_by_class_name(&self, name: &str) -> Result<&Entity, EntityError> {
        self.iter()
            .find(|&entity| entity.class().name() == name)
            .ok_or(EntityError::ClassNameNotFound(name.to_string()))
    }
}

#[derive(Clone)]
pub struct Entity {
    index: u32,
    serial: u32,
    pub(crate) class: Rc<Class>,
    pub(crate) state: FieldState,
}

impl Entity {
    pub(crate) fn new(index: u32, serial: u32, class: Rc<Class>, state: FieldState) -> Self {
        Entity {
            index,
            serial,
            class,
            state,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn serial(&self) -> u32 {
        self.serial
    }

    pub fn handle(&self) -> u32 {
        self.serial << 14 | self.index
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    /// Returns [`FieldValue`] for given property name. You can also use
    /// [`property!`] and [`try_property!`] macros.
    ///
    /// # Examples
    ///
    /// ```
    /// use d2_stampede::prelude::*;
    ///
    /// #[derive(Default)]
    /// struct MyObs;
    ///
    /// impl Observer for MyObs {
    ///     fn on_entity(
    ///         &mut self,
    ///         ctx: &Context,
    ///         event: EntityEvents,
    ///         entity: &Entity,
    ///     ) -> ObserverResult {
    ///         let x: u8 = entity
    ///             .get_property_by_name("CBodyComponent.m_cellX")?
    ///             .try_into()?;
    ///
    ///         let y: u8 = property!(entity, "CBodyComponent.m_cellY");
    ///
    ///         let z = property!(entity, u8, "CBodyComponent.m_cellY");
    ///
    ///         Ok(())
    ///     }
    /// }
    /// ```
    ///
    /// [`property!`]: crate::property
    /// [`try_property!`]: crate::try_property
    pub fn get_property_by_name(&self, name: &str) -> Result<&FieldValue, EntityError> {
        self.get_property_by_field_path(&self.class.serializer.get_field_path_for_name(name)?)
    }

    pub(crate) fn get_property_by_field_path(
        &self,
        fp: &FieldPath,
    ) -> Result<&FieldValue, EntityError> {
        self.state.get_value(fp).ok_or_else(|| {
            EntityError::PropertyNameNotFound(
                self.class.serializer.get_name_for_field_path(fp),
                self.class.name().to_string(),
                format!("{}", fp),
            )
        })
    }
}

impl Display for Entities {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.add_row(row!["idx", "serial", "handle", "class"]);
        for e in self.entities_vec.iter().flatten() {
            table.add_row(row![
                e.index().to_string(),
                e.serial().to_string(),
                e.handle().to_string(),
                e.class().name(),
            ]);
        }
        write!(f, "{}", table)
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table.add_row(row!["#", "Field", "Type", "Value"]);

        for fp in self
            .class
            .serializer
            .get_field_paths(&mut FieldPath::new(), &self.state)
        {
            let field_type = self.class.serializer.get_type_for_field_path(&fp);
            let name = self.class.serializer.get_name_for_field_path(&fp);
            let value = self.state.get_value(&fp);
            if let Some(v) = value {
                table.add_row(row![fp, name, field_type.as_string(), format!("{:?}", v)]);
            } else {
                table.add_row(row![fp, name, field_type.as_string(), "None"]);
            }
        }

        write!(f, "{}", table)
    }
}
