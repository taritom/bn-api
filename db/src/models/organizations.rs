use chrono::NaiveDateTime;
use diesel;
use diesel::dsl::{exists, select};
use diesel::expression::dsl;
use diesel::expression::sql_literal::sql;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text, Timestamp};
use models::scopes;
use models::*;
use schema::{events, organization_users, organizations, users, venues};
use serde_with::rust::double_option;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, QueryableByName, AsChangeset)]
#[belongs_to(User, foreign_key = "owner_user_id")]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "organizations"]
pub struct Organization {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub event_fee_in_cents: Option<i64>,
    pub sendgrid_api_key: Option<String>,
    pub google_ga_key: Option<String>,
    pub facebook_pixel_key: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub fee_schedule_id: Uuid,
}

#[derive(Serialize)]
pub struct DisplayOrganizationLink {
    pub id: Uuid,
    pub name: String,
    pub role: String,
}

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "organizations"]
pub struct NewOrganization {
    pub owner_user_id: Uuid,
    pub name: String,
    pub fee_schedule_id: Uuid,
    pub event_fee_in_cents: Option<i64>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub sendgrid_api_key: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub google_ga_key: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub facebook_pixel_key: Option<String>,
}

impl NewOrganization {
    pub fn commit(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        let db_err = diesel::insert_into(organizations::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new organization")?;

        Ok(db_err)
    }
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "organizations"]
pub struct OrganizationEditableAttributes {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub city: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub state: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub country: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub postal_code: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub event_fee_in_cents: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub sendgrid_api_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub google_ga_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub facebook_pixel_key: Option<Option<String>>,
}

impl Organization {
    pub fn create(owner_user_id: Uuid, name: &str, fee_schedule_id: Uuid) -> NewOrganization {
        NewOrganization {
            owner_user_id: owner_user_id,
            name: name.into(),
            fee_schedule_id,
            ..Default::default()
        }
    }

    pub fn update(
        &self,
        attributes: OrganizationEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set((attributes, organizations::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update organization")
    }

    pub fn set_owner(
        &self,
        owner_user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set((
                organizations::owner_user_id.eq(owner_user_id),
                organizations::updated_at.eq(dsl::now),
            ))
            .get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not update organization owner",
            )
    }

    pub fn users(&self, conn: &PgConnection) -> Result<Vec<User>, DatabaseError> {
        let organization_users = OrganizationUser::belonging_to(self);
        let organization_owner = users::table
            .find(self.owner_user_id)
            .first::<User>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;
        let mut users = organization_users
            .inner_join(users::table)
            .filter(users::id.ne(self.owner_user_id))
            .select(users::all_columns)
            .order_by(users::last_name.asc())
            .then_order_by(users::first_name.asc())
            .load::<User>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;

        users.insert(0, organization_owner);
        Ok(users)
    }

    pub fn venues(&self, conn: &PgConnection) -> Result<Vec<Venue>, DatabaseError> {
        venues::table
            .filter(venues::organization_id.eq(self.id))
            .order_by(venues::name)
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve venues")
    }

    pub fn has_fan(&self, user: &User, conn: &PgConnection) -> Result<bool, DatabaseError> {
        use schema::*;

        select(exists(
            order_items::table
                .inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
                .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
                .filter(orders::status.eq(OrderStatus::Paid.to_string()))
                .filter(events::organization_id.eq(self.id))
                .filter(orders::user_id.eq(user.id)),
        ))
        .get_result(conn)
        .to_db_error(
            ErrorCode::QueryError,
            "Could not check if organization has fan",
        )
    }

    pub fn get_scopes_for_user(
        &self,
        user: &User,
        conn: &PgConnection,
    ) -> Result<Vec<String>, DatabaseError> {
        Ok(scopes::get_scopes(self.get_roles_for_user(user, conn)?))
    }

    pub fn get_roles_for_user(
        &self,
        user: &User,
        conn: &PgConnection,
    ) -> Result<Vec<String>, DatabaseError> {
        let mut roles = Vec::new();
        if user.id == self.owner_user_id || user.is_admin() {
            roles.push(Roles::OrgOwner.to_string());
            roles.push(Roles::OrgMember.to_string());
        } else {
            if self.is_member(user, conn)? {
                roles.push(Roles::OrgMember.to_string());
            }
        }

        Ok(roles)
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        organizations::table
            .find(id)
            .first::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading organization")
    }

    pub fn find_for_event(
        event_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        events::table
            .inner_join(organizations::table)
            .filter(events::id.eq(event_id))
            .select(organizations::all_columns)
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not find organization for this event",
            )
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<Organization>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organizations::table
                .order_by(organizations::name)
                .load(conn),
        )
    }

