// src/models.rs

//use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/*
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct InsertablePerson {
    pub full_name: String,
}

impl FromStr for InsertablePerson {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InsertablePerson {
            full_name: "".to_string(),
        })
    }
}
*/

// this struct will be used to represent database record
#[derive(Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct Person {
    //#[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub full_name: String,
}


impl Person {
    pub fn compare(&self, pers: &Person) -> bool {
        if self.full_name == pers.full_name {
            true
        } else {
            false
        }
    }
}

#[derive(
Debug,
Clone,
Deserialize,
Serialize,
PartialEq,
FromRow,
)]
pub struct Genre {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub name: String,
}

#[derive(
Debug, Clone, Deserialize, Serialize, FromRow,
)]
pub struct Partition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub person_id: i32,
    pub genre_id: i32,
}

// une struct pour présenter les partitions avec les
// éléments des différentes tables
//
#[derive(Debug, Serialize, Deserialize)]
pub struct ShowPartition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub full_name: String,
    pub name: String,
}




