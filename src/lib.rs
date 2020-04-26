#![recursion_limit = "256"]

use anyhow::Error;
use serde_derive::{Deserialize};
use yew::format::{Json, Nothing};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};
use yew::services::ConsoleService;

pub enum Msg {
    FetchData,
    FetchReady(Result<DataFromFile, Error>),
    Ignore,
    UpdateSearchText(String)
}

#[derive(Deserialize, Debug)]
pub struct ResultWikipedia {
    ns: u32,
    title: String,
    pageid: u32,
    size: u32,
    wordcount: u32,
    snippet: String,
    timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct SearchInfo {
    totalhits: u32,
}

#[derive(Deserialize, Debug)]
pub struct QueryWikipedia {
    searchinfo: SearchInfo,
    search: Vec<ResultWikipedia>,
}

#[derive(Deserialize, Debug)]
pub struct ContinueWikipedia {
    sroffset: u32,
    r#continue: String,
}

#[derive(Deserialize, Debug)]
pub struct DataFromFile {
    batchcomplete: String,
    r#continue: ContinueWikipedia,
    query: QueryWikipedia,
}

pub struct Model {
    fetch_service: FetchService,
    console: ConsoleService,
    link: ComponentLink<Model>,
    fetching: bool,
    text_search: String,
    data: Option<Vec<ResultWikipedia>>,
    ft: Option<FetchTask>,
}


impl Model {
    fn view_data(&self) -> Html {    
        if let Some(search) = &self.data {
            html! {
                <div>
                    { search.iter().map(|result| self.view_result(result)).collect::<Html>() }
                </div>
            }
        } else {
            html! {
                <p></p>
            }
        }
    }

    fn view_result(&self, result: &ResultWikipedia) -> Html {    
        html! {
            <div>
                <h2>{ &result.title }</h2>
                <div>{ &result.snippet }</div>
            </div>
        }
    }

    fn fetch_json(&mut self) -> yew::services::fetch::FetchTask {
        let url = ["https://en.wikipedia.org/w/api.php?action=query&list=search&utf8=&format=json&srsearch=", self.text_search.as_str()].concat();
        let callback = self.link.callback(
            move |response: Response<Json<Result<DataFromFile, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                println!("META: {:?}, {:?}", meta, data);
                if meta.status.is_success() {
                    Msg::FetchReady(data)
                } else {
                    Msg::Ignore
                }
            },
        );
        let request = Request::builder()
            .method("GET")
            .uri(url.as_str())
            .header("Access-Control-Allow-Origin", "http://[::1]:8000/")
            .body(Nothing)
            .unwrap();
        self.fetch_service.fetch_binary(request, callback).unwrap()
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            fetch_service: FetchService::new(),
            console: ConsoleService::new(),
            link,
            text_search: "".to_string(),
            fetching: false,
            data: None,
            ft: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchData => {
                self.fetching = true;
                let task = self.fetch_json();
                self.ft = Some(task);
            }
            Msg::FetchReady(response) => {
                if response.is_ok() {
                    self.console.log("ok");
                } else {
                    self.console.log("ko");
                }
                self.fetching = false;
                self.data = response.map(|data| data.query.search).ok();
            }
            Msg::Ignore => {
                return false;
            }
            Msg::UpdateSearchText(val) => {
                self.text_search = val;
            }
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input
                   placeholder="Enter your search"
                   value=&self.text_search
                   oninput=self.link.callback(|e: InputData| Msg::UpdateSearchText(e.value)) />
                    <button onclick=self.link.callback(|_| Msg::FetchData)>
                        { "Search Data" }
                    </button>
                    { self.view_data() }
            </div>
        }
    }
}
