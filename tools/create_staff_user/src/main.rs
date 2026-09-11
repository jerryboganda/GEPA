//! Create (or reset) a reviewer/admin account in `staff_users`
//! (DECISIONS.md D-021 — replaces Firebase Google sign-in + custom claims).
//!
//! Usage: cargo run --bin create_staff_user -- <email> <password> <reviewer|admin>

use server::auth::hash_password;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [email, password, role] = args.as_slice() else {
        eprintln!("Usage: create_staff_user <email> <password> <reviewer|admin>");
        std::process::exit(1);
    };
    if role != "reviewer" && role != "admin" {
        eprintln!("role must be 'reviewer' or 'admin', got: {role}");
        std::process::exit(1);
    }

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://gepa:gepa@localhost:5432/gepa".to_string());
    let (client, connection) = tokio_postgres::connect(&database_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("postgres connection error: {e}");
        }
    });

    client
        .batch_execute(include_str!("../../../server/migrations/001_init.sql"))
        .await?;

    let password_hash = hash_password(password).map_err(|e| format!("hash failed: {e}"))?;
    let id = format!("staff_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]);

    client
        .execute(
            "INSERT INTO staff_users (id, email, password_hash, role) VALUES ($1, $2, $3, $4)
             ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash, role = EXCLUDED.role",
            &[&id, email, &password_hash, role],
        )
        .await?;

    println!("Staff user ready: {email} (role: {role})");
    Ok(())
}
