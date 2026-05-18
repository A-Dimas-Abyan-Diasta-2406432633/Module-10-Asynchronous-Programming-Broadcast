use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
       <div class="w-screen h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-cyan-900 to-emerald-800">
            <div class="w-full max-w-xl mx-4 bg-white/90 backdrop-blur rounded-2xl shadow-2xl border border-white/40">
                <div class="p-8">
                    <div class="text-3xl font-extrabold text-slate-800">{"Rust Yew Lounge"}</div>
                    <div class="mt-2 text-sm text-slate-600">{"Masuk dulu, lalu ngobrol realtime dengan websocket."}</div>
                    <form class="mt-6 flex gap-2">
                        <input
                            {oninput}
                            class="flex-1 rounded-xl p-3 border border-slate-300 text-slate-800 bg-white focus:outline-none focus:ring-2 focus:ring-cyan-500"
                            placeholder="Masukkan username"
                        />
                        <Link<Route> to={Route::Chat}>
                            <button
                                {onclick}
                                disabled={username.len() < 1}
                                class="px-6 rounded-xl bg-cyan-600 text-white font-bold border border-cyan-700 disabled:opacity-40 disabled:cursor-not-allowed"
                            >
                                {"Masuk Chat"}
                            </button>
                        </Link<Route>>
                    </form>
                </div>
            </div>
        </div>
    }
}
