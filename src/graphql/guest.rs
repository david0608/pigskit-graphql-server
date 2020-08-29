use uuid::Uuid;
use crate::{
    sql::UuidNN,
    graphql::{
        context::Context,
        order::{
            query_carts,
            query_orders,
            Cart,
            Order,
        }
    },
    error::Error,
};

pub struct QueryGuest;

#[juniper::graphql_object(Context = Context)]
impl QueryGuest {
    fn carts(context: &Context, shop_id: Option<Uuid>) -> Result<Vec<Cart>, Error> {
        let mut conn = context.state().db_connection()?;
        let guest_session_id = context.guest_session_id()?;

        let (ok,) = query_one!(
            conn,
            "SELECT is_guest_session_valid($1) AS ok;",
            &[&UuidNN(guest_session_id)],
            (ok: bool),
        )?;

        if !ok { return Err(Error::session_expired("GSSID")) }

        query_carts(conn, shop_id, Some(guest_session_id))
    }

    fn orders(context: &Context, shop_id: Uuid) -> Result<Vec<Order>, Error> {
        let guest_session_id = context.guest_session_id()?;
        let mut conn = context.state().db_connection()?;

        let (ok,) = query_one!(
            conn,
            "SELECT is_guest_session_valid($1) AS ok;",
            &[&UuidNN(guest_session_id)],
            (ok: bool),
        )?;

        if !ok { return Err(Error::session_expired("GSSID")) }

        query_orders(conn, None, Some(guest_session_id))
    }
}