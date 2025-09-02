// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fosstoon_lib::run();
}

/* App dataflow design (-> = request)
- UI -> get_user_data at start
- UI -> get_webtoon_info: when going on a webtoon page, this command fetch if not existing or update if expired the webtoon and cache it in the app store
- UI -> subscribe_to_webtoon: this only mark into the user database the webtoon to sub, it does not fetch or update the webtoon data. If when requesting to sub webtoon data is not existing -> error
- UI -> force_refresh_episodes: when finished it also update the webtoon info in the app store
*/
