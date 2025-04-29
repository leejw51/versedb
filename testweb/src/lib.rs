use anyhow::Result;
use futures::lock::Mutex;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

mod versedb;
use versedb::{bytes_to_string, string_to_bytes, VerseDb};

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
lazy_static::lazy_static! {
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

    let entry = NameEntry {
        id: js_sys::Date::new_0().get_time() as i64,
        name: name.to_string(),
        created_at: js_sys::Date::new_0().to_iso_string().into(),
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
    let (name, set_name) = create_signal(String::new());
    let (greeting, set_greeting) = create_signal(String::new());
    let (names, set_names) = create_signal(Vec::<NameEntry>::new());

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name = name.get();
        if name.is_empty() {
            return;
        }

        spawn_local(async move {
            if let Err(e) = store_name(&name).await {
                log(&format!("Error storing name: {}", e));
            }
            set_name.set(String::new());
            if let Ok(names) = get_names().await {
                set_names.set(names);
            }
        });
    };

    spawn_local(async move {
        if let Ok(names) = get_names().await {
            set_names.set(names);
        }
    });

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

            <form class="row" on:submit=on_submit>
                <input
                    id="greet-input"
                    placeholder="Enter a name..."
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    prop:value=name
                />
                <button type="submit">"Greet"</button>
            </form>

            <p><b>{ move || greeting.get() }</b></p>

            <div class="names-list">
                <h2>"Previous Names"</h2>
                <ul>
                    {move || names.get().into_iter().map(|entry| view! {
                        <li>{format!("{} - {}", entry.name, entry.created_at)}</li>
                    }).collect::<Vec<_>>()}
                </ul>
            </div>
        </main>
    }
}
