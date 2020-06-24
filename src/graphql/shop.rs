use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::{json, Map};
use crate::{
    sql::{
        UuidNN,
        Permission,
        clause::Clause,
    },
    graphql::{
        Context,
        user::User,
    },
    error::Error,
};

pub struct QueryShop;

#[juniper::graphql_object(Context = Context)]
impl QueryShop {
    fn my(context: &Context, id: Option<Uuid>, name: Option<String>) -> Result<Vec<Shop>, Error> {
        let user_session_id = context.user_session_id()?;
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
                "WITH
                    my_user_shops AS (
                        SELECT
                            shop_id
                        FROM
                            shop_user
                        WHERE
                            user_id = get_session_user($1)
                    ),
                    my_shops AS (
                        SELECT
                            id, name, latest_update
                        FROM
                            shops INNER JOIN my_user_shops ON shops.id = my_user_shops.shop_id
                    )
                SELECT
                    id, name, latest_update
                FROM
                    my_shops
                {}",
                clause,
            ).as_str(),
            &[&UuidNN(user_session_id)],
        )?;

        Ok(
            rows.iter().map(|row| {
                Shop::new(
                    row.get("id"),
                    row.get("name"),
                    row.get("latest_update"),
                )
            })
            .collect()
        )
    }

    fn search(context: &Context, id: Option<Uuid>, name: Option<String>) -> Result<Vec<Shop>, Error> {
        let mut clauses = String::new();

        if let Some(id) = id.as_ref() {
            if clauses == "" {
                clauses.push_str(" WHERE");
            } else {
                clauses.push_str(" AND");
            }
            clauses.push_str(format!(" id = '{}'", id).as_str());
        }

        if let Some(name) = name.as_ref() {
            if clauses == "" {
                clauses.push_str(" WHERE");
            } else {
                clauses.push_str(" AND");
            }
            clauses.push_str(format!(" name_upper LIKE upper('%{}%')", name).as_str());
        }

        let mut conn = context.state().db_connection()?;
        let rows = query!(
            conn,
            format!("SELECT id, name, latest_update FROM shops{}", clauses).as_str(),
            &[],
        )?;
        Ok(
            rows.iter().map(|row| {
                Shop::new(
                    row.get("id"),
                    row.get("name"),
                    row.get("latest_update"),
                )
            })
            .collect()
        )
    }
}

pub struct Shop {
    id: Uuid,
    name: String,
    latest_update: DateTime<Utc>,
}

