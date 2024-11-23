use crate::prisma;
use password_auth::{generate_hash, verify_password};
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
    pub session_id: Option<[u8; 32]>,
}
struct LoggedInState {
    // app_state: AppState,
    db: Arc<prisma::PrismaClient>,
    session_id: [u8; 32],
    user: prisma::user::Data,
}
#[derive(Serialize, Deserialize, Type)]
struct UserMsg {
    email: String,
    password: String,
}
fn rspc_error(e: prisma_client_rust::QueryError) -> rspc::Error {
    // e.into()
    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
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
        .mutation("login", |t| {
            t(|ctx: AppState, input: UserMsg| async move {
                let user = ctx
                    .db
                    .user()
                    .find_unique(prisma::user::UniqueWhereParam::EmailEquals(input.email))
                    .exec()
                    .await
                    .map_err(rspc_error)?;
                if let Some(user) = user {
                    if let Ok(_) = verify_password(input.password, &user.password) {
                        let session_id: [u8; 32] = random();
                        let sess = ctx
                            .db
                            .session()
                            .create(
                                session_id.to_vec(),
                                prisma::user::UniqueWhereParam::IdEquals(user.id),
                                vec![],
                            )
                            .exec()
                            .await
                            .map_err(rspc_error)?;
                        return Ok(sess);
                    }
                };
                Err(rspc::Error::new(
                    rspc::ErrorCode::Unauthorized,
                    "user email not found".to_string(),
                ))
            })
        })
        .middleware(|mw| {
            mw.middleware(|mw| async move {
                let old_ctx = mw.ctx.clone();
                if let Some(session_id) = old_ctx.session_id {
                    let user = old_ctx
                        .db
                        .session()
                        .find_unique(prisma::session::UniqueWhereParam::SessionIdEquals(
                            session_id.to_vec(),
                        ))
                        .with(prisma::session::user::fetch())
                        .exec()
                        .await
                        .map_err(rspc_error)?;
                    if let Some(user) = user {
                        if let Some(user) = user.user {
                            return Ok(mw.with_ctx(LoggedInState {
                                db: old_ctx.db.clone(),
                                session_id,
                                user: *user,
                            }));
                        }
                    }
                };
                Err(rspc::Error::new(
                    rspc::ErrorCode::Unauthorized,
                    "not logged in".to_string(),
                ))
            })
        })
        .mutation("logout", |t| {
            t(|ctx: LoggedInState, _: ()| {
                ctx.db
                    .session()
                    .delete(prisma::session::UniqueWhereParam::SessionIdEquals(
                        ctx.session_id.to_vec(),
                    ));
            })
        })
        .query("ping", |t| {
            t(|ctx: LoggedInState, _: ()| ("pong", ctx.user.email))
        })
        .build()
        .arced()
}
