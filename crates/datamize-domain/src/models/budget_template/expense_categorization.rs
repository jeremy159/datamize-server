use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{Category, CategoryGroup, CategoryGroupWithCategories};

use super::{ExpenseType, SubExpenseType};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow, PartialEq, Eq, Hash)]
pub struct ExpenseCategorization {
    pub id: Uuid,
    pub name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    #[sqlx(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
}

impl ExpenseCategorization {
    pub fn new(id: Uuid, name: String) -> Self {
        Self {
            id,
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CategoryGroupWithCategoriesConversionError;

impl TryFrom<CategoryGroupWithCategories> for ExpenseCategorization {
    type Error = CategoryGroupWithCategoriesConversionError;

    fn try_from(value: CategoryGroupWithCategories) -> Result<Self, Self::Error> {
        if value.deleted || value.hidden {
            return Err(CategoryGroupWithCategoriesConversionError);
        }

        if value.name == "Hidden Categories"
            || value.name == "Internal Master Category"
            || value.name == "Credit Card Payments"
        {
            return Err(CategoryGroupWithCategoriesConversionError);
        }

        Ok(ExpenseCategorization::new(value.id, value.name))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CategoryGroupConversionError;

impl TryFrom<CategoryGroup> for ExpenseCategorization {
    type Error = CategoryGroupConversionError;

    fn try_from(value: CategoryGroup) -> Result<Self, Self::Error> {
        if value.deleted || value.hidden {
            return Err(CategoryGroupConversionError);
        }

        if value.name == "Hidden Categories"
            || value.name == "Internal Master Category"
            || value.name == "Credit Card Payments"
        {
            return Err(CategoryGroupConversionError);
        }

        Ok(ExpenseCategorization::new(value.id, value.name))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CategoryConversionError;

impl TryFrom<Category> for ExpenseCategorization {
    type Error = CategoryConversionError;

    fn try_from(value: Category) -> Result<Self, Self::Error> {
        if value.deleted || value.hidden {
            return Err(CategoryConversionError);
        }

        if value.category_group_name == "Hidden Categories"
            || value.category_group_name == "Internal Master Category"
            || value.category_group_name == "Credit Card Payments"
        {
            return Err(CategoryConversionError);
        }

        Ok(ExpenseCategorization::new(
            value.category_group_id,
            value.category_group_name,
        ))
    }
}
