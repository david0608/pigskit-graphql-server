use uuid::Uuid;
use crate::{
    sql::{
        UuidNN,
        Permission,
        clause::Clause,
    },
    graphql::{
        context::Context,
        shop::Shop,
    },
    error::Error,
};

pub struct QueryUser;

#[juniper::graphql_object(Context = Context)]
impl QueryUser {
    fn me(context: &Context) -> Result<User, Error> {
        let user_session_id = context.user_session_id()?;
        let mut conn = context.state().db_connection()?;
        let (id, username, nickname,) = query_one!(
            conn,
            "WITH ss_user AS (
                SELECT user_id id FROM get_session_user($1)
            )
            SELECT id, username, nickname FROM users WHERE id = (SELECT id FROM ss_user);",
            &[&UuidNN(user_session_id)],
            (id: Uuid, username: String, nickname: Option<String>),
        )?;
        Ok(User::new(id, username, nickname))
    }

    fn search(context: &Context, id: Option<Uuid>, name: Option<String>) -> Result<Vec<User>, Error> {
        let mut conn = context.state().db_connection()?;

        let mut clause = Clause::new();
        if let Some(id) = id.as_ref() {
            clause.and(Clause::equal("id", format!("'{}'", id)));
        }
        if let Some(name) = name.as_ref() {
            clause.and(Clause::like("upper(name)", format!("upper('%{}%')", name)));
        }

        let rows = query!(
            conn,
            format!("SELECT id, username, nickname FROM users{}", clause).as_str(),
            &[],
        )?;
        Ok(
            rows.iter().map(|row| {
                User::new(
                    row.get("id"),
                    row.get("username"),
                    row.get("nickname"),
                )
            })
            .collect()
        )
    }
}

pub struct User {
    id: Uuid,
    username: String,
    nickname: Option<String>,
}

impl User {
    pub fn new(id: Uuid, username: String, nickname: Option<String>) -> Self {
        User {
            id: id,
            username: username,
            nickname: nickname,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl User {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn username(&self) -> &String {
        &self.username
    }

    fn nickname(&self) -> Option<&String> {
        self.nickname.as_ref()
    }

    fn user_shops(&self, context: &Context, id: Option<Uuid>, name: Option<String>) -> Result<Option<Vec<UserShop>>, Error> {
        let mut conn = context.state().db_connection()?;

        let mut clause = Clause::new();
        if let Some(id) = id.as_ref() {
            clause.and(Clause::equal("id", format!("'{}'", id)));
        }
        if let Some(name) = name.as_ref() {
            clause.and(Clause::like("upper(name)", format!("upper('%{}%')", name)));
        }

        let rows = query!(
            conn,
            format!(
                "SELECT
                    shop.id,
                    shop.name,
                    shop.latest_update,
                    shop_user.member_authority,
                    shop_user.order_authority,
                    shop_user.product_authority
                FROM
                    (SELECT * FROM shops{}) shop
                INNER JOIN
                    shop_user
                ON
                    shop.id = shop_user.shop_id
                    AND shop_user.user_id = $1",
                clause
            ).as_str(),
            &[&self.id],
        )?;
        Ok(Some(
            rows.iter().map(|row| {
                UserShop::new(
                    Shop::new(
                        row.get("id"),
                        row.get("name"),
                        row.get("latest_update"),
                    ),
                    row.get("member_authority"),
                    row.get("order_authority"),
                    row.get("product_authority"),
                )
            })
            .collect()
        ))
    }
}

struct UserShop {
    shop: Shop,
    member_authority: Permission,
    order_authority: Permission,
    product_authority: Permission,
}

impl UserShop {
    fn new(shop: Shop, member_authority: Permission, order_authority: Permission, product_authority: Permission) -> Self {
        UserShop {
            shop: shop,
            member_authority: member_authority,
            order_authority: order_authority,
            product_authority: product_authority,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl UserShop {
    fn shop(&self) -> &Shop {
        &self.shop
    }

    fn member_authority(&self) -> &Permission {
        &self.member_authority
    }

    fn order_authority(&self) -> &Permission {
        &self.order_authority
    }

    fn product_authority(&self) -> &Permission {
        &self.product_authority
    }
}