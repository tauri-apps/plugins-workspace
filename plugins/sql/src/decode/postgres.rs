// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use rust_decimal::prelude::ToPrimitive;
use serde_json::Value as JsonValue;
use sqlx::{postgres::PgValueRef, TypeInfo, Value, ValueRef};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use uuid::Uuid;

use crate::Error;

pub(crate) fn to_json(v: PgValueRef) -> Result<JsonValue, Error> {
    if v.is_null() {
        return Ok(JsonValue::Null);
    }

    let res = match v.type_info().name() {
        "CHAR" | "VARCHAR" | "TEXT" | "NAME" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode() {
                JsonValue::String(v)
            } else {
                JsonValue::Null
            }
        }
        "UUID" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<Uuid>() {
                JsonValue::String(v.to_string())
            } else {
                JsonValue::Null
            }
        }
        "FLOAT4" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<f32>() {
                JsonValue::from(v)
            } else {
                JsonValue::Null
            }
        }
        "FLOAT8" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<f64>() {
                JsonValue::from(v)
            } else {
                JsonValue::Null
            }
        }
        "INT2" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<i16>() {
                JsonValue::Number(v.into())
            } else {
                JsonValue::Null
            }
        }
        "INT4" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<i32>() {
                JsonValue::Number(v.into())
            } else {
                JsonValue::Null
            }
        }
        "INT8" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<i64>() {
                JsonValue::Number(v.into())
            } else {
                JsonValue::Null
            }
        }
        "BOOL" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode() {
                JsonValue::Bool(v)
            } else {
                JsonValue::Null
            }
        }
        "DATE" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<Date>() {
                JsonValue::String(v.to_string())
            } else {
                JsonValue::Null
            }
        }
        "TIME" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<Time>() {
                JsonValue::String(v.to_string())
            } else {
                JsonValue::Null
            }
        }
        "TIMESTAMP" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<PrimitiveDateTime>() {
                JsonValue::String(v.to_string())
            } else {
                JsonValue::Null
            }
        }
        "TIMESTAMPTZ" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<OffsetDateTime>() {
                JsonValue::String(v.to_string())
            } else {
                JsonValue::Null
            }
        }
        "JSON" | "JSONB" => ValueRef::to_owned(&v).try_decode().unwrap_or_default(),
        "BYTEA" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<Vec<u8>>() {
                JsonValue::Array(v.into_iter().map(|n| JsonValue::Number(n.into())).collect())
            } else {
                JsonValue::Null
            }
        }
        "NUMERIC" => {
            if let Ok(v) = ValueRef::to_owned(&v).try_decode::<rust_decimal::Decimal>() {
                if let Some(n) = v.to_f64().and_then(serde_json::Number::from_f64) {
                    JsonValue::Number(n)
                } else {
                    JsonValue::String(v.to_string())
                }
            } else {
                JsonValue::Null
            }
        }
        "VOID" => JsonValue::Null,
        // Handle custom types (enums, domains, etc.) by trying to decode as string
        _ => {
            let type_name = v.type_info().name().to_string();
            if let Ok(v) = ValueRef::to_owned(&v).try_decode_unchecked::<String>() {
                log::warn!("unsupported type {type_name} decoded as string");
                JsonValue::String(v)
            } else {
                return Err(Error::UnsupportedDatatype(v.type_info().name().to_string()));
            }
        }
    };

    Ok(res)
}
