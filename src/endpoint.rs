use serde::Serialize;

/// HTTP methods used in API.
#[derive(Serialize, Clone, Debug)]
pub enum HttpMethod {
    GET,
    PUT,
    DELETE,
}

/// Endpoint represented by url and HTTP method.
#[derive(Serialize, Clone, Debug)]
pub struct Endpoint {
    url: String,
    method: HttpMethod,
}

impl Endpoint {
    /// Create new endpoint at provided url with support for HTTP `Get`
    pub fn get(url: String) -> Self {
        Endpoint {
            url,
            method: HttpMethod::GET,
        }
    }

    /// Create new endpoint at provided url with support for HTTP `Put`
    pub fn put(url: String) -> Self {
        Endpoint {
            url,
            method: HttpMethod::PUT,
        }
    }

    /// Create new endpoint at provided url with support for HTTP `Delete`
    pub fn delete(url: String) -> Self {
        Endpoint {
            url,
            method: HttpMethod::DELETE,
        }
    }
}

/// Endpoints returned on call to `me` endpoint without valid token
#[derive(Serialize, Clone, Debug)]
pub struct InitEndpoints {
    login: Endpoint,
    signup: Endpoint,
}

/// Endpoints returned on call to `login` endpoint or `me` endpoint with valid token
#[derive(Serialize, Clone, Debug)]
pub struct UserEndpoints {
    logout: Endpoint,
    delete_user: Endpoint,
    list_notes: Endpoint,
    save_note: Endpoint,
}

/// Endpoints returned on call to `list notes` endpoint.
#[derive(Serialize, Clone, Debug)]
pub struct NotesEndpoints {
    get: Endpoint,
    delete: Endpoint,
}

/// Endpoints returned on call to `get note` endpoint.
#[derive(Serialize, Clone, Debug)]
pub struct GetNoteEndpoints {
    update: Endpoint,
    delete: Endpoint,
}

/// Endpoints returned on call to `delete note` endpoint.
#[derive(Serialize, Clone, Debug)]
pub struct DeleteNoteEndpoints {
    undelete: Endpoint,
}

/// Get init endpoints.
pub fn get_init_endpoints() -> InitEndpoints {
    let base_url = dotenv::var("BASE_URL").unwrap();

    let login_url = format!("{}/{}", base_url, "session");
    let signup_url = format!("{}/{}", base_url, "user");

    InitEndpoints {
        login: Endpoint::put(login_url),
        signup: Endpoint::put(signup_url),
    }
}

/// Get user endpoints.
pub fn get_user_endpoints() -> UserEndpoints {
    let base_url = dotenv::var("BASE_URL").unwrap();

    let logout_url = format!("{}/{}", base_url, "session");
    let delete_user_url = format!("{}/{}", base_url, "user");
    let list_notes_url = format!("{}/{}", base_url, "notes");
    let save_note_url = format!("{}/{}", base_url, "notes");

    UserEndpoints {
        logout: Endpoint::delete(logout_url),
        delete_user: Endpoint::delete(delete_user_url),
        list_notes: Endpoint::get(list_notes_url),
        save_note: Endpoint::put(save_note_url),
    }
}

// List notes endpoints.
pub fn list_notes_endpoints(id: &str) -> NotesEndpoints {
    let base_url = dotenv::var("BASE_URL").unwrap();

    let get_note_url = format!("{}/{}/{}", base_url, "notes", id);
    let delete_note_url = format!("{}/{}/{}", base_url, "notes", id);

    NotesEndpoints {
        get: Endpoint::get(get_note_url),
        delete: Endpoint::delete(delete_note_url),
    }
}

// Get note endpoints.
pub fn get_note_endpoints(id: &str) -> GetNoteEndpoints {
    let base_url = dotenv::var("BASE_URL").unwrap();

    let update_note_url = format!("{}/{}/{}", base_url, "notes", id);
    let delete_note_url = format!("{}/{}/{}", base_url, "notes", id);

    GetNoteEndpoints {
        update: Endpoint::put(update_note_url),
        delete: Endpoint::delete(delete_note_url),
    }
}

// Delete note endpoints.
pub fn delete_note_endpoints(id: &str) -> DeleteNoteEndpoints {
    let base_url = dotenv::var("BASE_URL").unwrap();

    let undelete_note_url = format!("{}/{}/{}", base_url, "notes/undelete", id);

    DeleteNoteEndpoints {
        undelete: Endpoint::put(undelete_note_url),
    }
}
