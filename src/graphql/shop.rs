use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::{json, Map};
use crate::{
    sql::{
        UuidNN,
        clause::Clause,
    },
    graphql::context::Context,
    error::Error,
    utils::dict::Dict,
};

pub struct QueryShop;

#[juniper::graphql_object(Context = Context)]
impl QueryShop {
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

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[juniper::graphql_object(Context = Context)]
impl Shop {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn latest_update(&self) -> DateTime<Utc> {
        self.latest_update
    }

    fn products(&self, context: &Context, key: Option<Uuid>, name: Option<String>) -> Result<Vec<Product>, Error> {
        let mut conn = context.state().db_connection()?;

        let mut clause = Clause::new();
        if let Some(key) = key.as_ref() {
            clause.and(Clause::equal("key", format!("'{}'", key)));
        }
        if let Some(name) = name.as_ref() {
            clause.and(Clause::like("upper((product).name)", format!("upper('%{}%')", name)));
        }

        let rows = query!(
            conn,
            format!(
                "WITH
                    products AS (
                        SELECT
                            key,
                            product
                        FROM
                            query_shop_products($1){}
                    ),
                    customizes AS (
                        SELECT
                            key prod_key,
                            (query_product_customizes(product)).*
                        FROM
                            products
                    ),
                    selections AS (
                        SELECT
                            key cus_key,
                            (query_customize_selections(customize)).*
                        FROM
                            customizes
                    )
                SELECT
                    products.key prod_key,
                    (product).name prod_name,
                    (product).description prod_description,
                    (product).price prod_price,
                    (product).series_id prod_series_id,
                    (product).has_picture prod_has_picture,
                    (product).latest_update prod_latest_update,
                    cus_join_sel.cus_key,
                    (cus_join_sel.customize).name cus_name,
                    (cus_join_sel.customize).description cus_description,
                    (cus_join_sel.customize).latest_update cus_latest_update,
                    cus_join_sel.sel_key,
                    (cus_join_sel.selection).name sel_name,
                    (cus_join_sel.selection).price sel_price
                FROM
                    products
                LEFT JOIN
                    (
                        SELECT
                            prod_key,
                            customizes.key cus_key,
                            customize,
                            selections.key sel_key,
                            selection
                        FROM
                            customizes
                        LEFT JOIN
                            selections
                        ON
                            customizes.key = selections.cus_key
                    ) cus_join_sel
                ON
                    products.key = cus_join_sel.prod_key",
                clause,
            ).as_str(),
            &[&UuidNN(self.id)],
        )?;

        let mut products = Dict::new();
        for row in rows.iter() {
            let prod_key = row.get::<&str, Uuid>("prod_key");
            let product = if let Some(product) = products.ref_mut_value(prod_key) {
                product
            } else {
                let product = Product::new(
                    prod_key,
                    row.get("prod_name"),
                    row.get("prod_description"),
                    row.get("prod_price"),
                    row.get("prod_has_picture"),
                    row.get("prod_latest_update"),
                );
                products.insert_uncheck(prod_key, product)
            };

            if let Ok(cus_key) = row.try_get::<&str, Uuid>("cus_key") {
                let customize = if let Some(customize) = product.ref_mut_customize(cus_key) {
                    customize
                } else {
                    let customize = Customize::new(
                        cus_key,
                        row.get("cus_name"),
                        row.get("cus_description"),
                        row.get("cus_latest_update"),
                    );
                    product.insert_customize_uncheck(cus_key, customize)
                };

                if let Ok(sel_key) = row.try_get::<&str, Uuid>("sel_key") {
                    if let None = customize.ref_mut_selection(sel_key) {
                        let selection = Selection::new(
                            sel_key,
                            row.get("sel_name"),
                            row.get("sel_price"),
                        );
                        customize.insert_selection_uncheck(sel_key, selection);
                    }
                }
            }
        }

        Ok(products.values())
    }
    
