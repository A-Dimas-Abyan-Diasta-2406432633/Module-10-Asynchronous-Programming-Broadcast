use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    current_user: String,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            current_user: username,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen h-screen bg-gradient-to-br from-slate-900 via-cyan-900 to-emerald-900">
                <div class="flex-none w-72 h-screen bg-slate-950/85 text-white border-r border-cyan-500/30">
                    <div class="px-4 py-5 border-b border-cyan-400/20">
                        <div class="text-lg font-bold">{"Online Users"}</div>
                        <div class="text-xs text-cyan-200">
                            {format!("{} user aktif", self.users.len())}
                        </div>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-slate-800/80 rounded-xl p-2 border border-slate-700">
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-sm font-semibold justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-cyan-200">
                                            {"Ready to chat"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col bg-white/95">
                    <div class="w-full h-16 border-b border-slate-200 px-6 flex items-center justify-between">
                        <div>
                            <div class="text-xl font-bold text-slate-800">{"Rust Yew Lounge"}</div>
                            <div class="text-xs text-slate-500">{format!("Login sebagai {}", self.current_user)}</div>
                        </div>
                        <div class="text-xs px-3 py-1 rounded-full bg-cyan-100 text-cyan-700 border border-cyan-200">
                            {"Realtime WebSocket"}
                        </div>
                    </div>
                    <div class="w-full grow overflow-auto border-b border-slate-200 py-4">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let avatar = user
                                    .map(|u| u.avatar.clone())
                                    .unwrap_or_else(|| format!("https://avatars.dicebear.com/api/adventurer-neutral/{}.svg", m.from));
                                let is_me = m.from == self.current_user;
                                let row_class = if is_me {
                                    "flex justify-end w-full px-6 my-3"
                                } else {
                                    "flex justify-start w-full px-6 my-3"
                                };
                                let bubble_class = if is_me {
                                    "max-w-xl p-4 rounded-2xl rounded-br-md bg-cyan-600 text-white shadow-md"
                                } else {
                                    "max-w-xl p-4 rounded-2xl rounded-bl-md bg-slate-100 text-slate-800 shadow-sm border border-slate-200"
                                };
                                html!{
                                    <div class={row_class}>
                                        if !is_me {
                                            <img class="w-9 h-9 rounded-full mr-3 self-end" src={avatar.clone()} alt="avatar"/>
                                        }
                                        <div class={bubble_class}>
                                            <div class="text-xs font-semibold mb-1 opacity-90">
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-sm leading-relaxed">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded-lg" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                        if is_me {
                                            <img class="w-9 h-9 rounded-full ml-3 self-end" src={avatar.clone()} alt="avatar"/>
                                        }
                                    </div>
                                }
                            }).collect::<Html>()
                        }

                    </div>
                    <div class="w-full h-16 flex px-4 items-center bg-white">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Ketik pesan..."
                            class="block w-full py-2 pl-4 mx-2 bg-slate-100 rounded-full outline-none focus:ring-2 focus:ring-cyan-500"
                            name="message"
                            required=true
                        />
                        <button onclick={submit} class="p-3 shadow-sm bg-cyan-600 w-11 h-11 rounded-full flex justify-center items-center color-white">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
