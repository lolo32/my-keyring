use std::str::FromStr;

use saphir::prelude::{Json, Request, SaphirError};
use serde::Deserialize;

#[inline]
pub async fn read_body<T>(req: &mut Request) -> Result<T, SaphirError>
where
    T: for<'a> Deserialize<'a> + Unpin + 'static,
{
    let body = req.body_mut().take_as::<Json<T>>().await;
    body.map(|x| Json(x).into_inner())
}

#[inline]
pub fn read_param<T>(req: &mut Request, param: &str) -> Result<T, SaphirError>
where
    T: FromStr,
{
    req.captures_mut()
        .remove(param)
        .map(|p| p.parse::<T>())
        .transpose()
        .map_err(|_| SaphirError::InvalidParameter(param.to_string(), false))?
        .ok_or_else(|| SaphirError::MissingParameter(param.to_string(), false))
}
