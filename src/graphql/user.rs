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
        order::{
            Order,
            query_orders,
        },
    },
    error::Error,
};

pub struct QueryUser;

#[juniper::graphql_object(Context = Context)]
impl QueryUser {
    fn me(context: &Context) -> Result<CurrentUser, Error> {
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
        Ok(CurrentUser::new(id, username, nickname))
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

struct CurrentUser {
    id: Uuid,
    username: String,
    nickname: Option<String>,
}

impl CurrentUser {
    fn new(id: Uuid, username: String, nickname: Option<String>) -> Self {
        CurrentUser {
            id: id,
            username: username,
            nickname: nickname,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl CurrentUser {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn username(&self) -> &String {
        &self.username
    }

    fn nickname(&self) -> Option<&String> {
        self.nickname.as_ref()
    }

    fn shops(&self, context: &Context, id: Option<Uuid>, name: Option<String>) -> Result<Vec<UserShop>, Error> {
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
        Ok(
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

#[juniper::graphql_object]
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

    fn id(&self) -> Uuid {
        self.shop.id()
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

    fn members(&self, context: &Context) -> Result<Option<Vec<Member>>, Error> {
        if let Permission::None = self.member_authority {
            return Err(Error::unauthorized())
        }

        let mut conn = context.state().db_connection()?;
        let user_session_id = context.user_session_id()?;

        let rows = query!(
            conn,
            "SELECT
                users.id,
                users.username,
                users.nickname,
                shop_user.member_authority,
                shop_user.order_authority,
                shop_user.product_authority
            FROM
                shop_user
            INNER JOIN
                users
            ON
                shop_user.user_id = users.id
            WHERE
                shop_user.shop_id = $1
            ",
            &[&self.id()],
        )?;
        Ok(Some(
            rows.iter().map(|row| {
                // Only person who have "All" member_authority can query authority of members.
                let authority = if let Permission::All = self.member_authority {
                    Some(Authority::new(
                        row.get("member_authority"),
                        row.get("order_authority"),
                        row.get("product_authority"),
                    ))
                } else {
                    None
                };

                Member::new(
                    row.get("id"),
                    row.get("username"),
                    row.get("nickname"),
                    authority,
                )
            })
            .collect()
        ))
    }

    fn orders(&self, context: &Context) -> Result<Option<Vec<Order>>, Error> {
        if let Permission::None = self.order_authority {
            return Err(Error::unauthorized())
        }

        let mut conn = context.state().db_connection()?;

        query_orders(conn, Some(self.id()), None)
    }
}

struct Member {
    id: Uuid,
    username: String,
    nickname: Option<String>,
    authority: Option<Authority>,
}

impl Member {
    fn new(id: Uuid, username: String, nickname: Option<String>, authority: Option<Authority>) -> Self {
        Member {
            id: id,
            username: username,
            nickname: nickname,
            authority: authority,
        }
    }
}

#[juniper::graphql_object]
impl Member {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn username(&self) -> &String {
        &self.username
    }

    fn nickname(&self) -> Option<&String> {
        self.nickname.as_ref()
    }

    fn authority(&self) -> Result<Option<&Authority>, Error> {
        if self.authority.is_none() {
            return Err(Error::unauthorized())
        }
        Ok(self.authority.as_ref())
    }
}

struct Authority {
    member: Permission,
    order: Permission,
    product: Permission,
}

impl Authority {
    fn new(member: Permission, order: Permission, product: Permission) -> Self {
        Authority {
            member: member,
            order: order,
            product: product,
        }
    }
}

#[juniper::graphql_object]
impl Authority {
    fn member(&self) -> &Permission {
        &self.member
    }

    fn order(&self) -> &Permission {
        &self.order
    }

    fn product(&self) -> &Permission {
        &self.product
    }
}