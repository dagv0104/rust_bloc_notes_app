use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use chrono::Utc;

use crate::{
    models::{User, Note, NewUser, NewNote, UpdateNote, AuthData},
    schema::{users, notes},
    errors::ServiceError,
    auth::{create_jwt, get_user_id_from_token},
};

type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

type HandlerResult = Result<HttpResponse, ServiceError>;

// Registro de usuario
pub async fn register(
    pool: web::Data<DbPool>,
    user_data: web::Json<AuthData>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    
    // Verificar si el usuario ya existe
    let exists: bool = diesel::select(diesel::dsl::exists(
        users::table.filter(users::username.eq(&user_data.username))
    ))
    .get_result(&mut conn)
    .map_err(|_| ServiceError::InternalServerError)?;
    
    if exists {
        return Err(ServiceError::BadRequest("Username already exists".into()));
    }
    
    // Hashear la contraseña
    let hashed_password = hash(&user_data.password, DEFAULT_COST)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    // Crear nuevo usuario
    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: user_data.username.clone(),
        password: hashed_password,
    };
    
    // Insertar en la base de datos
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    Ok(HttpResponse::Created().json("User registered successfully"))
}

// Login de usuario
pub async fn login(
    pool: web::Data<DbPool>,
    auth_data: web::Json<AuthData>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    
    // Buscar usuario en la base de datos
    let user: User = users::table
        .filter(users::username.eq(&auth_data.username))
        .first(&mut conn)
        .map_err(|_| ServiceError::Unauthorized("Invalid credentials".into()))?;
    
    // Verificar contraseña
    if !verify(&auth_data.password, &user.password)
        .map_err(|_| ServiceError::InternalServerError)? 
    {
        return Err(ServiceError::Unauthorized("Invalid credentials".into()));
    }
    
    // Generar token JWT
    let token = create_jwt(&user.id)?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user_id": user.id
    })))
}

// Obtener todas las notas del usuario
pub async fn get_notes(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    let user_id = get_user_id_from_token(req)?;
    
    let user_notes: Vec<Note> = notes::table
        .filter(notes::user_id.eq(user_id))
        .order(notes::created_at.desc())
        .load(&mut conn)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    Ok(HttpResponse::Ok().json(user_notes))
}

// Crear una nueva nota
pub async fn create_note(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    note_data: web::Json<NewNote>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    let user_id = get_user_id_from_token(req)?;
    
    let new_note = NewNote {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        title: note_data.title.clone(),
        content: note_data.content.clone(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    
    diesel::insert_into(notes::table)
        .values(&new_note)
        .execute(&mut conn)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    Ok(HttpResponse::Created().json(new_note))
}

// Obtener una nota específica
pub async fn get_note(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    note_id: web::Path<String>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    let user_id = get_user_id_from_token(req)?;
    let note_id_str = note_id.into_inner();
    
    let note: Note = notes::table
        .filter(notes::id.eq(&note_id_str))
        .filter(notes::user_id.eq(user_id))
        .first(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;
    
    Ok(HttpResponse::Ok().json(note))
}

// Actualizar una nota
pub async fn update_note(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    note_id: web::Path<String>,
    note_data: web::Json<UpdateNote>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    let user_id = get_user_id_from_token(req)?;
    let note_id_str = note_id.into_inner();
    
    // Verificar que la nota exista y pertenezca al usuario
    let exists: bool = diesel::select(diesel::dsl::exists(
        notes::table
            .filter(notes::id.eq(&note_id_str))
            .filter(notes::user_id.eq(user_id))
    ))
    .get_result(&mut conn)
    .map_err(|_| ServiceError::InternalServerError)?;
    
    if !exists {
        return Err(ServiceError::NotFound);
    }
    
    // Preparar datos de actualización
    let update_data = UpdateNote {
        title: note_data.title.clone(),
        content: note_data.content.clone(),
        updated_at: Some(Utc::now().naive_utc()),
    };
    
    // Actualizar la nota
    diesel::update(notes::table.find(&note_id_str))
        .set(&update_data)
        .execute(&mut conn)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    // Devolver la nota actualizada
    let updated_note: Note = notes::table
        .find(&note_id_str)
        .first(&mut conn)
        .map_err(|_| ServiceError::InternalServerError)?;
    
    Ok(HttpResponse::Ok().json(updated_note))
}

// Eliminar una nota
pub async fn delete_note(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    note_id: web::Path<String>,
) -> HandlerResult {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalServerError)?;
    let user_id = get_user_id_from_token(req)?;
    let note_id_str = note_id.into_inner();
    
    // Verificar que la nota exista y pertenezca al usuario
    let deleted = diesel::delete(
        notes::table
            .filter(notes::id.eq(&note_id_str))
            .filter(notes::user_id.eq(user_id))
    )
    .execute(&mut conn)
    .map_err(|_| ServiceError::InternalServerError)?;
    
    if deleted == 0 {
        return Err(ServiceError::NotFound);
    }
    
    Ok(HttpResponse::Ok().json("Note deleted successfully"))
}
trait DbOperations {
    fn get_conn(&self, pool: &web::Data<DbPool>) -> Result<SqliteConnection, ServiceError>;
    fn verify_user_note(&self, user_id: &str, note_id: &str) -> Result<bool, ServiceError>;
}