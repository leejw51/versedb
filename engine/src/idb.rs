use crate::database::{Database, Result};
use async_trait::async_trait;
use std::error::Error;
use std::fmt;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use serde_json::to_string;
use js_sys::{Uint8Array, Promise};
use web_sys::{IdbDatabase, IdbFactory, IdbTransactionMode, IdbObjectStore, IdbRequest, IdbOpenDbRequest, DomException, DomStringList};

#[derive(Debug)]
struct JsError(DomException);

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IndexedDB error: {:?}", self.0)
    }
}

impl Error for JsError {}

impl From<DomException> for JsError {
    fn from(value: DomException) -> Self {
        JsError(value)
    }
}

#[derive(Debug)]
pub struct IdbDatabaseWrapper {
    db: IdbDatabase,
}

// SAFETY: IdbDatabase is safe to send between threads in WebAssembly
unsafe impl Send for IdbDatabaseWrapper {}
unsafe impl Sync for IdbDatabaseWrapper {}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for IdbDatabaseWrapper {
    async fn open(path: &str) -> Result<Self> {
        let window = web_sys::window().ok_or_else(|| JsError::from(DomException::new().unwrap()))?;
        let factory: IdbFactory = window.indexed_db().map_err(|_| JsError::from(DomException::new().unwrap()))?.expect("Failed to get IndexedDB");
        
        let open_request: IdbOpenDbRequest = factory.open_with_u32(path, 1).map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        // Create store if it doesn't exist
        let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let open_request: IdbOpenDbRequest = target.dyn_into().unwrap();
            let db = open_request.result().unwrap().dyn_into::<IdbDatabase>().unwrap();
            
            let store_names: DomStringList = db.object_store_names();
            let mut found = false;
            for i in 0..store_names.length() {
                if let Some(name) = store_names.get(i) {
                    if name == "store" {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                db.create_object_store("store").unwrap();
            }
        });
        
        open_request.set_onupgradeneeded(Some(callback.as_ref().unchecked_ref()));
        callback.forget(); // Prevent closure from being dropped
        
        // Convert the request to a Promise and await it
        let promise = Promise::new(&mut |resolve, _reject| {
            let request_success = open_request.clone();
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &request_success.result().unwrap()).unwrap();
            });

            let request_error = open_request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            open_request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
        
        let db = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?
            .dyn_into::<IdbDatabase>()
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        Ok(IdbDatabaseWrapper { db })
    }

    async fn close(&mut self) -> Result<()> {
        self.db.close();
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let tx = self.db.transaction_with_str_sequence_and_mode(
            &js_sys::Array::of1(&JsValue::from_str("store")),
            IdbTransactionMode::Readwrite
        ).map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let store = tx.object_store("store").map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let key_js = Uint8Array::from(key);
        let value_js = Uint8Array::from(value);
        let request: IdbRequest = store.put_with_key(&value_js.into(), &key_js.into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let promise = Promise::new(&mut |resolve, _reject| {
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &JsValue::undefined()).unwrap();
            });

            let request_error = request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
        
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let tx = self.db.transaction_with_str_sequence_and_mode(
            &js_sys::Array::of1(&JsValue::from_str("store")),
            IdbTransactionMode::Readonly
        ).map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let store = tx.object_store("store").map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let key_js = Uint8Array::from(key);
        let request: IdbRequest = store.get(&key_js.into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let promise = Promise::new(&mut |resolve, _reject| {
            let request_success = request.clone();
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &request_success.result().unwrap()).unwrap();
            });

            let request_error = request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
            
        let result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        if result.is_undefined() {
            Ok(None)
        } else {
            let array = Uint8Array::new(&result);
            let mut vec = vec![0; array.length() as usize];
            array.copy_to(&mut vec);
            Ok(Some(vec))
        }
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        let tx = self.db.transaction_with_str_sequence_and_mode(
            &js_sys::Array::of1(&JsValue::from_str("store")),
            IdbTransactionMode::Readwrite
        ).map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let store = tx.object_store("store").map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let key_js = Uint8Array::from(key);
        let request: IdbRequest = store.delete(&key_js.into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let promise = Promise::new(&mut |resolve, _reject| {
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &JsValue::undefined()).unwrap();
            });

            let request_error = request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
            
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let tx = self.db.transaction_with_str_sequence_and_mode(
            &js_sys::Array::of1(&JsValue::from_str("store")),
            IdbTransactionMode::Readonly
        ).map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        let store = tx.object_store("store").map_err(|_| JsError::from(DomException::new().unwrap()))?;
        
        // Create a key range
        let start_key = Uint8Array::from(start);
        let end_key = Uint8Array::from(end);
        let key_range = web_sys::IdbKeyRange::bound(&start_key.into(), &end_key.into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        // Get all entries within the range
        let request: IdbRequest = store.get_all_with_key(&key_range.clone().into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let promise = Promise::new(&mut |resolve, _reject| {
            let request_success = request.clone();
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &request_success.result().unwrap()).unwrap();
            });

            let request_error = request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
            
        let result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let array = js_sys::Array::from(&result);
        let mut items = Vec::new();
        
        // Get all keys within the range
        let request: IdbRequest = store.get_all_keys_with_key(&key_range.into())
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let promise = Promise::new(&mut |resolve, _reject| {
            let request_success = request.clone();
            let on_success = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                resolve.call1(&JsValue::undefined(), &request_success.result().unwrap()).unwrap();
            });

            let request_error = request.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let error = request_error.error().unwrap();
                _reject.call1(&JsValue::undefined(), &JsValue::from(error)).unwrap();
            });
            
            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            
            on_success.forget();
            on_error.forget();
        });
            
        let keys_result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| JsError::from(DomException::new().unwrap()))?;
            
        let keys_array = js_sys::Array::from(&keys_result);
        
        // Combine keys and values
        for i in 0..array.length() {
            let key = keys_array.get(i);
            let value = array.get(i);
            
            if !key.is_undefined() && !value.is_undefined() {
                let key_array = Uint8Array::new(&key);
                let value_array = Uint8Array::new(&value);
                let mut key_vec = vec![0; key_array.length() as usize];
                let mut value_vec = vec![0; value_array.length() as usize];
                key_array.copy_to(&mut key_vec);
                value_array.copy_to(&mut value_vec);
                items.push((key_vec, value_vec));
            }
        }
        
        Ok(items)
    }
}
