use tokio::{
    time::Duration
};
use std::{time::{SystemTime, UNIX_EPOCH}, convert::TryInto};
use crate::frame::Frame;
use mlua::{
    Value as LuaValue,
    Lua,
};

pub fn resp_ok() -> Frame {
    Frame::Simple("OK".to_string())
}

pub fn resp_invalid_arguments() -> Frame {
    Frame::Simple("Invalid arguments".to_string())
}

pub fn resp_err(e: &str) -> Frame {
    Frame::Error(e.to_string())
}

pub fn resp_sstr(val: &'static str) -> Frame {
    Frame::Simple(val.to_string())
}

pub fn resp_str(val: &str) -> Frame {
    Frame::Simple(val.to_string())
}

pub fn resp_int(val: i64) -> Frame {
    Frame::Integer(val)
}

pub fn resp_bulk(val: Vec<u8>) -> Frame {
    Frame::Bulk(val.into())
}

pub fn resp_nil() -> Frame {
    Frame::Null
}

pub fn resp_array(val: Vec<Frame>) -> Frame {
    Frame::Array(val.into())
}

pub async fn sleep(ms: u32) {
    tokio::time::sleep(Duration::from_millis(ms as u64)).await;
}

pub fn lua_resp_to_redis_resp(resp: LuaValue) -> Frame {
    match resp {
        LuaValue::String(r) => {
            resp_bulk(r.to_str().unwrap().as_bytes().to_vec())
        },
        LuaValue::Integer(r) => {
            resp_int(r)
        },
        LuaValue::Boolean(r) => {
            let r_int: i64 = if r == false { 0 } else { 1 };
            resp_int(r_int)
        },
        LuaValue::Number(r) => {
            resp_bulk(r.to_string().as_bytes().to_vec())
        },
        LuaValue::Table(r) => {
            let len: i64 = r.raw_get("__self_length__").unwrap();
            //let len: usize = r.len().unwrap().try_into().unwrap();
            let mut arr = Vec::with_capacity(len.try_into().unwrap());
            for idx in 0..len {
                // key is start from 1
                // if key not exists in lua table, it means the value is nil
                let key = idx+1;//.to_string();
                let v: LuaValue = r.raw_get(key).unwrap();
                arr.push(lua_resp_to_redis_resp(v));
            }
            resp_array(arr)
        },
        LuaValue::Nil => {
            resp_nil()
        }
        LuaValue::Error(r) => { resp_err(&r.to_string())},
        _ => {resp_err("panic")},
    }
}

pub fn redis_resp_to_lua_resp(resp: Frame, lua: &Lua) -> LuaValue {
    match resp {
        Frame::Simple(v) => {
            LuaValue::String(lua.create_string(&v).unwrap())
        },
        Frame::Bulk(v) => {
            let str = String::from_utf8_lossy(&v).to_string();
            LuaValue::String(lua.create_string(&str).unwrap())
        },
        Frame::Error(e) => {
            LuaValue::String(lua.create_string(&e).unwrap())
        },
        Frame::Integer(i) => {
            LuaValue::Integer(i)
        },
        Frame::Null => {
            LuaValue::Nil
        },
        Frame::Array(arr) => {
            let len = arr.len();
            let table = lua.create_table().unwrap();
            if len == 0 {
                table.raw_set("__self_length__", LuaValue::Integer(len as i64)).unwrap();
            }
            for idx in 0..len {
                let v = redis_resp_to_lua_resp(arr[idx].clone(), lua);
                // if v is nil, table set will remove the item in table, but we should save the the nil item in table
                if v != LuaValue::Nil {
                    let key = idx+1;//.to_string();
                    table.raw_set(key, v).unwrap();
                }
                // save the length of the redis array in table, but this will have side effect 
                table.raw_set("__self_length__", LuaValue::Integer(len as i64)).unwrap();
            }
            
            LuaValue::Table(table)
        },
        _ => {LuaValue::Nil}
    }
}

pub fn key_is_expired(ttl: u64) -> bool {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    let ts = d.as_secs()*1000 + d.subsec_millis() as u64;
    if ttl > 0 && ttl < ts {
        true
    } else {
        false
    }
}

pub fn now_timestamp_in_millis() -> u64 {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    return d.as_secs()*1000 + d.subsec_millis() as u64;
}

pub fn timestamp_from_ttl(ttl: u64) -> u64 {
    ttl + now_timestamp_in_millis()
}

pub fn ttl_from_timestamp(timestamp: u64) -> u64 {
    let now = now_timestamp_in_millis();
    if now > timestamp {
        return 0
    } else {
        return timestamp - now;
    }
}