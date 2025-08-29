pub mod store;
pub mod types;

#[macro_export]
/// The macro handles the command result from tauri backend and parse the result in the desired struct.
///
/// If an error happens, a toast will be automatically issued to the UI and the scope where this macro was executed will be
/// halted (aka this macro return void)
///
/// When calling this macro, your file must have "serde_wasm_bindgen", "Alert", AlertLevel" and "push_toast" included
///
/// For exemple:
///
/// let webtoons: Vec<WebtoonSearchInfo> = parse_or_toast!(invoke("search_webtoon", args).await, Ty = Vec<WebtoonSearchInfo>, push_toast);
macro_rules! parse_or_toast {
    // $expr is the future/await expression returning Result<JsValue, JsValue-like>
    // Ty = $ty: the type to deserialize into
    // $push_toast: ident referring to push_toast in scope (it must implement .run(Alert))
    ($expr:expr, Ty = $ty:ty, $push_toast:ident) => {
        match $expr
            .map(|v| {
                serde_wasm_bindgen::from_value::<$ty>(v)
                    .map_err(|_| "Failed to parse data as the right struct".to_string())
            })
            .map_err(|e| {
                e.as_string().unwrap_or(
                    "An error happened, but we can't provide more information".to_string(),
                )
            }) {
            Ok(Ok(wt)) => wt,
            Ok(Err(e)) | Err(e) => return $push_toast.run(Alert::new(e, AlertLevel::Error, None)),
        }
    };
}

const IS_ANDROID: bool = true;
/*
#[cfg(any(windows, target_os = "android"))]
let base = ;
#[cfg(not(any(windows, target_os = "android")))]
let base = ;
*/

pub fn convert_file_src(file_path: &str) -> String {
    let base = match IS_ANDROID {
        true => "http://asset.localhost/",
        false => "asset://localhost/",
    };

    let urlencoded_path = urlencoding::encode(file_path);
    format!("{base}{urlencoded_path}")
}
