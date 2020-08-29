use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::{
    sql::{
        clause::Clause,
    },
    graphql::context::Context,
    state::db::Connection,
    error::Error,
    utils::dict::Dict,
};

pub fn query_orders(mut db_connection: Connection, shop_id: Option<Uuid>, guest_session_id: Option<Uuid>) -> Result<Vec<Order>, Error> {
    let mut clause = Clause::new();
    if let Some(shop_id) = shop_id.as_ref() {
        clause.and(Clause::equal("shop_id", format!("'{}'", shop_id)));
    }
    if let Some(guest_session_id) = guest_session_id.as_ref() {
        clause.and(Clause::equal("guest_session_id", format!("'{}'", guest_session_id)));
    }

    let rows = query!(
        db_connection,
        format!(
            "WITH
                query_orders AS (
                    SELECT * FROM orders {}
                ),
                query_items AS (
                    SELECT id order_id, (each(items)).* FROM query_orders
                ),
                query_customizes AS (
                    SELECT key item_key, (query_product_item_customize_items(value::PRODUCT_ITEM)).* FROM query_items
                )
            SELECT
                id order_id,
                guest_session_id,
                shop_id,
                order_number,
                order_at,
                item_key,
                (item).product_key,
                (item).name,
                (item).price,
                (item).count,
                (item).remark,
                (item).order_at item_order_at,
                customize_key,
                (customize).name customize_name,
                (customize).selection,
                (customize).selection_key,
                (customize).price selection_price,
                (customize).order_at customize_order_at
            FROM
                query_orders
            LEFT JOIN
                (
                    SELECT
                        order_id,
                        query_items.key::UUID item_key,
                        query_items.value::PRODUCT_ITEM item,
                        query_customizes.key::UUID customize_key,
                        customize
                    FROM
                        query_items
                    LEFT JOIN
                        query_customizes
                    ON
                        query_items.key = query_customizes.item_key
                ) item_join_cus
            ON
                query_orders.id = item_join_cus.order_id
            ",
            clause,
        ).as_str(),
        &[],
    )?;

    let mut orders = Dict::new();
    for row in rows.iter() {
        let order_id = row.get::<&str, Uuid>("order_id");
        let order = if let Some(order) = orders.ref_mut_value(order_id) {
            order
        } else {
            let order = Order::new(
                order_id,
                row.get("guest_session_id"),
                row.get("shop_id"),
                row.get("order_number"),
                row.get("order_at"),
            );
            orders.insert_uncheck(order_id, order)
        };

        if let Ok(item_key) = row.try_get::<&str, Uuid>("item_key") {
            let item = if let Some(item) = order.ref_mut_item(item_key) {
                item
            } else {
                let item = ProductItem::new(
                    item_key,
                    row.get("product_key"),
                    row.get("name"),
                    row.get("price"),
                    row.get("count"),
                    row.get("remark"),
                    row.get("item_order_at"),
                );
                order.insert_item_uncheck(item_key, item)
            };

            if let Ok(customize_key) = row.try_get::<&str, Uuid>("customize_key") {
                if let None = item.ref_mut_customize(customize_key) {
                    let customize = CustomizeItem::new(
                        customize_key,
                        row.get("customize_name"),
                        row.get("selection"),
                        row.get("selection_key"),
                        row.get("selection_price"),
                        row.get("customize_order_at"),
                    );
                    item.insert_customize_uncheck(customize_key, customize);
                }
            }
        }
    }

    Ok(orders.values())
}