    fn products_json(&self, context: &Context, key: Option<Uuid>, name: Option<String>) -> Result<String, Error> {
        let mut conn = context.state().db_connection()?;

        let mut clause = Clause::new();
        if let Some(key) = key.as_ref() {
            clause.and(Clause::equal("key", format!("'{}'", key)));
        }
        if let Some(name) = name.as_ref() {
            clause.and(Clause::like("upper((product).name)", format!("upper('%{}%')", name)));
        }

        let rows = query!(
            conn,
            format!(
                "WITH
                    products AS (
                        SELECT
                            key,
                            product
                        FROM
                            query_shop_products($1){}
                    ),
                    customizes AS (
                        SELECT
                            key prod_key,
                            (query_product_customizes(product)).*
                        FROM
                            products
                    ),
                    selections AS (
                        SELECT
                            key cus_key,
                            (query_customize_selections(customize)).*
                        FROM
                            customizes
                    )
                SELECT
                    products.key prod_key,
                    (product).name prod_name,
                    (product).description prod_description,
                    (product).price prod_price,
                    (product).series_id prod_series_id,
                    (product).has_picture prod_has_picture,
                    (product).latest_update prod_latest_update,
                    cus_join_sel.cus_key,
                    (cus_join_sel.customize).name cus_name,
                    (cus_join_sel.customize).description cus_description,
                    (cus_join_sel.customize).latest_update cus_latest_update,
                    cus_join_sel.sel_key,
                    (cus_join_sel.selection).name sel_name,
                    (cus_join_sel.selection).price sel_price
                FROM
                    products
                LEFT JOIN
                    (
                        SELECT
                            prod_key,
                            customizes.key cus_key,
                            customize,
                            selections.key sel_key,
                            selection
                        FROM
                            customizes
                        LEFT JOIN
                            selections
                        ON
                            customizes.key = selections.cus_key
                    ) cus_join_sel
                ON
                    products.key = cus_join_sel.prod_key",
                clause,
            ).as_str(),
            &[&UuidNN(self.id)],
        )?;

        let mut products = Map::new();
        for row in rows.iter() {
            let prod_key = row.get::<&str, Uuid>("prod_key");
            if !products.contains_key(&prod_key.to_string()) {
                products.insert(
                    prod_key.to_string(),
                    json!({
                        "name": row.get::<&str, String>("prod_name"),
                        "description": row.get::<&str, Option<String>>("prod_description"),
                        "price": row.get::<&str, i32>("prod_price"),
                        "series_id": row.get::<&str, Option<Uuid>>("prod_series_id"),
                        "has_picture": row.get::<&str, bool>("prod_has_picture"),
                        "latest_update": row.get::<&str, DateTime<Utc>>("prod_latest_update").to_string(),
                        "customizes": Map::new()
                    })
                );
            }

            if let Ok(cus_key) = row.try_get::<&str, Uuid>("cus_key") {
                let customizes = products.get_mut(&prod_key.to_string()).unwrap().get_mut("customizes").unwrap().as_object_mut().unwrap();
                if !customizes.contains_key(&cus_key.to_string()) {
                    customizes.insert(
                        cus_key.to_string(),
                        json!({
                            "name": row.get::<&str, String>("cus_name"),
                            "description": row.get::<&str, Option<String>>("cus_description"),
                            "latest_update": row.get::<&str, DateTime<Utc>>("cus_latest_update").to_string(),
                            "selections": Map::new()
                        })
                    );
                }

                if let Ok(sel_key) = row.try_get::<&str, Uuid>("sel_key") {
                    let selections = customizes.get_mut(&cus_key.to_string()).unwrap().get_mut("selections").unwrap().as_object_mut().unwrap();
                    selections.insert(
                        sel_key.to_string(),
                        json!({
                            "name": row.get::<&str, String>("sel_name"),
                            "price": row.get::<&str, i32>("sel_price")
                        })
                    );
                }
            }
        }

        Ok(serde_json::to_string(&products)?)
    }
}

struct Product {
    key: Uuid,
    name: String,
    description: Option<String>,
    price: i32,
    has_picture: bool,
    latest_update: DateTime<Utc>,
    customizes: Dict<Uuid, Customize>,
}

impl Product {
    fn new(key: Uuid, name: String, description: Option<String>, price: i32, has_picture: bool, latest_update: DateTime<Utc>) -> Self {
        Product {
            key: key,
            name: name,
            description: description,
            price: price,
            has_picture: has_picture,
            latest_update: latest_update,
            customizes: Dict::new(),
        }
    }
    
    fn ref_mut_customize(&mut self, key: Uuid) -> Option<&mut Customize> {
        self.customizes.ref_mut_value(key)
    }

    fn insert_customize_uncheck(&mut self, key: Uuid, cus: Customize) -> &mut Customize {
        self.customizes.insert_uncheck(key, cus)
    }
}

#[juniper::graphql_object(Context = Context)]
impl Product {
    fn key(&self) -> Uuid {
        self.key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn description(&self) -> &Option<String> {
        &self.description
    }

    fn price(&self) -> i32 {
        self.price
    }

    fn has_picture(&self) -> bool {
        self.has_picture
    }

    fn latest_update(&self) -> DateTime<Utc> {
        self.latest_update
    }

    fn customizes(&self) -> &Vec<Customize> {
        self.customizes.ref_values()
    }
}

struct Customize {
    key: Uuid,
    name: String,
    description: Option<String>,
    latest_update: DateTime<Utc>,
    selections: Dict<Uuid, Selection>,
}

impl Customize {
    fn new(key: Uuid, name: String, description: Option<String>, latest_update: DateTime<Utc>) -> Self {
        Customize {
            key: key,
            name: name,
            description: description,
            latest_update: latest_update,
            selections: Dict::new(),
        }
    }

    fn ref_mut_selection(&mut self, key: Uuid) -> Option<&mut Selection> {
        self.selections.ref_mut_value(key)
    }

    fn insert_selection_uncheck(&mut self, key: Uuid, sel: Selection) -> &mut Selection {
        self.selections.insert_uncheck(key, sel)
    }
}

#[juniper::graphql_object(Context = Context)]
impl Customize {
    fn key(&self) -> Uuid {
        self.key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn description(&self) -> &Option<String> {
        &self.description
    }

    fn latest_update(&self) -> DateTime<Utc> {
        self.latest_update
    }

    fn selections(&self) -> &Vec<Selection> {
        self.selections.ref_values()
    }
}

struct Selection {
    key: Uuid,
    name: String,
    price: i32,
}

impl Selection {
    fn new(key: Uuid, name: String, price: i32) -> Self {
        Selection {
            key: key,
            name: name,
            price: price,
        }
    }
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