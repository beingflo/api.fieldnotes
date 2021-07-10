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
