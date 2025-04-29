use crate::versedb::{bytes_to_string, string_to_bytes, VerseDb};
use anyhow::Result;
use futures::lock::Mutex;
use futures::lock::MutexGuard;
use lazy_static::lazy_static;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*, web_sys};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Serialize, Deserialize, Clone)]
struct NameEntry {
    id: i64,
    name: String,
    created_at: String,
}

// Global database instance
lazy_static! {
    static ref DB: Arc<Mutex<Option<VerseDb>>> = Arc::new(Mutex::new(None));
}

async fn init_db() -> Result<()> {
    let mut db_guard = DB.lock().await;
    if db_guard.is_none() {
        *db_guard = Some(VerseDb::new("test_db").await?);
    }
    Ok(())
}

async fn store_name(name: &str) -> Result<()> {
    init_db().await?;
    let mut db_guard = DB.lock().await;
    let db = db_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

    let date = js_sys::Date::new_0();
    let entry = NameEntry {
        id: date.get_time() as i64,
        name: name.to_string(),
        created_at: date
            .to_locale_string("en-US", &JsValue::from_str(""))
            .into(),
    };

    let key = format!("name:{}", entry.id);
    let value = serde_json::to_string(&entry)?;

    db.store(&string_to_bytes(&key), &string_to_bytes(&value))
        .await?;
    Ok(())
}

async fn get_names() -> Result<Vec<NameEntry>> {
    init_db().await?;
    let db_guard = DB.lock().await;
    let db = db_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

    let start = string_to_bytes("name:");
    let end = string_to_bytes("name:~");
    let entries = db.get_range(&start, &end).await?;

    let mut names = Vec::new();
    for (_, value) in entries {
        if let Ok(entry) = serde_json::from_str::<NameEntry>(&bytes_to_string(&value)) {
            names.push(entry);
        }
    }

    names.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(names)
}

async fn greet_handler(name: &str) -> String {
    if let Err(e) = store_name(name).await {
        log(&format!("Error storing name: {:?}", e));
    }

    let window = web_sys::window().unwrap();
    if js_sys::Reflect::has(&window, &JsValue::from_str("__TAURI__")).unwrap_or(false) {
        let args = serde_wasm_bindgen::to_value(&GreetArgs { name }).unwrap();
        invoke("greet", args)
            .await
            .as_string()
            .unwrap_or_else(|| "Error calling Tauri".to_string())
    } else {
        format!("Hello, {}! (Web Mode)", name)
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());
    let (names, set_names) = signal(Vec::<NameEntry>::new());

    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };

    let load_names = move || {
        spawn_local(async move {
            if let Ok(names_list) = get_names().await {
                set_names.set(names_list);
            }
        });
    };

    // Load names on component mount
    load_names();

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                leptos::logging::log!("Name is empty, returning early");
                return;
            }

            leptos::logging::log!("Processing greeting for name: {}", name);
            let new_msg = greet_handler(&name).await;
            leptos::logging::log!("Received message: {}", new_msg);
            set_greet_msg.set(new_msg);
            set_name.set("".to_string());

            // Reload names after adding a new one
            load_names();
        });
    };

    view! {
        <main class="container">
            <h1>"Welcome to Tauri + Leptos2"</h1>

            <div class="row">
                <a href="https://tauri.app" target="_blank">
                    <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank">
                    <img src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
                </a>
            </div>
            <p>"Click on the Tauri and Leptos logos to learn more."</p>

            <form class="row" on:submit=greet>
                <input
                    id="greet-input"
                    placeholder="Enter a name..."
                    on:input=update_name
                    prop:value=name
                />
                <button type="submit">"Greet"</button>
            </form>

            <p>{ move || greet_msg.get() }</p>

            <div class="names-list">
                <h2>"Previous Names"</h2>
                <style>
                    r#"
                    table {
                        width: 100%;
                        border-collapse: collapse;
                        margin-top: 1rem;
                        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                    }
                    th, td {
                        border: 1px solid #e0e0e0;
                        padding: 12px;
                        text-align: left;
                    }
                  
                    "#
                </style>
                <table>
                    <thead>
                        <tr>
                            <th>"Name"</th>
                            <th>"Date Added"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || names.get().iter().map(|entry| view! {
                            <tr>
                                <td>{entry.name.clone()}</td>
                                <td>{entry.created_at.clone()}</td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </main>
    }
}
