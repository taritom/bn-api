use chrono::prelude::Utc;
use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::expression::sql_literal::sql;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::BigInt;
use models::*;
use schema::{events, organization_users, organizations, users};
use std::collections::HashMap;
use time::Duration;
use utils::errors::{ConvertToDatabaseError, DatabaseError, ErrorCode};
use utils::passwords::PasswordHash;
use uuid::Uuid;
use validator::Validate;

#[derive(Insertable, PartialEq, Debug, Validate)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hashed_pw: String,
    role: Vec<Roles>,
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, QueryableByName)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
    pub hashed_pw: String,
    pub password_modified_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
    pub active: bool,
    pub role: Vec<Roles>,
    pub password_reset_token: Option<Uuid>,
    pub password_reset_requested_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
    pub last_cart_id: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DisplayUser {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
    pub is_org_owner: bool,
}

#[derive(AsChangeset, Default, Deserialize, Validate, Clone)]
#[table_name = "users"]
pub struct UserEditableAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub active: Option<bool>,
    pub role: Option<Vec<Roles>>,
    #[validate(url(message = "Profile pic URL is invalid"))]
    pub profile_pic_url: Option<String>,
    #[validate(url(message = "Thumb profile pic URL is invalid"))]
    pub thumb_profile_pic_url: Option<String>,
    #[validate(url(message = "Cover photo URL is invalid"))]
    pub cover_photo_url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FanProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub facebook_linked: bool,
    pub event_count: u32,
    pub revenue_in_cents: u32,
    pub ticket_sales: u32,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
    pub created_at: NaiveDateTime,
}

impl NewUser {
    pub fn commit(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        self.validate()?;
        let user: User = diesel::insert_into(users::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new user")?;

        Wallet::create_for_user(user.id, "Default".to_string(), true, conn)?;

        Ok(user)
    }
}

impl User {
    pub fn create(
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        password: &str,
    ) -> NewUser {
        let hash = PasswordHash::generate(password, None);
        let lower_email = email.clone().map(|e| e.to_lowercase());
        NewUser {
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            email: lower_email,
            phone: phone.clone(),
            hashed_pw: hash.to_string(),
            role: vec![Roles::User],
        }
    }

    pub fn create_from_external_login(
        external_user_id: String,
        first_name: String,
        last_name: String,
        email: String,
        site: String,
        access_token: String,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        let hash = PasswordHash::generate("random", None);
        let lower_email = email.to_lowercase();
        let new_user = NewUser {
            first_name: Some(first_name),
            last_name: Some(last_name),
            email: Some(lower_email.to_string()),
            phone: None,
            hashed_pw: hash.to_string(),
            role: vec![Roles::User],
        };
        new_user.commit(conn).and_then(|user| {
            user.add_external_login(external_user_id, site, access_token, conn)?;
            Ok(user)
        })
    }

    pub fn create_stub(
        first_name: String,
        last_name: String,
        email: Option<String>,
        phone: Option<String>,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        let hash = PasswordHash::generate("random", None);
        let new_user = NewUser {
            first_name: Some(first_name),
            last_name: Some(last_name),
            email,
            phone,
            hashed_pw: hash.to_string(),
            role: vec![Roles::User],
        };
        new_user.commit(conn)
    }