pub fn query_carts(mut db_connection: Connection, shop_id: Option<Uuid>, guest_session_id: Option<Uuid>) -> Result<Vec<Cart>, Error> {
    let mut clause = Clause::new();
    if let Some(shop_id) = shop_id.as_ref() {
        clause.and(Clause::equal("shop_id", format!("'{}'", shop_id)));
    }
    if let Some(guest_session_id) = guest_session_id.as_ref() {
        clause.and(Clause::equal("guest_session_id", format!("'{}'", guest_session_id)));
    }

    let rows = query!(
        db_connection,
        format!(
            "WITH
                query_carts AS (
                    SELECT * FROM cart {}
                ),
                query_items AS (
                    SELECT id cart_id, (each(items)).* FROM query_carts
                ),
                query_customizes AS (
                    SELECT key item_key, (query_product_item_customize_items(value::PRODUCT_ITEM)).* FROM query_items
                )
            SELECT
                id cart_id,
                guest_session_id,
                shop_id,
                item_key,
                (item).product_key,
                (item).name,
                (item).price,
                (item).count,
                (item).remark,
                (item).order_at item_order_at,
                customize_key,
                (customize).name customize_name,
                (customize).selection,
                (customize).selection_key,
                (customize).price selection_price,
                (customize).order_at customize_order_at
            FROM
                query_carts
            LEFT JOIN
                (
                    SELECT
                        cart_id,
                        query_items.key::UUID item_key,
                        query_items.value::PRODUCT_ITEM item,
                        query_customizes.key::UUID customize_key,
                        customize
                    FROM
                        query_items
                    LEFT JOIN
                        query_customizes
                    ON
                        query_items.key = query_customizes.item_key
                ) item_join_cus
            ON
                query_carts.id = item_join_cus.cart_id
            ",
            clause,
        ).as_str(),
        &[],
    )?;

    let mut carts = Dict::new();
    for row in rows.iter() {
        let cart_id = row.get::<&str, Uuid>("cart_id");
        let cart = if let Some(cart) = carts.ref_mut_value(cart_id) {
            cart
        } else {
            let cart = Cart::new(
                cart_id,
                row.get("shop_id"),
                row.get("guest_session_id"),
            );
            carts.insert_uncheck(cart_id, cart)
        };

        if let Ok(item_key) = row.try_get::<&str, Uuid>("item_key") {
            let item = if let Some(item) = cart.ref_mut_item(item_key) {
                item
            } else {
                let item = ProductItem::new(
                    item_key,
                    row.get("product_key"),
                    row.get("name"),
                    row.get("price"),
                    row.get("count"),
                    row.get("remark"),
                    row.get("item_order_at"),
                );
                cart.insert_item_uncheck(item_key, item)
            };

            if let Ok(customize_key) = row.try_get::<&str, Uuid>("customize_key") {
                if let None = item.ref_mut_customize(customize_key) {
                    let customize = CustomizeItem::new(
                        customize_key,
                        row.get("customize_name"),
                        row.get("selection"),
                        row.get("selection_key"),
                        row.get("selection_price"),
                        row.get("customize_order_at"),
                    );
                    item.insert_customize_uncheck(customize_key, customize);
                }
            }
        }
    }

    Ok(carts.values())
}

pub struct Cart {
    id: Uuid,
    shop_id: Uuid,
    guest_session_id: Uuid,
    items: Dict<Uuid, ProductItem>,
}

impl Cart {
    fn new(
        id: Uuid,
        shop_id: Uuid,
        guest_session_id: Uuid,
    ) -> Self {
        Cart {
            id: id,
            shop_id: shop_id,
            guest_session_id: guest_session_id,
            items: Dict::new(),
        }
    }

    fn ref_mut_item(&mut self, key: Uuid) -> Option<&mut ProductItem> {
        self.items.ref_mut_value(key)
    }

    fn insert_item_uncheck(&mut self, key: Uuid, item: ProductItem) -> &mut ProductItem {
        self.items.insert_uncheck(key, item)
    }
}

#[juniper::graphql_object(Context = Context)]
impl Cart {
    fn id(&self) -> Uuid {
        self.id
    }

    fn shop_id(&self) -> Uuid {
        self.shop_id
    }

    fn guest_session_id(&self) -> Uuid {
        self.guest_session_id
    }

    fn items(&self) -> &Vec<ProductItem> {
        self.items.ref_values()
    }
}

pub struct Order {
    id: Uuid,
    guest_session_id: Uuid,
    shop_id: Uuid,
    order_number: i32,
    order_at: DateTime<Utc>,
    items: Dict<Uuid, ProductItem>,
}

