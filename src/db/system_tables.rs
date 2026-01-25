use super::UUID;
use crate::db::{TableReference, ZeroTable};

#[derive(Debug, crate::ZeroTable)]
pub struct User {
    first_name: String,
    last_name: String,
    email: String,
}

impl User {
    pub const SYSTEM: TableReference<User> = TableReference {
        z_uuid: UUID {
            data_1: 26997595,
            data_2: 17129,
            data_3: 63502,
            data_4: [0, 0, 0, 0, 0, 0, 0, 0],
        },
        _ty: std::marker::PhantomData,
    };
}

#[derive(Debug, crate::ZeroTable)]
pub struct UserV1 {
    first_name: String,
    last_name: String,
}

pub trait TableVersion<T: ZeroTable>: ZeroTable {
    fn current_table_name() -> &'static str {
        Self::table_name()
    }
    fn map_to_current(old: T) -> Self;
}

impl TableVersion<UserV1> for User {
    fn map_to_current(old: UserV1) -> User {
        User {
            first_name: old.first_name,
            last_name: old.last_name,
            email: String::default(),
        }
    }
}
