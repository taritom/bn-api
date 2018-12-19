use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::{DomainEvent, DomainEventTypes, ForDisplay, Tables};
use schema::*;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[derive(Clone, Debug, Identifiable, PartialEq, Queryable)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub provider: String,
    pub provider_data: serde_json::Value,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "payment_methods"]
pub struct PaymentMethodEditableAttributes {
    pub provider_data: Option<serde_json::Value>,
}

impl PaymentMethod {
    pub fn create(
        user_id: Uuid,
        name: String,
        is_default: bool,
        provider: String,
        data: serde_json::Value,
    ) -> NewPaymentMethod {
        NewPaymentMethod {
            user_id,
            name,
            is_default,
            provider,
            provider_data: data,
        }
    }

    pub fn find_default_for_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        payment_methods::table
            .filter(payment_methods::user_id.eq(user_id))
            .filter(payment_methods::is_default.eq(true))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load default payment method for user",
            )
    }

    pub fn find_for_user(
        user_id: Uuid,
        name: Option<String>,
        conn: &PgConnection,
    ) -> Result<Vec<PaymentMethod>, DatabaseError> {
        let mut query = payment_methods::table
            .filter(payment_methods::user_id.eq(user_id))
            .into_boxed();

        if let Some(name) = name {
            query = query.filter(payment_methods::name.eq(name));
        }

        query
            .order_by(payment_methods::name)
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load payment methods for user",
            )
    }

    pub fn update(
        &self,
        attributes: &PaymentMethodEditableAttributes,
        current_user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        DomainEvent::create(
            DomainEventTypes::PaymentMethodUpdated,
            "Payment method was updated".to_string(),
            Tables::PaymentMethods,
            Some(self.id),
            Some(current_user_id),
            Some(self.provider_data.clone()),
        )
        .commit(conn)?;

        let query =
            diesel::update(self).set((attributes, payment_methods::updated_at.eq(dsl::now)));

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Error updating payment_method",
            query.get_result(conn),
        )
    }
}

impl ForDisplay<DisplayPaymentMethod> for PaymentMethod {
    fn for_display(self) -> Result<DisplayPaymentMethod, DatabaseError> {
        Ok(self.into())
    }
}

#[derive(Insertable)]
#[table_name = "payment_methods"]
pub struct NewPaymentMethod {
    user_id: Uuid,
    name: String,
    is_default: bool,
    provider: String,
    provider_data: serde_json::Value,
}

impl NewPaymentMethod {
    pub fn commit(
        self,
        current_user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        let payment_method = diesel::insert_into(payment_methods::table)
            .values(self)
            .get_result::<PaymentMethod>(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create payment method")?;

        DomainEvent::create(
            DomainEventTypes::PaymentMethodCreated,
            "Payment method was created".to_string(),
            Tables::PaymentMethods,
            Some(payment_method.id),
            Some(current_user_id),
            Some(payment_method.provider_data.clone()),
        )
        .commit(conn)?;

        Ok(payment_method)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DisplayPaymentMethod {
    pub name: String,
    pub is_default: bool,
}

impl From<PaymentMethod> for DisplayPaymentMethod {
    fn from(payment_method: PaymentMethod) -> Self {
        DisplayPaymentMethod {
            name: payment_method.name,
            is_default: payment_method.is_default,
        }
    }
}
