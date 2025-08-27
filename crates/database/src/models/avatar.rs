use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::{FromRow, query_as};
use uuid::Uuid;

use crate::db;
use crate::utils::error::{DatabaseError, ModelResult};

/// The width of the avatar canvas in pixels.
const AVATAR_CANVAS_WIDTH: i32 = 16;

#[derive(FromRow)]
pub struct AvatarModel {
    /// The unique identifier of the user who owns the avatar.
    user_id: Uuid,

    /// A collection of pixel data, where each entry contains:
    /// - offset (i32) calculated as `x * y * canvas_width`
    /// - color (i32) representing the pixel color
    ///
    /// Example: `[[offset1, color1], [offset2, color2], ...]`
    /// This structure is optimized for frontend rendering.
    pixels: JsonValue,

    /// The timestamp of the last update made to the avatar.
    updated_at: NaiveDateTime,
}

/// A data structure representing the input required for creating a new avatar.
///
/// Fields:
/// * `pixels`: A vector of pixel data, where each entry consists of:
///   - offset: An integer calculated as `x * y * canvas_width`.
///   - color: An integer representing the pixel color.
/// * `user_id`: The unique identifier of the user who owns the avatar.
#[derive(Deserialize)]
pub struct AvatarCreation {
    pixels: Vec<[i32; 2]>,
    user_id: String,
}

/// A data structure used to return the result of avatar operations.
///
/// Fields:
/// * `user_id`: The unique identifier of the user who owns the avatar (UUID string).
/// * `pixels`: A vector of pixel data, where each entry contains:
///   - x-coordinate (i32)
///   - y-coordinate (i32)
///   - color (i32)
/// * `updated_at`: The timestamp of the last update made to the avatar.
#[derive(Serialize)]
pub struct AvatarResult {
    user_id: String,
    pixels: Vec<[i32; 3]>,
    updated_at: NaiveDateTime,
}

impl AvatarModel {
    /// Creates a new avatar based on the provided input data.
    ///
    /// # Arguments
    /// * `creation` - An instance of `AvatarCreation` containing the necessary data for creating the avatar.
    ///
    /// # Returns
    /// A `ModelResult` containing the newly created `AvatarModel` on success, or an appropriate error.
    pub async fn create_new(creation: AvatarCreation) -> ModelResult<Self> {
        let user_id = Uuid::try_from(creation.user_id).map_err(DatabaseError::from)?;

        let avatar = query_as!(
            Self,
            r#"
                INSERT INTO user_avatars (
                    user_id,
                    pixels
                )
                VALUES ($1, $2)
                ON CONFLICT (user_id) DO UPDATE
                SET
                    pixels = EXCLUDED.pixels,
                    updated_at = NOW()
                RETURNING *
            "#,
            user_id,
            serde_json::to_value(creation.pixels).map_err(DatabaseError::from)?
        )
        .fetch_one(db!())
        .await?;

        Ok(avatar)
    }

    /// Converts the `AvatarModel` instance into an `AvatarResult`.
    ///
    /// # Returns
    /// An instance of `AvatarResult` containing the transformed data.
    pub fn to_result(&self) -> AvatarResult {
        let pixels: Vec<[i32; 3]> = self
            .pixels
            .as_array()
            .and_then(|array| {
                array
                    .iter()
                    .map(|entry| {
                        entry.as_array().and_then(|arr| {
                            if arr.len() == 2 {
                                let offset = arr[0].as_i64()? as i32;
                                let color = arr[1].as_i64()? as i32;

                                return Some([
                                    offset % AVATAR_CANVAS_WIDTH, // x-coordinate
                                    offset / AVATAR_CANVAS_WIDTH, // y-coordinate
                                    color,
                                ]);
                            }

                            None
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        AvatarResult {
            user_id: self.user_id.to_string(),
            pixels,
            updated_at: self.updated_at,
        }
    }
}
