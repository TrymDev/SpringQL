// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use crate::{
    error::Result,
    stream_engine::{
        autonomous_executor::row::value::sql_value::nn_sql_value::NnSqlValue,
        time::duration::event_duration::SpringEventDuration,
    },
};

use super::{SpringValue, ToNnSqlValue};

impl SpringValue for SpringEventDuration {
    fn try_from_duration(v: &SpringEventDuration) -> Result<Self> {
        Ok(*v)
    }
}

impl ToNnSqlValue for SpringEventDuration {
    fn into_sql_value(self) -> NnSqlValue {
        NnSqlValue::Duration(self)
    }
}
