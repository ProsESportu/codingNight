use crate::prisma;
use password_auth::generate_hash;
use rand::prelude::*;
// use argon2::Argon2;
// use password_hash::PasswordHash;
use rspc::{Config, Router};
use serde::{Deserialize, Serialize};
// use sha2::{Digest, Sha512};
use specta::Type;
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<prisma::PrismaClient>,
}
#[derive(Serialize, Deserialize, Type)]
struct UserMsg {
    email: String,
    password: String,
}
pub fn router() -> Arc<Router<AppState>> {
    Router::<AppState>::new()
        .config(Config::new().export_ts_bindings("./bind.ts"))
        .query("version", |t| {
            t(|_ctx: AppState, _input: ()| env!("CARGO_PKG_VERSION"))
        })
        .mutation("create_user", |t| {
            t(|ctx: AppState, input: UserMsg| async move {
                let hash = generate_hash(input.password);
                let session_id: [u8; 32] = random();
                let db = ctx.db;
                // let db = match ctx.db.lock() {
                //     Ok(db) => db,
                //     Err(e) => {
                //         return Err(rspc::Error::new(
                //             rspc::ErrorCode::InternalServerError,
                //             e.to_string(),
                //         ))
                //     }
                // };
                fn rspc_error(e: prisma_client_rust::QueryError) -> rspc::Error {
                    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
                }
                let user = db
                    .user()
                    .create(input.email, hash, vec![])
                    .exec()
                    .await
                    .map_err(rspc_error)?;
                let sess = db
                    .session()
                    .create(
                        session_id.to_vec(),
                        prisma::user::UniqueWhereParam::IdEquals(user.id),
                        vec![],
                    )
                    .exec()
                    .await
                    .map_err(rspc_error)?;
                Ok(sess)
            })
        })
        .build()
        .arced()
}