impl Order {
    fn new(
        id: Uuid,
        guest_session_id: Uuid,
        shop_id: Uuid,
        order_number: i32,
        order_at: DateTime<Utc>,
    ) -> Self {
        Order {
            id: id,
            guest_session_id: guest_session_id,
            shop_id: shop_id,
            order_number: order_number,
            order_at: order_at,
            items: Dict::new(),
        }
    }

    fn ref_mut_item(&mut self, key: Uuid) -> Option<&mut ProductItem> {
        self.items.ref_mut_value(key)
    }

    fn insert_item_uncheck(&mut self, key: Uuid, item: ProductItem) -> &mut ProductItem {
        self.items.insert_uncheck(key, item)
    }
}

#[juniper::graphql_object(Context = Context)]
impl Order {
    fn id(&self) -> Uuid {
        self.id
    }

    fn guest_session_id(&self) -> Uuid {
        self.guest_session_id
    }

    fn shop_id(&self) -> Uuid {
        self.shop_id
    }

    fn order_number(&self) -> i32 {
        self.order_number
    }

    fn items(&self) -> &Vec<ProductItem> {
        self.items.ref_values()
    }

    fn order_at(&self) -> DateTime<Utc> {
        self.order_at
    }
}

struct ProductItem {
    key: Uuid,
    product_key: Uuid,
    name: String,
    price: i32,
    count: i32,
    remark: Option<String>,
    order_at: DateTime<Utc>,
    customizes: Dict<Uuid, CustomizeItem>,
}

impl ProductItem {
    fn new(
        key: Uuid,
        product_key: Uuid,
        name: String,
        price: i32,
        count: i32,
        remark: Option<String>,
        order_at: DateTime<Utc>,
    ) -> Self {
        ProductItem {
            key: key,
            product_key: product_key,
            name: name,
            price: price,
            count: count,
            remark: remark,
            order_at: order_at,
            customizes: Dict::new(),
        }
    }

    fn ref_mut_customize(&mut self, key: Uuid) -> Option<&mut CustomizeItem> {
        self.customizes.ref_mut_value(key)
    }

    fn insert_customize_uncheck(&mut self, key: Uuid, cus: CustomizeItem) -> &mut CustomizeItem {
        self.customizes.insert_uncheck(key, cus)
    }
}

#[juniper::graphql_object(Context = Context)]
impl ProductItem {
    fn key(&self) -> Uuid {
        self.key
    }

    fn product_key(&self) -> Uuid {
        self.product_key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn price(&self) -> i32 {
        self.price
    }

    fn count(&self) -> i32 {
        self.count
    }

    fn remark(&self) -> &Option<String> {
        &self.remark
    }

    fn order_at(&self) -> DateTime<Utc> {
        self.order_at
    }

    fn customizes(&self) -> &Vec<CustomizeItem> {
        self.customizes.ref_values()
    }
}

struct CustomizeItem {
    customize_key: Uuid,
    name: String,
    selection: Option<String>,
    selection_key: Option<Uuid>,
    selection_price: Option<i32>,
    order_at: DateTime<Utc>,
}

impl CustomizeItem {
    fn new(
        customize_key: Uuid,
        name: String,
        selection: Option<String>,
        selection_key: Option<Uuid>,
        selection_price: Option<i32>,
        order_at: DateTime<Utc>,
    ) -> Self {
        CustomizeItem {
            customize_key: customize_key,
            name: name,
            selection: selection,
            selection_key: selection_key,
            selection_price: selection_price,
            order_at: order_at,
        }
    }
}

#[juniper::graphql_object(Context = Context)]
impl CustomizeItem {
    fn customize_key(&self) -> Uuid {
        self.customize_key
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn selection(&self) -> &Option<String> {
        &self.selection
    }

    fn selection_key(&self) -> Option<Uuid> {
        self.selection_key
    }

    fn selection_price(&self) -> Option<i32> {
        self.selection_price
    }

    fn order_at(&self) -> DateTime<Utc> {
        self.order_at
    }
}