    pub fn get_history_for_organization(
        &self,
        organization: &Organization,
        page: u32,
        limit: u32,
        sort_direction: SortingDir,
        conn: &PgConnection,
    ) -> Result<Payload<HistoryItem>, DatabaseError> {
        use schema::*;
        let query = order_items::table
            .inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .filter(orders::status.eq(OrderStatus::Paid))
            .filter(orders::user_id.eq(self.id))
            .filter(events::organization_id.eq(organization.id))
            .group_by((orders::id, orders::order_date, events::name))
            .select((
                orders::id,
                orders::order_date,
                events::name,
                sql::<BigInt>(
                    "cast(COALESCE(sum(
                    CASE WHEN order_items.item_type = 'Tickets'
                    THEN order_items.quantity
                    ELSE 0 END
                    ), 0) as BigInt)",
                ),
                sql::<BigInt>(
                    "cast(sum(order_items.unit_price_in_cents * order_items.quantity) as bigint)",
                ),
                sql::<BigInt>("count(*) over()"),
            ))
            .order_by(sql::<()>(&format!("orders.order_date {}", sort_direction)))
            .limit(limit as i64)
            .offset((limit * page) as i64);

        #[derive(Queryable)]
        struct R {
            order_id: Uuid,
            order_date: NaiveDateTime,
            event_name: String,
            ticket_sales: i64,
            revenue_in_cents: i64,
            total_rows: i64,
        }
        let results: Vec<R> = query.get_results(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not load history for organization fan",
        )?;

        let paging = Paging::new(page, limit);
        let mut total: u64 = 0;
        if !results.is_empty() {
            total = results[0].total_rows as u64;
        }

        let history = results
            .into_iter()
            .map(|r| HistoryItem::Purchase {
                order_id: r.order_id,
                order_date: r.order_date,
                event_name: r.event_name,
                ticket_sales: r.ticket_sales as u32,
                revenue_in_cents: r.revenue_in_cents as u32,
            })
            .collect();

        let mut payload = Payload::new(history, paging);
        payload.paging.total = total;
        Ok(payload)
    }

    pub fn get_profile_for_organization(
        &self,
        organization: &Organization,
        conn: &PgConnection,
    ) -> Result<FanProfile, DatabaseError> {
        use schema::*;
        let query = order_items::table
            .inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .filter(orders::status.eq(OrderStatus::Paid))
            .filter(events::organization_id.eq(organization.id))
            .filter(orders::user_id.eq(self.id))
            .select((
                sql::<BigInt>(
                    "cast(COALESCE(sum(
                    CASE WHEN order_items.item_type = 'Tickets'
                    THEN order_items.quantity
                    ELSE 0 END
                    ), 0) as BigInt)",
                ),
                sql::<BigInt>(
                    "cast(COALESCE(sum(order_items.unit_price_in_cents * order_items.quantity), 0) as bigint)",
                ),
                sql::<BigInt>("cast(COALESCE(count(distinct events.id), 0) as BigInt)"),
            ));

        #[derive(Queryable)]
        struct R {
            ticket_sales: i64,
            revenue_in_cents: i64,
            event_count: i64,
        }
        let result: R = query.get_result(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not load profile for organization fan",
        )?;

        Ok(FanProfile {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email: self.email.clone(),
            facebook_linked: self.find_external_login(FACEBOOK_SITE, conn)?.is_some(),
            event_count: result.event_count as u32,
            revenue_in_cents: result.revenue_in_cents as u32,
            ticket_sales: result.ticket_sales as u32,
            profile_pic_url: self.profile_pic_url.clone(),
            thumb_profile_pic_url: self.thumb_profile_pic_url.clone(),
            cover_photo_url: self.cover_photo_url.clone(),
            created_at: self.created_at,
        })
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table.find(id).first::<User>(conn),
        )
    }

    pub fn find_by_email(email: &str, conn: &PgConnection) -> Result<User, DatabaseError> {
        let lower_email = email.to_lowercase();
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::email.eq(lower_email))
                .first::<User>(conn),
        )
    }

    pub fn find_by_phone(phone: &str, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::phone.eq(phone))
                .first::<User>(conn),
        )
    }

    pub fn update(
        &self,
        attributes: &UserEditableAttributes,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        let mut lower_cased_attributes = (*attributes).clone();
        lower_cased_attributes.validate()?;
        if let Some(i) = lower_cased_attributes.email {
            lower_cased_attributes.email = Some(i.to_lowercase());
        }
        let query =
            diesel::update(self).set((lower_cased_attributes, users::updated_at.eq(dsl::now)));

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Error updating user",
            query.get_result(conn),
        )
    }

    pub fn check_password(&self, password: &str) -> bool {
        let hash = match PasswordHash::from_str(&self.hashed_pw) {
            Ok(h) => h,
            Err(_) => return false,
        };
        hash.verify(password)
    }

    pub fn add_role(&self, r: Roles, conn: &PgConnection) -> Result<User, DatabaseError> {
        let mut new_roles = self.role.clone();
        if !new_roles.contains(&r) {
            new_roles.push(r);
        }

        self.update_role(new_roles, conn)
    }

    pub fn remove_role(&self, r: Roles, conn: &PgConnection) -> Result<User, DatabaseError> {
        let mut current_roles = self.role.clone();

        current_roles.retain(|x| x != &r);

        self.update_role(current_roles, conn)
    }

    pub fn has_role(&self, role: Roles) -> bool {
        self.role.contains(&role)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(Roles::Admin)
    }

    pub fn get_global_scopes(&self) -> Vec<String> {
        scopes::get_scopes(self.role.clone())
    }

    pub fn get_roles_by_organization(
        &self,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, Vec<Roles>>, DatabaseError> {
        let mut roles_by_organization = HashMap::new();
        for organization in self.organizations(conn)? {
            roles_by_organization.insert(
                organization.id.clone(),
                organization.get_roles_for_user(self, conn)?,
            );
        }
        Ok(roles_by_organization)
    }

    pub fn get_scopes_by_organization(
        &self,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, Vec<String>>, DatabaseError> {
        let mut scopes_by_organization = HashMap::new();
        for organization in self.organizations(conn)? {
            scopes_by_organization.insert(
                organization.id.clone(),
                organization.get_scopes_for_user(self, conn)?,
            );
        }
        Ok(scopes_by_organization)
    }

    pub fn organizations(&self, conn: &PgConnection) -> Result<Vec<Organization>, DatabaseError> {
        organizations::table
            .left_join(organization_users::table)
            .filter(
                organization_users::user_id
                    .eq(self.id)
                    .or(sql("true=").bind::<sql_types::Bool, _>(self.is_admin())),
            )
            .select(organizations::all_columns)
            .order_by(organizations::name.asc())
            .load::<Organization>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )
    }

    pub fn payment_methods(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<PaymentMethod>, DatabaseError> {
        PaymentMethod::find_for_user(self.id, None, conn)
    }

    pub fn default_payment_method(
        &self,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        PaymentMethod::find_default_for_user(self.id, conn)
    }

    pub fn payment_method(
        &self,
        name: String,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        let mut payment_methods = PaymentMethod::find_for_user(self.id, Some(name), conn)?;
        if payment_methods.is_empty() {
            Err(DatabaseError::new(
                ErrorCode::NoResults,
                Some("No payment method found for user".to_string()),
            ))
        } else {
            Ok(payment_methods.remove(0))
        }
    }

    fn update_role(
        &self,
        new_roles: Vec<Roles>,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update role for user",
            diesel::update(self)
                .set((users::role.eq(new_roles), users::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn find_events_with_access_to_scan(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        let event_start = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(1));

        if self.is_admin() {
            DatabaseError::wrap(
                ErrorCode::QueryError,
                "Error loading scannable events",
                events::table
                    .filter(events::status.eq(EventStatus::Published))
                    .filter(events::event_start.ge(event_start))
                    .order_by(events::event_start.asc())
                    .load(conn),
            )
        } else {
            let user_organizations = self.organizations(conn)?;
            let user_organization_ids: Vec<Uuid> =
                user_organizations.iter().map(|org| org.id).collect();

            let result = events::table
                .filter(events::status.eq(EventStatus::Published))
                .filter(events::event_start.ge(event_start))
                .filter(events::organization_id.eq_any(user_organization_ids))
                .order_by(events::event_start.asc());
            //            println!("SQL {}", diesel::query_builder::debug_query(&result).to_string());
            let result = result.select(events::all_columns).load(conn);
            DatabaseError::wrap(
                ErrorCode::QueryError,
                "Error loading scannable events",
                result,
            )
        }
    }

    pub fn full_name(&self) -> String {
        vec![
            self.first_name.clone().unwrap_or("".to_string()),
            self.last_name.clone().unwrap_or("".to_string()),
        ]
        .join(" ")
    }

    pub fn find_external_login(
        &self,
        site: &str,
        conn: &PgConnection,
    ) -> Result<Option<ExternalLogin>, DatabaseError> {
        ExternalLogin::find_for_site(self.id, site, conn)
    }

    pub fn add_external_login(
        &self,
        external_user_id: String,
        site: String,
        access_token: String,
        conn: &PgConnection,
    ) -> Result<ExternalLogin, DatabaseError> {
        ExternalLogin::create(external_user_id, site, self.id, access_token).commit(conn)
    }

    pub fn wallets(&self, conn: &PgConnection) -> Result<Vec<Wallet>, DatabaseError> {
        Wallet::find_for_user(self.id, conn)
    }

    pub fn update_last_cart(
        &self,
        new_cart_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        // diesel does not have any easy way of handling "last_cart_id is null OR last_cart_id = 'x'"
        let query = if self.last_cart_id.is_none() {
            diesel::update(
                users::table
                    .filter(users::id.eq(self.id))
                    .filter(users::updated_at.eq(self.updated_at))
                    .filter(users::last_cart_id.is_null()),
            )
            .into_boxed()
        } else {
            diesel::update(
                users::table
                    .filter(users::id.eq(self.id))
                    .filter(users::updated_at.eq(self.updated_at))
                    .filter(users::last_cart_id.eq(self.last_cart_id)),
            )
            .into_boxed()
        };
        let rows_affected = query
            .set((
                users::last_cart_id.eq(new_cart_id),
                users::updated_at.eq(dsl::now),
            ))
            .execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update last cart on user")?;

        match rows_affected {
        1 => {
           Ok(())
        },

        _ => DatabaseError::concurrency_error("Could not update last cart on user because the row has been changed by another source")
    }
    }
}

impl From<User> for DisplayUser {
    fn from(user: User) -> Self {
        DisplayUser {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            profile_pic_url: user.profile_pic_url,
            thumb_profile_pic_url: user.thumb_profile_pic_url,
            cover_photo_url: user.cover_photo_url,
            is_org_owner: false,
        }
    }
}

impl ForDisplay<DisplayUser> for User {
    fn for_display(self) -> Result<DisplayUser, DatabaseError> {
        Ok(self.into())
    }
}