impl Shop {
    pub fn new(id: Uuid, name: String, latest_update: DateTime<Utc>) -> Self {
        Shop {
            id: id,
            name: name,
            latest_update: latest_update,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl Shop {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn latest_update(&self) -> &DateTime<Utc> {
        &self.latest_update
    }

    fn products(&self, context: &Context) -> Result<Vec<Product>, Error> {
        let mut conn = context.state().db_connection()?;
        let rows = query!(
            conn,
            "SELECT key, (product).name, (product).description, (product).price, (product).latest_update
                FROM query_shop_products($1);",
            &[&UuidNN(self.id)],
        )?;
        Ok(
            rows.iter().map(|row| {
                Product {
                    shop_id: self.id,
                    key: row.get("key"),
                    name: row.get("name"),
                    description: row.get("description"),
                    price: row.get("price"),
                    latest_update: row.get("latest_update"),
                }
            })
            .collect()
        )
    }

    fn products_json(&self, context: &Context) -> Result<String, Error> {
        let mut conn = context.state().db_connection()?;
        let rows = query!(
            conn,
            "WITH
                query_products AS (
                    SELECT
                        (query_shop_products($1)).*
                ),
                query_customizes AS (
                    SELECT
                        key prod_key,
                        product prod,
                        (query_product_customizes(product)).*
                    FROM
                        query_products
                ),
                query_selections AS (
                    SELECT
                        prod_key,
                        prod,
                        key cus_key,
                        customize cus,
                        (query_customize_selections(customize)).*
                    FROM
                        query_customizes
                )
            SELECT
                prod_key,
                (prod).name prod_name,
                (prod).description prod_desc,
                (prod).price prod_price,
                (prod).series_id prod_series_id,
                (prod).latest_update prod_latest_update,
                cus_key,
                (cus).name cus_name,
                (cus).description cus_desc,
                (cus).latest_update cus_latest_update,
                key sel_key,
                (selection).name sel_name,
                (selection).price sel_price
            FROM
                query_selections",
            &[&UuidNN(self.id)],
        )?;

        let mut products = Map::new();
        for row in rows.iter() {
            let prod_key: Uuid = row.get("prod_key");
            if !products.contains_key(&prod_key.to_string()) {
                let name: String = row.get("prod_name");
                let desc: Option<String> = row.get("prod_desc");
                let price: i32 = row.get("prod_price");
                let series_id: Option<Uuid> = row.get("prod_series_id");
                let latest_update: DateTime<Utc> = row.get("prod_latest_update");
                let customizes = Map::new();
                let product = json!({
                    "name": name,
                    "description": desc,
                    "price": price,
                    "series_id": series_id,
                    "latest_update": latest_update.to_string(),
                    "customizes": customizes
                });
                products.insert(prod_key.to_string(), product);
            }

            let customizes = products.get_mut(&prod_key.to_string()).unwrap().get_mut("customizes").unwrap().as_object_mut().unwrap();
            let cus_key: Uuid = row.get("cus_key");
            if !customizes.contains_key(&cus_key.to_string()) {
                let name: String = row.get("cus_name");
                let desc: String = row.get("cus_desc");
                let latest_update: DateTime<Utc> = row.get("cus_latest_update");
                let selections = Map::new();
                let customize = json!({
                    "name": name,
                    "description": desc,
                    "latest_update": latest_update.to_string(),
                    "selections": selections
                });
                customizes.insert(cus_key.to_string(), customize);
            }

            let selections = customizes.get_mut(&cus_key.to_string()).unwrap().get_mut("selections").unwrap().as_object_mut().unwrap();
            let sel_key: Uuid = row.get("sel_key");
            let sel_key = sel_key.to_string();
            let name: String = row.get("sel_name");
            let price: i32 = row.get("sel_price");
            let selection = json!({
                "name": name,
                "price": price
            });
            selections.insert(sel_key, selection);
        }

        Ok(serde_json::to_string(&products)?)
    }

    fn shop_users(&self, context: &Context) -> Result<Option<Vec<ShopUser>>, Error> {
        let mut conn = context.state().db_connection()?;
        let user_session_id = context.user_session_id()?;
        if let Some(row) = query_opt!(
            conn,
            "SELECT team_authority FROM shop_user
                WHERE shop_id = $1 AND user_id = (SELECT user_id FROM get_session_user($2))",
            &[&self.id, &UuidNN(user_session_id)],
        )? {
            if let Permission::None = row.get("team_authority") {
                return Err(Error::permission_denied())
            }
        } else {
            return Err(Error::permission_denied())
        }

        let rows = query!(
            conn,
            "SELECT users.id, users.username, users.nickname, shop_user.team_authority, shop_user.store_authority, shop_user.product_authority
                FROM shop_user INNER JOIN users
                ON shop_user.user_id = users.id
                AND shop_user.shop_id = $1",
            &[&self.id],
        )?;
        Ok(Some(
            rows.iter().map(|row| {
                ShopUser::new(
                    User::new(
                        row.get("id"),
                        row.get("username"),
                        row.get("nickname"),
                    ),
                    row.get("team_authority"),
                    row.get("store_authority"),
                    row.get("product_authority"),
                )
            })
            .collect()
        ))
    }
}

struct Product {
    shop_id: Uuid,
    key: Uuid,
    name: String,
    description: String,
    price: i32,
    latest_update: DateTime<Utc>,
}

#[juniper::graphql_object(Context = Context)]
impl Product {
    fn key(&self) -> &Uuid {
        &self.key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn description(&self) -> &String {
        &self.description
    }

    fn price(&self) -> &i32 {
        &self.price
    }

    fn latest_update(&self) -> &DateTime<Utc> {
        &self.latest_update
    }

    fn customizes(&self, context: &Context) -> Result<Vec<Customize>, Error> {
        let mut conn = context.state().db_connection()?;
        let rows = query!(
            conn,
            "WITH query AS (
                SELECT query_product_customizes(product) AS cus FROM query_shop_products($1) WHERE key = $2
            )
            SELECT (cus).key, ((cus).customize).name, ((cus).customize).description, ((cus).customize).latest_update FROM query;",
            &[&UuidNN(self.shop_id), &self.key],
        )?;
        Ok(
            rows.iter().map(|row| {
                Customize {
                    shop_id: self.shop_id,
                    product_key: self.key,
                    key: row.get("key"),
                    name: row.get("name"),
                    description: row.get("description"),
                    latest_update: row.get("latest_update"),
                }
            })
            .collect()
        )
    }
}

struct Customize {
    shop_id: Uuid,
    product_key: Uuid,
    key: Uuid,
    name: String,
    description: String,
    latest_update: DateTime<Utc>,
}

#[juniper::graphql_object(Context = Context)]
impl Customize {
    fn key(&self) -> &Uuid {
        &self.key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn description(&self) -> &String {
        &self.description
    }

    fn latest_update(&self) -> &DateTime<Utc> {
        &self.latest_update
    }

    fn selections(&self, context: &Context) -> Result<Vec<Selection>, Error> {
        let mut conn = context.state().db_connection()?;
        let rows = query!(
            conn,
            "WITH customizes AS (
                SELECT query_product_customizes(product) AS cus FROM query_shop_products($1) WHERE key = $2
            ), selections AS (
                SELECT query_customize_selections((cus).customize) AS sel FROM customizes WHERE (cus).key = $3
            )
            SELECT (sel).key, ((sel).selection).name, ((sel).selection).price FROM selections;",
            &[&UuidNN(self.shop_id), &self.product_key, &self.key],
        )?;
        Ok(
            rows.iter().map(|row| {
                Selection {
                    key: row.get("key"),
                    name: row.get("name"),
                    price: row.get("price"),
                }
            })
            .collect()
        )
    }
}

struct Selection {
    key: Uuid,
    name: String,
    price: i32,
}

#[juniper::graphql_object]
impl Selection {
    fn key(&self) -> &Uuid {
        &self.key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn price(&self) -> &i32 {
        &self.price
    }
}

struct ShopUser {
    user: User,
    team_authority: Permission,
    store_authority: Permission,
    product_authority: Permission,
}

impl ShopUser {
    fn new(user: User, team_authority: Permission, store_authority: Permission, product_authority: Permission) -> Self {
        ShopUser {
            user: user,
            team_authority: team_authority,
            store_authority: store_authority,
            product_authority: product_authority,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl ShopUser {
    fn user(&self) -> &User {
        &self.user
    }

    fn team_authority(&self) -> &Permission {
        &self.team_authority
    }

    fn store_authority(&self) -> &Permission {
        &self.store_authority
    }

    fn product_authority(&self) -> &Permission {
        &self.product_authority
    }
}