    pub fn owner(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load owner",
            users::table.find(self.owner_user_id).first::<User>(conn),
        )
    }

    pub fn all_linked_to_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Organization>, DatabaseError> {
        let orgs = organizations::table
            .filter(organizations::owner_user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations");

        let mut orgs = match orgs {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        let mut org_members = organization_users::table
            .filter(organization_users::user_id.eq(user_id))
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;

        orgs.append(&mut org_members);
        orgs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(orgs)
    }

    pub fn all_org_names_linked_to_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrganizationLink>, DatabaseError> {
        //Compile list of organisations where the user is the owner
        let org_owner_list: Vec<Organization> = organizations::table
            .filter(organizations::owner_user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;
        //Compile list of organisations where the user is a member of that organisation
        let org_member_list: Vec<Organization> = organization_users::table
            .filter(organization_users::user_id.eq(user_id))
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;
        //Compile complete list with only display information
        let role_owner_string = String::from("owner");
        let role_member_string = String::from("member");
        let mut result_list: Vec<DisplayOrganizationLink> = Vec::new();
        for curr_org_owner in &org_owner_list {
            let curr_entry = DisplayOrganizationLink {
                id: curr_org_owner.id,
                name: curr_org_owner.name.clone(),
                role: role_owner_string.clone(),
            };
            result_list.push(curr_entry);
        }
        for curr_org_member in &org_member_list {
            let curr_entry = DisplayOrganizationLink {
                id: curr_org_member.id,
                name: curr_org_member.name.clone(),
                role: role_member_string.clone(),
            };
            result_list.push(curr_entry);
        }
        result_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result_list)
    }

    pub fn remove_user(&self, user_id: Uuid, conn: &PgConnection) -> Result<usize, DatabaseError> {
        diesel::delete(
            organization_users::table
                .filter(organization_users::user_id.eq(user_id))
                .filter(organization_users::organization_id.eq(self.id)),
        )
        .execute(conn)
        .to_db_error(ErrorCode::DeleteError, "Error removing user")
    }

    pub fn add_user(
        &self,
        user_id: Uuid,
        role: Option<Roles>,
        conn: &PgConnection,
    ) -> Result<OrganizationUser, DatabaseError> {
        let org_user = OrganizationUser::create(self.id, user_id, role).commit(conn)?;
        Ok(org_user)
    }

    pub fn is_member(&self, user: &User, conn: &PgConnection) -> Result<bool, DatabaseError> {
        if self.owner_user_id == user.id {
            return Ok(true);
        }

        let query = select(exists(
            organization_users::table
                .filter(
                    organization_users::user_id
                        .eq(user.id)
                        .and(organization_users::organization_id.eq(self.id)),
                )
                .select(organization_users::organization_id),
        ));
        query
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not check user member status")
    }

    pub fn add_fee_schedule(
        &self,
        fee_schedule: &FeeSchedule,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set((
                organizations::fee_schedule_id.eq(fee_schedule.id),
                organizations::updated_at.eq(dsl::now),
            ))
            .get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not set the fee schedule for this organization",
            )
    }

    pub fn search_fans(
        &self,
        query: Option<String>,
        page: u32,
        limit: u32,
        sort_field: FanSortField,
        sort_direction: SortingDir,
        conn: &PgConnection,
    ) -> Result<Payload<DisplayFan>, DatabaseError> {
        use schema::*;

        let search_filter = format!("%{}%", query.unwrap_or("".to_string()));

        let sort_column = match sort_field {
            FanSortField::FirstName => "2",
            FanSortField::LastName => "3",
            FanSortField::Email => "4",
            FanSortField::Phone => "5",
            FanSortField::Orders => "7",
            FanSortField::FirstOrder => "8",
            FanSortField::LastOrder => "9",
            FanSortField::Revenue => "10",
        };

        let query = order_items::table
            .inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(users::table.on(users::id.eq(orders::user_id)))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .filter(orders::status.eq(OrderStatus::Paid.to_string()))
            .filter(events::organization_id.eq(self.id))
            .filter(
                sql("users.first_name ilike ")
                    .bind::<Text, _>(&search_filter)
                    .sql(" OR users.last_name ilike ")
                    .bind::<Text, _>(&search_filter)
                    .sql(" OR users.email ilike ")
                    .bind::<Text, _>(&search_filter)
                    .sql("or users.phone ilike ")
                    .bind::<Text, _>(&search_filter),
            )
            .group_by((
                events::organization_id,
                users::first_name,
                users::last_name,
                users::email,
                users::phone,
                users::id,
            ))
            .select((
                events::organization_id,
                users::first_name,
                users::last_name,
                users::email,
                users::phone,
                users::thumb_profile_pic_url,
                users::id,
                sql::<BigInt>("count(distinct orders.id)"),
                users::created_at,
                sql::<Timestamp>("min(orders.order_date)"),
                sql::<Timestamp>("max(orders.order_date)"),
                sql::<BigInt>(
                    "cast(sum(order_items.unit_price_in_cents * order_items.quantity) as bigint)",
                ),
                sql::<BigInt>("count(*) over()"),
            ))
            .order_by(sql::<()>(&format!("{} {}", sort_column, sort_direction)));

        let query = query.limit(limit as i64).offset((limit * page) as i64);

        #[derive(Queryable)]
        struct R {
            organization_id: Uuid,
            first_name: Option<String>,
            last_name: Option<String>,
            email: Option<String>,
            phone: Option<String>,
            thumb_profile_pic_url: Option<String>,
            user_id: Uuid,
            order_count: i64,
            created_at: NaiveDateTime,
            first_order_time: NaiveDateTime,
            last_order_time: NaiveDateTime,
            revenue_in_cents: i64,
            total_rows: i64,
        }

        let results: Vec<R> = query.get_results(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not load fans for organization",
        )?;

        let paging = Paging::new(page, limit);
        let mut total = results.len() as u64;
        if !results.is_empty() {
            total = results[0].total_rows as u64;
        }

        let fans = results
            .into_iter()
            .map(|r| DisplayFan {
                user_id: r.user_id,
                first_name: r.first_name,
                last_name: r.last_name,
                email: r.email,
                phone: r.phone,
                thumb_profile_pic_url: r.thumb_profile_pic_url,
                organization_id: r.organization_id,
                order_count: r.order_count as u32,
                created_at: r.created_at,
                first_order_time: r.first_order_time,
                last_order_time: r.last_order_time,
                revenue_in_cents: r.revenue_in_cents,
            })
            .collect();

        let mut p = Payload::new(fans, paging);
        p.paging.total = total;
        Ok(p)
    }
}

#[derive(Serialize)]
pub struct DisplayFan {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub organization_id: Uuid,
    pub order_count: u32,
    pub created_at: NaiveDateTime,
    pub first_order_time: NaiveDateTime,
    pub last_order_time: NaiveDateTime,
    pub revenue_in_cents: i64,
}
