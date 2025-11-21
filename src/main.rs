use gtk4::prelude::*;
use gtk4::{Window, Box, Application, ApplicationWindow, Button, Entry, Grid, TextView, TreeView, TreeStore, ScrolledWindow, TextBuffer, Label, TreeViewColumn, CellRendererText, Orientation, MessageDialog, DialogFlags, MessageType, ButtonsType};
use std::process::exit;
use std::rc::Rc;
use std::cell::RefCell;
use regex::Regex;
use serde_json::Value;
use tokio;
use std::process::Command;
use std::str;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::thread;
use std::time::Duration;
use glib::source::timeout_add_seconds_local;
use glib::ControlFlow;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use glib::MainContext;
use serde::Deserialize;

use chrono::{TimeZone, Utc};
use serde::{Serialize};
use serde_yaml;
use thiserror::Error;

use gtk4::CssProvider;

use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    use_awk: Option<String>,
    premium: Option<String>,
    theme: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            use_awk: Some("no".to_string()),
            premium: Some("no".to_string()),
            theme: Some("no".to_string()),
        }
    }
}

#[derive(Debug, Error)]
enum ConfigError {
    #[error("failed to read config file")]
    ReadError(#[from] std::io::Error),
    #[error("failed to parse YAML")]
    ParseError(#[from] serde_yaml::Error),
}

// Путь к корневому каталогу
const ROOT_DIR: &str = "/opt/aptos_clockwork";

fn config_load() -> Result<Config, ConfigError> {
    let config_path = format!("{}/aptos_clockwork.yaml", ROOT_DIR);
//    let config_path = "aptos_clockwork.yaml";

//    let config = if Path::new(config_path).exists() {
    let config = if Path::new(&config_path).exists() {
        println!("Конфиг найден: {}", config_path);
        let config_data = fs::read_to_string(config_path)?;
        let mut loaded_config: Config = serde_yaml::from_str(&config_data)?;

        if loaded_config.use_awk.is_none() {
            loaded_config.use_awk = Some("no".to_string());
        }
        if loaded_config.premium.is_none() {
            loaded_config.premium = Some("no".to_string());
        }
        if loaded_config.theme.is_none() {
            loaded_config.theme = Some("no".to_string());
        }

        loaded_config
    } else {
//        // Использует дефолтные значения
//        Config::default()
        let default_config = Config::default();
        println!("{} не найден. Создается новый config...", config_path);
        let config_data = serde_yaml::to_string(&default_config)?;
        fs::write(config_path, config_data)?; // Записать дефолтный конфиг в файл
        default_config
    };

    Ok(config)
}

fn check_and_update_premium(config: &mut Config) {
    let default_premium_value = "jFg{s;QdsF#45#)&@e22./#d";

    if let Some(ref premium) = config.premium {
        if premium != default_premium_value {
            println!("Значение 'premium' в конфиге отличается от стандартного. Обновляем...");
            config.premium = Some(default_premium_value.to_string());
        }
    } else {
        // Если premium отсутствует, то устанавливаем значение по умолчанию
        println!("Значение 'premium' отсутствует. Устанавливаем стандартное...");
        config.premium = Some(default_premium_value.to_string());
    }

    // Сохраняем измененный конфиг
    let config_path = "aptos_clockwork.yaml";
    let config_data = serde_yaml::to_string(config).unwrap(); // Обработка ошибки в unwrap
    fs::write(config_path, config_data).unwrap(); // Обработка ошибки в unwrap
}

// Единственный путь к aptos-cli
const APTOS_CLI_PATH: &str = "/opt/aptos-core";
// Проверка наличия aptos-cli
fn check_aptos_cli() -> bool {
    if !Path::new(APTOS_CLI_PATH).exists() {
        println!("Приложение aptos-core не найдено. Чтобы работал кошелек, нужно подключиться к ноде. Каталог для aptos-core читается по пути {}", APTOS_CLI_PATH);
        return false;
    } else {
        println!("aptos-core найден: {}", APTOS_CLI_PATH);
    }
    true
}

#[tokio::main]
async fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let premium_flag = Rc::new(RefCell::new(false));
    let theme_flag = Rc::new(RefCell::new(0));
    match config_load() {
        Ok(config) => {
	    if config.premium.as_deref() == Some("jFg{s;QdsF#45#)&@e22./#d") {
                *premium_flag.borrow_mut() = true;
//	        println!("premium yes: {}", config.premium.as_ref().unwrap());
	        if config.theme.as_deref() == Some("1") {
//	            println!("theme 1");
                    *theme_flag.borrow_mut() = 1;
                } else if config.theme.as_deref() == Some("2") {
                    *theme_flag.borrow_mut() = 2;
	        } else {
	            println!("theme other");
	        }
	    } else {
	        println!("premium no");
                *premium_flag.borrow_mut() = false;
	    }
        },
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            *premium_flag.borrow_mut() = false;
        },
    }


//    if *premium_flag.borrow() {
//        println!("Флаг premium установлен в true");
//    } else {
//        println!("Флаг premium установлен в false");
//    }
//    let theme_value = *theme_flag.borrow();
//    println!("theme flag value: {}", theme_value);

//    // Если конфиг не содержит поля premium с нужным значением, ничего не делаем
//    if config.premium.as_deref() == Some("jFg{s;QdsF#45#)&@e22./#d") {
////        println!("premium yes: {}", config.premium.unwrap());
//        println!("premium yes: {}", config.premium.as_ref().unwrap());
//        // Если поле `theme` равно "1", выводим "theme 1"
//        if config.theme.as_deref() == Some("1") {
//            println!("theme 1");
////            if config.theme.as_deref() == Some("1") {
////                // Загрузка изображения с интернета
////                let image_url = "http://ava7patterns.com/patterns/1417.png";
////                let response = get(image_url)?;
////                let bytes = response.bytes()?;
////        
////                // Сохранение изображения во временный файл
////                let tmp_file_path = "/tmp/background.png";
////                let mut file = File::create(tmp_file_path)?;
////                file.write_all(&bytes)?;
////        
////                // Инициализация GTK
////                gtk4::init()?;
////        
////                // Создание окна
////                let window = gtk4::Window::new(gtk4::WindowType::Toplevel);
////                window.set_title("My App with Custom Background");
////        
////                // Загрузка изображения как фон
////                let pixbuf = Pixbuf::new_from_file_at_scale(tmp_file_path, 800, 600, true)?;
////                let pattern = Pixbuf::get_from_file(tmp_file_path)?;
////        
////                // Настройка фона с использованием изображения
////                let background = gtk4::Image::new_from_pixbuf(Some(&pixbuf));
////                window.set_child(Some(&background));
////        
////                // Отображаем окно
////                window.show_all();
////                gtk4::main();
////            }
//        } else {
//            println!("theme other");
//        }
//    } else {
//        println!("premium no");
//    }
//
//    // Записываем конфигурацию обратно в файл, если были изменения
//    if !Path::new(config_path).exists() {
//        let file = File::create(config_path)?;
//        serde_yaml::to_writer(file, &config)?;
//        println!("Конфигурация создана с дефолтными значениями.");
//    }

    let app = Application::new(Some("com.aptos.clockwork"), Default::default());
    app.connect_activate(move |app| {
        let premium_flag = Rc::clone(&premium_flag);
        let theme_value = *theme_flag.borrow();

        // Создаем главное окно
        let window = ApplicationWindow::new(app);
        window.set_title(Some("Aptos Clockwork"));
        window.set_default_size(765, 406);

//        if *premium_flag.borrow() {
//            println!("Флаг premium внутри приложения: true");
//        } else {
//            println!("Флаг premium внутри приложения: false");
//        }
        println!("Start, Current theme flag value inside activate: {}", theme_value);
        // Проверяем значение theme_value
        if theme_value == 1 {
            // Создаем CSS для окна
            let css_provider = CssProvider::new();

            // Загружаем данные CSS как строку
            css_provider.load_from_data("window { background-color: black; }");

            // Применяем CSS к окну
            let style_context = window.style_context();
            style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        } else if theme_value == 2 {
            let root_path = Path::new(ROOT_DIR);
            println!("Корневой каталог: {:?}", root_path);
//            let exe_path = std::env::current_exe().unwrap();
//            println!("exe_path: {:?}", exe_path);
//            let program_dir = exe_path.parent().unwrap();
//            println!("program_dir: {:?}", program_dir);
            // Формируем путь к изображению
            let background_path = root_path.join("background.png");
            println!("background_path: {:?}", background_path);
            // Создаем CSS для фонового изображения
            let css_provider = CssProvider::new();
//            let style_context = window.style_context();
//            style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            let css = format!(
                "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                background_path.to_str().unwrap()
            );
            css_provider.load_from_data(&css);
            let style_context = window.style_context();
            style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        } 
     
        // Создаем сетку для размещения виджетов
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);

        // Поле ввода
        let input = Rc::new(RefCell::new(Entry::new()));
        input.borrow().set_placeholder_text(Some("Введите адрес..."));
        grid.attach(&*input.borrow(), 0, 0, 4, 1);
                                                   
        // Создаем таблицу для Coin с прокруткой
        let coin_scroll = Rc::new(RefCell::new(ScrolledWindow::new()));
        coin_scroll.borrow().set_min_content_height(300);
        let coin_table = create_coin_table();
        coin_scroll.borrow().set_child(Some(&coin_table));
        grid.attach(&*coin_scroll.borrow(), 0, 3, 4, 1);
//        coin_scroll.borrow().set_visible(false);

        // Создаем таблицу для NFT с прокруткой
        let nft_scroll = Rc::new(RefCell::new(ScrolledWindow::new()));
        nft_scroll.borrow().set_min_content_height(300);
        let nft_table = create_nft_table();
        nft_scroll.borrow().set_child(Some(&nft_table));
        grid.attach(&*nft_scroll.borrow(), 0, 4, 4, 1);
        nft_scroll.borrow().set_visible(false); // По умолчанию скрыта

        // Создаем метки
        let label1 = Rc::new(RefCell::new(Label::new(Some("Lines: 0"))));
//        let label2 = Label::new(Some("Sequence Number: N/A"));
        let label2 = Rc::new(RefCell::new(Label::new(Some("Sequence Number: N/A"))));
//        let label3 = Label::new(Some("Fundraising: N/A"));
        let label3 = Rc::new(RefCell::new(Label::new(Some("Fundraising: N/A"))));
        label1.borrow_mut().set_margin_top(5);
        label1.borrow_mut().set_margin_bottom(15);
        label2.borrow_mut().set_margin_top(5);
        label2.borrow_mut().set_margin_bottom(15);
//        label2.set_margin_top(5);
//        label2.set_margin_bottom(15);
//        label3.set_margin_top(5);
//        label3.set_margin_bottom(15);
        label3.borrow_mut().set_margin_top(5);
        label3.borrow_mut().set_margin_bottom(15);
        // Устанавливаем свойство "расширения" для меток, чтобы они заполняли пространство
        label1.borrow_mut().set_hexpand(true);
//        label2.set_hexpand(true);
        label2.borrow_mut().set_hexpand(true);
//        label3.set_hexpand(true);
        label3.borrow_mut().set_hexpand(true);
        // Добавляем метки в Grid
        grid.attach(&*label1.borrow(), 0, 5, 1, 1);
//        grid.attach(&label2, 1, 5, 2, 1);
        grid.attach(&*label2.borrow(), 1, 5, 2, 1);
//      grid.attach(&label3, 3, 5, 1, 1);
        grid.attach(&*label3.borrow(), 3, 5, 1, 1);

        // Кнопка Coin
        let input_clone = Rc::clone(&input); // Клонируем для использования в замыкании
        let coin_scroll_clone = Rc::clone(&coin_scroll);  // Клонируем coin_scroll для использования в замыкании
        let nft_scroll_clone = Rc::clone(&nft_scroll);
        let coin_button = Button::with_label("Coin");

        let label1_clone = Rc::clone(&label1);
        let label2_clone = Rc::clone(&label2);
        coin_button.connect_clicked(move |_| {
            let label1_clone_async = Rc::clone(&label1_clone);
            let label2_clone_async = Rc::clone(&label2_clone);
            coin_scroll_clone.borrow().set_visible(true);  // Показываем таблицу для Coin
            nft_scroll_clone.borrow().set_visible(false);  // Скрываем таблицу для NFT
            // Извлекаем текст из поля ввода
            let entered_text = input_clone.borrow().text().to_string();
            // Проверка на пустую строку
            if entered_text.trim().is_empty() {
                // Выводим предупреждение
                let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Поле ввода пустое. Пожалуйста, введите адрес.");
                dialog.connect_response(|dialog, _| dialog.close());
                dialog.show();
                // Завершаем выполнение функции
                return;
            }
            // Проверка формата адреса
	    let re = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();
	    if !re.is_match(&entered_text) {
	        let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Неверный формат адреса!");
	        dialog.connect_response(|dialog, _| dialog.close());
	        dialog.show();
	        return;
	    }

//            println!("Введенный адрес: {}", entered_text);

	    // Формируем команду curl с заголовками и телом запроса
	    let output = Command::new("curl")
	        .arg("-sS")
	        .arg("https://api.mainnet.aptoslabs.com/v1/graphql")
	        .arg("-H").arg("accept: */*")
	        .arg("-H").arg("accept-language: ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7")
	        .arg("-H").arg("content-type: application/json")
	        .arg("-H").arg("dnt: 1")
	        .arg("-H").arg("origin: https://explorer.aptoslabs.com")
	        .arg("-H").arg("priority: u=1, i")
	        .arg("-H").arg("referer: https://explorer.aptoslabs.com/")
	        .arg("-H").arg("sec-ch-ua: \"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Google Chrome\";v=\"126\"")
	        .arg("-H").arg("sec-ch-ua-mobile: ?0")
	        .arg("-H").arg("sec-ch-ua-platform: \"Windows\"")
	        .arg("-H").arg("sec-fetch-dest: empty")
	        .arg("-H").arg("sec-fetch-mode: cors")
	        .arg("-H").arg("sec-fetch-site: same-site")
	        .arg("-H").arg("user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
	        .arg("--data-raw")
	        .arg(format!(
	            "{{\"operationName\":\"CoinsData\",\"variables\":{{\"owner_address\":\"{}\"}},\"query\":\"query CoinsData($owner_address: String, $limit: Int, $offset: Int) {{ current_fungible_asset_balances(where: {{owner_address: {{_eq: $owner_address}}}}, limit: $limit, offset: $offset) {{ amount asset_type metadata {{ name decimals symbol token_standard }} }} }}\"}}",
	            entered_text
	        ))
	        .output()
	        .expect("Не удалось выполнить команду curl");
	
	    // Преобразуем результат команды curl в строку
	    let result = str::from_utf8(&output.stdout).unwrap();

	    // Преобразуем результат в JSON
	    let json_data: Value = serde_json::from_str(result).unwrap();
	
	    // Выводим JSON в читабельном виде
//	    println!("json_data:\n{:#?}", json_data);

            println!("Кол-во строк до функции: {:?}", label1_clone_async.borrow().text());
            // Передаем переменную в функцию
            update_coin_table(&coin_table, &json_data, label1_clone_async);

            // Используем асинхронную задачу для выполнения HTTP-запроса
	    let future = async move {
	        let url = format!("https://fullnode.mainnet.aptoslabs.com/v1/accounts/{}", entered_text);
	        
	        // Выполняем запрос
	        match reqwest::get(&url).await {
	            Ok(response) => {
	                match response.json::<Value>().await {
	                    Ok(json_data) => {
	                        if let Some(sequence_number) = json_data.get("sequence_number") {
	                            // Обновляем метку с новым значением sequence_number
//	                            let sequence_text = format!("Sequence Number: {}", sequence_number);
//	                            label2_clone_async.borrow_mut().set_text(&sequence_text);
                                    let sequence_text = match sequence_number.as_str() {
                                        Some(num_str) => format!("Sequence Number: {}", num_str),
                                        None => "Sequence Number: N/A".to_string(),
                                    };
                                    label2_clone_async.borrow_mut().set_text(&sequence_text);
	                        } else {
	                            // Если sequence_number не найден
	                            label2_clone_async.borrow_mut().set_text("Sequence Number: N/A");
	                        }
	                    },
	                    Err(_) => {
	                        label2_clone_async.borrow_mut().set_text("Error parsing JSON");
	                    },
	                }
	            },
	            Err(_) => {
	                label2_clone_async.borrow_mut().set_text("Error fetching data");
	            },
	        }
	    };

            // Запускаем асинхронную задачу
            glib::MainContext::default().spawn_local(future);
		
        });
        grid.attach(&coin_button, 0, 1, 1, 1);

        // Кнопка NFT
	let nft_button = Button::with_label("NFT");
	let nft_scroll_clone = Rc::clone(&nft_scroll);
	let coin_scroll_clone = Rc::clone(&coin_scroll);
	let input_clone_nft = Rc::clone(&input); // Клонируем для использования в замыкании
              
        let label1_clone = Rc::clone(&label1);
        let label2_clone = Rc::clone(&label2);
        nft_button.connect_clicked(move |_| {
            let label1_clone_async = Rc::clone(&label1_clone);
//            let label2_clone_async = Rc::clone(&label2_clone);
            let input_clone_nft = Rc::clone(&input_clone_nft);
            let nft_scroll_clone = Rc::clone(&nft_scroll_clone);
            let coin_scroll_clone = Rc::clone(&coin_scroll_clone);

            // Очистить таблицу до вызова.
            clear_nft_table(&nft_scroll.borrow());

            // Обновляем текст метки перед обновлением таблицы
            label1_clone.borrow_mut().set_text("Loading..."); // Удалили фигурные скобки

            label2_clone.borrow_mut().set_text("Sequence Number: N/A");

            glib::MainContext::default().spawn_local(async move {
                // Извлекаем текст из поля ввода
                let entered_text = input_clone_nft.borrow().text().to_string();

                // Проверка на пустую строку
                if entered_text.trim().is_empty() {
                    // Выводим предупреждение
                    let dialog = MessageDialog::new(
                        None::<&Window>,
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        "Поле ввода пустое. Пожалуйста, введите адрес.",
                    );
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.show();
                    return;
                }

                // Проверка формата адреса
                let re = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();
                if !re.is_match(&entered_text) {
                    let dialog = MessageDialog::new(
                        None::<&Window>,
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        "Неверный формат адреса!",
                    );
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.show();
                    return;
                }

                // Показываем таблицу для NFT и скрываем таблицу для Coin
                nft_scroll_clone.borrow().set_visible(true);
                coin_scroll_clone.borrow().set_visible(false);

                // Получаем значения переменных окружения
	        let api_key = env::var("API_KEY").unwrap_or_else(|_| "L5ZgiSi.aa37c3098a82bdebc3c9d733cb575836".to_string());
		let api_user = env::var("API_USER").unwrap_or_else(|_| "oweeea".to_string());
		let limit = 25;
		let mut offset = 0;
		
		// Создаем HTTP-клиент
		let client = Client::new();
		
		// Переменная для проверки наличия коллекции SCAM
//		let mut has_qribbles = false;

                let mut nft_info: Vec<(String, String)> = Vec::new();
		
		loop {
		    // Формируем JSON-запрос
		    let query = json!({
		        "query": "query fetchWalletInventoryWithListings( $where: nfts_bool_exp, $order_by: [nfts_order_by!] $offset: Int $limit: Int! ) { aptos { nfts(where: $where, order_by: $order_by, offset: $offset, limit: $limit) { id token_id token_id_index name media_url media_type ranking owner delegated_owner burned staked version chain_state claimable claimable_by claimable_reason claimable_contract_key collection { id slug semantic_slug title supply verified floor } listings(where: { listed: { _eq: true } }, order_by: { price: asc }) { id price price_str block_time seller market_name nonce contract { key } } topBid: bids( where: { status: { _eq: \"active\" } } order_by: { price: desc } limit: 1 ) { id bidder price } lastSale: actions( where: { type: { _in: [\"buy\", \"accept-collection-bid\", \"accept-bid\"] } } order_by: { block_time: desc } limit: 1 ) { price } contract { commission: default_commission { key market_fee market_name royalty is_custodial } } } } }",
		        "variables": {
		            "where": {
		                "_or": [
		                    { "owner": { "_eq": entered_text }, "listed": { "_eq": true } },
		                    { "owner": { "_eq": entered_text } },
		                    { "claimable_by": { "_eq": entered_text } }
		                ]
		            },
		            "order_by": [
		                { "collection": { "title": "asc" } },
		                { "ranking": "asc_nulls_last" },
		                { "token_id_index": "asc_nulls_last" }
		            ],
		            "limit": limit,
		            "offset": offset
		        }
		    });

                    // Выполняем HTTP-запрос
                    let response = match client
			    .post("https://api.indexer.xyz/graphql")
			    .header("x-api-key", &api_key)
			    .header("x-api-user", &api_user)
			    .header("Content-Type", "application/json")
			    .json(&query)
			    .send()
			    .await {
			        Ok(resp) => resp, // Если запрос успешен, сохраняем ответ
			        Err(err) => {
			            eprintln!("Ошибка при выполнении запроса: {}", err);
			            return; // Выход из блока в случае ошибки
			        }
			    };

                    // Получаем ответ в виде JSON
		    let response_json: serde_json::Value = match response.json().await {
		        Ok(json) => {
//		            println!("Response JSON: {:?}", json); // Выводим JSON-ответ
		            json // Если обработка успешна, сохраняем JSON
		        }
		        Err(err) => {
		            eprintln!("Ошибка при обработке ответа: {}", err);
		            return; // Выход из блока в случае ошибки
		        }
		    };

                    // Обрабатываем каждый элемент из массива NFTs
		    if let Some(nfts) = response_json["data"]["aptos"]["nfts"].as_array() {
		        for nft in nfts {
		            // Получаем значения полей "name" и "title"
		            let name = nft["name"].as_str().unwrap_or("Unknown name").to_string();
		            let title = nft["collection"]["title"].as_str().unwrap_or("Unknown title").to_string();
		            nft_info.push((name, title)); // Сохраняем вектор кортежей
		        }
		
		        // Если количество NFT меньше лимита, прекращаем цикл
		        if nfts.len() < limit {
		            break; // Прекращаем цикл
		        }
		
		        // Увеличиваем OFFSET и продолжаем цикл
		        offset += limit;
		        println!("Ждём...");
		        thread::sleep(Duration::from_secs(3));
		    } else {
		        eprintln!("NFTs не найдены или неверный формат ответа");
		        break;
		    }
	        }

                // Создаем и выводим nft_lines
	        let mut nft_lines = Vec::new();
	
	        for (name, title) in nft_info {
	            let formatted_line = format!("Name: {}, Collection: {}", name, title);
	            nft_lines.push(formatted_line.clone());
	        }

                let re = Regex::new(r"Name:\s*([^,]+),\s*Collection:\s*(.*)").unwrap();

                // Преобразуем данные
		let nft_data: Vec<(String, String)> = nft_lines.iter()
		    .filter_map(|entry| {
		        // Ищем совпадение по регулярному выражению
		        re.captures(entry).map(|cap| {
		            // Получаем значения "Name" и "Collection"
		            let name = cap[1].trim().to_string();
		            let collection = cap[2].trim().to_string();
		            (name, collection)
		        })
		    })
		    .collect();
		
                println!("Кол-во строк до функции: {:?}", label1_clone_async.borrow().text());

                update_nft_table(label1_clone_async, &nft_scroll_clone.borrow(), nft_data.clone());

            });
        });
        grid.attach(&nft_button, 1, 1, 1, 1);

        // Кнопка Clear
        let clear_button = Button::with_label("Clear");
        let input_clone = Rc::clone(&input);
        clear_button.connect_clicked(move |_| {
            input_clone.borrow().set_text("");
        });
        grid.attach(&clear_button, 2, 1, 1, 1);

        // Кнопка Close
        let close_button = Button::with_label("Close");
        close_button.connect_clicked(move |_| {
            exit(0);
        });
        grid.attach(&close_button, 3, 1, 1, 1);

        // Кнопка Wallet
        let wallet_button = Button::with_label("Wallet");
//        let label3_clone = label3.clone();

        let label3_clone = Rc::clone(&label3);

        wallet_button.connect_clicked(move |_| {
//            let path_to_node = Path::new("/opt/aptos-core");
            let path_to_node = Path::new(APTOS_CLI_PATH);
	    // Проверка существования директории
	    if !path_to_node.exists() {
		println!("Ошибка: Репозитория {APTOS_CLI_PATH} не существует. Для работы кошелька нужно подключиться к ноде.");
//	    } else {
//		println!("[ok] /opt/aptos-core найден");
            }
            // Создаем новое окно
            let wallet_window = Rc::new(Window::new());
            wallet_window.set_title(Some("Wallet"));
//            wallet_window.set_default_size(200, 200);
            // Создаем сетку для содержимого всплывающего окна
            let popup_grid = Grid::new();

// Ставим тему к Wallet-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
            println!("Wallet: Current theme flag value inside activate: {}", theme_value);
            // Проверяем значение theme_value
            if theme_value == 1 {
                // Создаем CSS для окна
                let css_provider = CssProvider::new();

                // Загружаем данные CSS как строку
                css_provider.load_from_data("window { background-color: black; }");

                // Применяем CSS к окну
                let style_context = wallet_window.style_context();
                style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            } else if theme_value == 2 {
                let root_path = Path::new(ROOT_DIR);
//                println!("Корневой каталог: {:?}", root_path);
                // Формируем путь к изображению
                let background_path = root_path.join("background.png");
                // Создаем CSS для фонового изображения
                let css_provider = CssProvider::new();
                let css = format!(
                    "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                    background_path.to_str().unwrap()
                );
                css_provider.load_from_data(&css);
                let style_context = wallet_window.style_context();
                style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            } 
        
            popup_grid.set_column_spacing(10);
            popup_grid.set_row_spacing(10);

            // Добавляем текстовую метку
            let text_label = Label::new(Some("N/A"));
            popup_grid.attach(&text_label, 0, 0, 2, 1);

            // Создаем кнопку "Close" для закрытия окна
            let close_button = Button::with_label("Close");
            let wallet_window_rc = Rc::new(wallet_window);

            popup_grid.set_margin_bottom(10);
            popup_grid.set_margin_top(10);
            popup_grid.set_margin_start(10);
            popup_grid.set_margin_end(10);

            // Используем Rc для управления временем жизни wallet_window
            let wallet_window_clone = Rc::clone(&wallet_window_rc);
            close_button.connect_clicked(move |_| {
                wallet_window_clone.close(); // Закрытие окна
            });
            popup_grid.attach(&close_button, 1, 3, 1, 1); // Кнопка Close в позиции (0, 3)

            let path_to_yaml = Path::new("/opt/aptos-core/.aptos/config.yaml");
	    // Проверка существования директории
	    if !path_to_yaml.exists() {
                // Кнопка Import
                let import_button = Button::with_label("Import");
                let wallet_window_rc_clone3 = Rc::clone(&wallet_window_rc);
                import_button.connect_clicked(move |_| {
//                    import(Rc::clone(&wallet_window_rc_clone3));
                    import(Rc::clone(&wallet_window_rc_clone3), theme_value);
                });
                popup_grid.attach(&import_button, 0, 1, 1, 1);

                // Кнопка Create
                let create_button = Button::with_label("Create");
                let wallet_window_rc_clone2 = Rc::clone(&wallet_window_rc);
                create_button.connect_clicked(move |_| {
                    createwallet(Rc::clone(&wallet_window_rc_clone2));
                });
                popup_grid.attach(&create_button, 1, 1, 1, 1);
	    } else {
//                println!("[ok]: yaml существует.");
                // Кнопка Export
                let export_button = Button::with_label("Export");
                export_button.connect_clicked(move |_| {
//                    export();
//                    MainContext::default().spawn_local(export());
                    MainContext::default().spawn_local(export(theme_value));
                });
                popup_grid.attach(&export_button, 0, 1, 1, 1);

                // Кнопка Remove
                let remove_button = Button::with_label("Remove");
                let wallet_window_rc_clone = Rc::clone(&wallet_window_rc);
                remove_button.connect_clicked(move |_| {
                    remove(Rc::clone(&wallet_window_rc_clone));
                });
                popup_grid.attach(&remove_button, 1, 1, 1, 1);

                // Кнопка Send
                let send_button = Button::with_label("Send");
                let wallet_window_rc_clone4 = Rc::clone(&wallet_window_rc);
                send_button.connect_clicked(move |_| {
//                    send();
//                    MainContext::default().spawn_local(send(Rc::clone(&wallet_window_rc_clone4)));
                    MainContext::default().spawn_local(send(Rc::clone(&wallet_window_rc_clone4), theme_value));
                });
                popup_grid.attach(&send_button, 0, 2, 1, 1);

                // Кнопка Receive
                let receive_button = Button::with_label("Receive");
                receive_button.connect_clicked(move |_| {
//                    receive();
//                    MainContext::default().spawn_local(receive());
                    MainContext::default().spawn_local(receive(theme_value));
                });
                popup_grid.attach(&receive_button, 1, 2, 1, 1);

                // Кнопка Donate
                let donate_button = Button::with_label("Donate");
                let wallet_window_rc_clone5 = Rc::clone(&wallet_window_rc);
                donate_button.connect_clicked(move |_| {
//                    donate();
//                    MainContext::default().spawn_local(donate(Rc::clone(&wallet_window_rc_clone5)));
                    MainContext::default().spawn_local(donate(Rc::clone(&wallet_window_rc_clone5), theme_value));
                });
                popup_grid.attach(&donate_button, 0, 3, 1, 1);

                if *premium_flag.borrow() {
                    println!("Premium flag is enabled");
                    // Кнопка Rotate
                    let rotate_button = Button::with_label("Rotate");
                    let wallet_window_rc_clone6 = Rc::clone(&wallet_window_rc);
                    rotate_button.connect_clicked(move |_| {
//                    rotate();
//                        MainContext::default().spawn_local(rotate(Rc::clone(&wallet_window_rc_clone6)));
                        MainContext::default().spawn_local(rotate(Rc::clone(&wallet_window_rc_clone6), theme_value));
                    });
                    popup_grid.attach(&rotate_button, 0, 5, 2, 2);
                } else {
                    println!("premium: false");
                }

                // Метка "Сборы"
                if let Some(apt_balance) = get_apt_balance() {
                    label3_clone.borrow_mut().set_text(&format!("Fundraising: {}", apt_balance));
                } else {
                    label3_clone.borrow_mut().set_text("Fundraising: Error");
                }                
            }

            // Функция для получения баланса APT
      	    fn get_apt_balance() -> Option<String> {
      	        let curl_command = "curl -sS 'https://fullnode.mainnet.aptoslabs.com/v1/accounts/0x60919385df081d9b73895462a68d016f9d38eae9f8a4c5d041567c5a0999261d/resources'";
      	        let result = Command::new("sh")
      	            .arg("-c")
      	            .arg(curl_command)
      	            .output()
      	            .expect("Failed to execute command");
      	
      	        if !result.stdout.is_empty() {
      	            let output = String::from_utf8_lossy(&result.stdout);
      	            let json_data: serde_json::Value = serde_json::from_str(&output).ok()?;
      	            for resource in json_data.as_array()? {
      	                if resource["type"] == "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>" {
      	                    let balance = resource["data"]["coin"]["value"].as_str()?;
      	                    let apt_balance = (balance.parse::<f64>().ok()? / 1e8).to_string();
      	                    return Some(apt_balance);
      	                }
      	            }
      	        }
      	        None
      	    }

//            label3_clone.borrow_mut().set_text("Fundraising: Updated");
//            if let Some(apt_balance) = get_apt_balance() {
//                label3_clone.borrow_mut().set_text(&format!("Fundraising: {}", apt_balance));
//            } else {
//                label3_clone.borrow_mut().set_text("Fundraising: Error");
//            }

            fn createwallet(wallet_window_clone2: Rc<Window>) {
            if !check_aptos_cli() {
                return;
            }
//            let path_to_aptos_cli = Path::new("/opt/aptos-core/target/cli/aptos");
//		if !path_to_aptos_cli.exists() {
//                    println!("Приложение aptos-cli не найдено. Чтобы работал кошелек, нужно подключиться к ноде.");
//		    return;
//		}
//                println!("[ok] aptos-cli найден.");
                // Сохраняем текущий каталог
		let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		// Переходим в каталог /opt/aptos-core
		env::set_current_dir("/opt/aptos-core").expect("Не удалось перейти в каталог /opt/aptos-core");
		// Проверяем существование файла /tmp/aptos_key
		let path_to_key = Path::new("/tmp/aptos_key");
		if path_to_key.exists() {
		    // Удаляем файл
		    if let Err(e) = fs::remove_file(&path_to_key) {
		        println!("Ошибка: Не удалось удалить файл /tmp/aptos_key: {}", e);
		        return;
		    }
		    // Проверяем снова
		    if path_to_key.exists() {
		        println!("Файл уже существует и не могу удалить. Сделайте сами: rm /tmp/aptos_key");
		        return;
		    }
		}
                // Проверяем существование файла /tmp/aptos_key.pub
		let path_to_pub_key = Path::new("/tmp/aptos_key.pub");
		if path_to_pub_key.exists() {
		    // Удаляем файл
		    if let Err(e) = fs::remove_file(&path_to_pub_key) {
		        println!("Ошибка: Не удалось удалить файл /tmp/aptos_key.pub: {}", e);
		        return;
		    }
		    // Проверяем снова
		    if path_to_pub_key.exists() {
		        println!("Файл уже существует и не могу удалить. Сделайте сами: rm /tmp/aptos_key.pub");
		        return;
		    }
                }
                // Генерация ключа
                let aptos_cli_path = "/opt/aptos-core";

                let command1 = format!("{}/target/cli/aptos key generate --output-file /tmp/aptos_key", aptos_cli_path);

		let status1 = Command::new("sh")
		    .arg("-c")
		    .arg(command1)
		    .status()
		    .expect("Не удалось выполнить команду генерации ключа");
		
		if status1.success() {
		    println!("Шаг 1: Ключ сгенерирован успешно.");
		} else {
		    println!("Шаг 1: Ошибка при генерации ключа.");
		    return;
		}
		
		// Чтение ключа
		let command2 = "cat /tmp/aptos_key";
		let output2 = Command::new("sh")
		    .arg("-c")
		    .arg(command2)
		    .output()
		    .expect("Не удалось выполнить команду чтения ключа");
		
                let private_key: String = if output2.status.success() {
	            let key = String::from_utf8(output2.stdout).expect("Не удалось прочитать ключ");
	            println!("Шаг 2: Ключ прочитан успешно: {}", key);
	            key
	        } else {
	            let error_message = String::from_utf8(output2.stderr).expect("Не удалось прочитать сообщение об ошибке");
	            println!("Шаг 2: Ошибка при чтении ключа: {}", error_message);
	            return;
	        };

                // Инициализация профиля
	        let command3 = format!("{APTOS_CLI_PATH}/target/cli/aptos init --profile default --network mainnet --private-key {}", private_key);
	        let output3 = Command::new("sh")
	            .arg("-c")
	            .arg(&command3)
	            .output()
	            .expect("Не удалось выполнить команду инициализации профиля");
	
	        let result3 = String::from_utf8(output3.stdout).expect("Не удалось прочитать результат команды инициализации профиля");
	        let err_msg3 = String::from_utf8(output3.stderr).expect("Не удалось прочитать сообщение об ошибке команды инициализации профиля");
	
	        if result3.contains("\"Result\": \"Success\"") {
	            println!("Шаг 3: Профиль инициализирован успешно:\n{}", result3);
	        } else {
	            println!("Шаг 3: Ошибка при инициализации профиля:\n{}\n{}", result3, err_msg3);
	            return;
	        }
                // Проверяем существование файла /tmp/aptos_key
		if path_to_key.exists() {
		    // Удаляем файл
		    if let Err(e) = fs::remove_file(&path_to_key) {
		        println!("Ошибка: Не удалось удалить файл /tmp/aptos_key: {}", e);
		        return;
		    }
		    // Проверяем снова
		    if path_to_key.exists() {
		        println!("Файл уже существует и не могу удалить. Сделайте сами: rm /tmp/aptos_key");
		        return;
		    }
		}
                // Проверяем существование файла /tmp/aptos_key.pub
		if path_to_pub_key.exists() {
		    // Удаляем файл
		    if let Err(e) = fs::remove_file(&path_to_pub_key) {
		        println!("Ошибка: Не удалось удалить файл /tmp/aptos_key.pub: {}", e);
		        return;
		    }
		    // Проверяем снова
		    if path_to_pub_key.exists() {
		        println!("Файл уже существует и не могу удалить. Сделайте сами: rm /tmp/aptos_key.pub");
		        return;
		    }
                }
                let path_to_yaml = Path::new("/opt/aptos-core/.aptos/config.yaml");
                // Проверка существования файла
	        if !path_to_yaml.exists() {
	            println!("[Ошибка]: Файл config.yaml не был создан.");
	            return;
	        } else {
                    println!("[Успех]: Файл config.yaml был успешно создан.");
                    let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Кошелек был успешно создан");
                    dialog.set_title(Some("[Успех]"));
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.show();
                }
		
		// Возвращаемся в предыдущий каталог
		env::set_current_dir(&current_dir).expect("Не удалось вернуться в предыдущий каталог");
		println!("Возвращен в каталог: {:?}", current_dir);

                wallet_window_clone2.close();
            }

            fn import(wallet_window_clone3: Rc<Window>, theme_value: i32) {
            if !check_aptos_cli() {
                return;
            }


//            let path_to_aptos_cli = Path::new("/opt/aptos-core/target/cli/aptos");
//		if !path_to_aptos_cli.exists() {
//                    println!("Приложение aptos-cli не найдено. Чтобы работал кошелек, нужно подключиться к ноде.");
//		    return;
//		}
//                println!("[ok] aptos-cli найден.");
                // Сохраняем текущий каталог
//		let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		// Переходим в каталог /opt/aptos-core
//		env::set_current_dir("/opt/aptos-core").expect("Не удалось перейти в каталог /opt/aptos-core");

                //
                //
                // Открытие окна с полем ввода и кнопками
	        let app = Application::new(Some("com.example.PrivateKey"), Default::default());
	        app.connect_activate(move |_| {
	            let window = Rc::new(RefCell::new(Window::new()));
	            window.borrow_mut().set_title(Some("Введите Приватный Ключ"));
	            window.borrow_mut().set_default_size(300, 100);

                    // Ставим тему к Import-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                    println!("Import: Current theme flag value inside activate: {}", theme_value);
                    // Проверяем значение theme_value
                    if theme_value == 1 {
                        // Создаем CSS для окна
                        let css_provider = CssProvider::new();
                        // Загружаем данные CSS как строку
                        css_provider.load_from_data("window { background-color: black; }");
                        let window_ref = window.borrow();
                        // Применяем CSS к окну
//                        let style_context = window.style_context();
                        let style_context = window_ref.style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    } else if theme_value == 2 {
                        let root_path = Path::new(ROOT_DIR);
//                        println!("Корневой каталог: {:?}", root_path);
		        // Формируем путь к изображению
		        let background_path = root_path.join("background.png");
		        // Создаем CSS для фонового изображения
		        let css_provider = CssProvider::new();
                        let window_ref = window.borrow();
		        let css = format!(
		            "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
		            background_path.to_str().unwrap()
		        );
		        css_provider.load_from_data(&css);
		        let style_context = window_ref.style_context();
		        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
		    }

	    
	            let vbox = Box::new(gtk4::Orientation::Vertical, 5);
	            let label = Label::new(Some("Приватный ключ:"));
	            let entry = Entry::new();
	    
	            let ok_button = Button::with_label("ОК");
	            let close_button = Button::with_label("ЗАКРЫТЬ");
	    
	            vbox.append(&label);
	            vbox.append(&entry);
	            vbox.append(&ok_button);
	            vbox.append(&close_button);
	    
	            // Работа с окном в замыканиях
	            let window_clone = Rc::clone(&window);
	            let entry_clone = entry.clone();
	            ok_button.connect_clicked(move |_| {
	                let priv_key = entry_clone.text().to_string();
	                if priv_key.is_empty() {
	                    println!("Ошибка: Приватный ключ не может быть пустым!");
	                } else {
	                    println!("Приватный ключ: {}", priv_key);

                            // Сохраняем текущий каталог
		            let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		            // Переходим в каталог /opt/aptos-core
		            env::set_current_dir("/opt/aptos-core").expect("Не удалось перейти в каталог /opt/aptos-core");

                            let command3 = format!("{APTOS_CLI_PATH}/target/cli/aptos init --profile default --network mainnet --private-key {}", priv_key);
                            let output3 = Command::new("sh")
	                        .arg("-c")
	                        .arg(&command3)
	                        .output()
	                        .expect("Не удалось выполнить команду инициализации профиля");
	
	                    let result3 = String::from_utf8(output3.stdout).expect("Не удалось прочитать результат команды инициализации профиля");
	                    let err_msg3 = String::from_utf8(output3.stderr).expect("Не удалось прочитать сообщение об ошибке команды инициализации профиля");
                            if result3.contains("\"Result\": \"Success\"") {
	                        println!("Шаг 3: Профиль инициализирован успешно:\n{}", result3);
	                    } else {
                                println!("Шаг 3: Ошибка при инициализации профиля:\n{}\n{}", result3, err_msg3);
	                        return;
	                    }

                            let path_to_yaml = Path::new("/opt/aptos-core/.aptos/config.yaml");
                            // Проверка существования файла
	                    if !path_to_yaml.exists() {
	                        println!("[Ошибка]: Файл config.yaml не был создан.");
	                        return;
	                    } else {
                                println!("[Успех]: Файл config.yaml был успешно создан.");
                                let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Кошелек был успешно импортирован");
                                dialog.set_title(Some("[Успех]"));
                                dialog.connect_response(|dialog, _| dialog.close());
                                dialog.show();
                            }

                            // Возвращаемся в предыдущий каталог
                            env::set_current_dir(&current_dir).expect("Не удалось вернуться в предыдущий каталог");
		            println!("Возвращен в каталог: {:?}", current_dir);

	                    window_clone.borrow_mut().close();
	                }
	            });
	    
	            let window_clone = Rc::clone(&window);
	            close_button.connect_clicked(move |_| {
	                window_clone.borrow_mut().close();
	            });
	    
	            window.borrow_mut().set_child(Some(&vbox));
	            window.borrow_mut().show();
	        });
	    
	        app.run();

                wallet_window_clone3.close();
            }

            check_paths(&text_label);

	    fn check_paths(text_label: &Label) {
	    let path_to_aptos = Path::new("/opt/aptos-core/.aptos");
		if !path_to_aptos.exists() {
                    text_label.set_label("N/A");
		    return;
		}
                let path_to_yaml = Path::new("/opt/aptos-core/.aptos/config.yaml");
                // Проверка существования файла
	        if !path_to_yaml.exists() {
	            println!("Файл config.yaml не существует.");
	            return;
	        }
		// Открытие файла
		let file = File::open(&path_to_yaml).expect("Не удалось открыть файл");
		
		// Чтение файла построчно
		let lines = io::BufReader::new(file).lines();
		// Переменные для хранения адреса
		let mut apt_addr = String::new();
		let mut in_default_section = false;
                let mut default_found = false;
		
		// Поиск нужного раздела и адреса
		for line in lines {
		    let line = line.expect("Не удалось прочитать строку");
		    if line.contains("default:") {
		        in_default_section = true;
                        default_found = true;
		    } else if in_default_section && line.trim().is_empty() {
		        in_default_section = false;
		    }
		
		    if in_default_section && line.contains("account:") {
		        if let Some(addr) = extract_address(&line) {
		            apt_addr = addr;
		            break;
		        }
		    }
		}
                // Проверка на наличие блока default
	        if !default_found {
	            println!("Ошибка: Профиль default не найден в config.yaml.");
	            return;
	        } 
	        // Проверка на наличие адреса
	        if apt_addr.is_empty() {
	            println!("Ошибка: Адрес не найден в блоке default.");
	            return;
	        }
		
		// Форматирование адреса
		let short_addr = format!("{}...{}", &apt_addr[0..10], &apt_addr[apt_addr.len()-8..]);
		
		// Вывод отладочных сообщений
		println!("Полный адрес: {}", apt_addr);
		println!("Сокращенный адрес: {}", short_addr);
		
		text_label.set_label(&short_addr);

		fn extract_address(line: &str) -> Option<String> {
		    let re = regex::Regex::new(r"[a-f0-9]{64}").expect("Не удалось создать регулярное выражение");
		    if let Some(captures) = re.captures(line) {
		        let mut addr = captures.get(0)?.as_str().to_string();
		        addr.insert_str(0, "0x");
		        return Some(addr);
		    }
		    None
		}
                // end
	    }

//            let path_to_aptos = Path::new("/opt/aptos-core/.aptos");
//	    if !path_to_aptos.exists() {
//                println!("кош не импортирован");
//                return false;
//            } else {
//                println!("кош импортирован, продолжаем");
//            }

//	    fn remove() {
//	        let dialog = MessageDialog::new(
//	            None::<&ApplicationWindow>,
//	            DialogFlags::MODAL,
//	            MessageType::Warning,
//	            ButtonsType::OkCancel,
//	            "Это удалит все данные об импортированных или созданных кошельках, продолжить?",
//	        );
//	        dialog.set_title(Some("Подтверждение удаления"));
//	    
//	        // Используем Rc<RefCell<bool>> для флага
//	        let is_handled = Rc::new(RefCell::new(false));
//	        let is_handled_clone = is_handled.clone();
//	    
//	        // Подключаем обработчик для кнопок
//	        dialog.connect_response(move |dialog, response| {
//	            // Проверяем флаг внутри RefCell
//	            if !*is_handled_clone.borrow() {
//	                *is_handled_clone.borrow_mut() = true; // Устанавливаем флаг
//	                if response == gtk4::ResponseType::Ok {
//	                    println!("Proceed with removal");
//	                    // Здесь код для удаления данных
//	                } else {
//	                    println!("Cancel removal");
//	                }
//	                dialog.close();
//	            }
//	        });
//	        dialog.show();
//	    }

            fn remove(wallet_window_clone: Rc<Window>) {
	        // Создаем диалоговое окно подтверждения удаления
	        let dialog = MessageDialog::new(None::<&ApplicationWindow>, DialogFlags::MODAL, MessageType::Warning, ButtonsType::OkCancel, "Это удалит все данные об импортированных или созданных кошельках, продолжить?");
	        dialog.set_title(Some("Подтверждение удаления"));
	    
	        // Используем Rc<RefCell<bool>> для предотвращения повторного срабатывания
	        let is_handled = Rc::new(RefCell::new(false));
	        let is_handled_clone = is_handled.clone();
	    
	        // Подключаем обработчик для кнопок
	        dialog.connect_response(move |dialog, response| {
	            if !*is_handled_clone.borrow() {
	                *is_handled_clone.borrow_mut() = true; 
	                if response == gtk4::ResponseType::Ok {
	                    // Выполняем команду удаления
	                    if let Err(e) = Command::new("rm")
	                        .arg("-r")
	                        .arg("/opt/aptos-core/.aptos")
	                        .status()
	                    {
	                        eprintln!("Ошибка при удалении данных: {}", e);
	                    } else {
	                        println!("Данные удалены");
	    
	                        // Создаем и показываем диалоговое окно с подтверждением удаления
                                let completed_dialog = MessageDialog::new(None::<&ApplicationWindow>, DialogFlags::MODAL, MessageType::Info, ButtonsType::Ok, "Все данные были очищены");
	                        completed_dialog.set_title(Some("Завершено"));
	                        completed_dialog.connect_response(|dialog, _| {
	                            dialog.close();
	                        });
	                        completed_dialog.show();

	    	    	        wallet_window_clone.close();
	                    }
	                } else {
	                    println!("Удаление отменено");
	                }
	                dialog.close();
	            }
	        });
	        // Показываем всплывающее окно
	        dialog.show();
	    }
	
//            async fn rotate(wallet_window_clone6: Rc<Window>) {
            async fn rotate(wallet_window_clone6: Rc<Window>, theme_value: i32) {
	        println!("Rotate");
                if !check_aptos_cli() {
                    return;
                }


//                let path_to_aptos_cli = Path::new("/opt/aptos-core/target/cli/aptos");
//		if !path_to_aptos_cli.exists() {
//                    println!("Приложение aptos-cli не найдено. Чтобы работал кошелек, нужно подключиться к ноде.");
//		    return;
//		}
//                println!("[ok] aptos-cli найден.");

                let app = Application::new(Some("com.example.Rotate"), Default::default());
	        app.connect_activate(move |_| {
	            let rotate_window = Rc::new(RefCell::new(Window::new()));
                    rotate_window.borrow_mut().set_title(Some("Введите новый private_key"));
	            rotate_window.borrow_mut().set_default_size(300, 150);

                    // Ставим тему к Rotate-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                    println!("Rotate: Current theme flag value inside activate: {}", theme_value);
                    // Проверяем значение theme_value
                    if theme_value == 1 {
                        // Создаем CSS для окна
                        let css_provider = CssProvider::new();
                        // Загружаем данные CSS как строку
                        css_provider.load_from_data("window { background-color: black; }");
                        // Применяем CSS к окну
                        let style_context = rotate_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    } else if theme_value == 2 {
                        let root_path = Path::new(ROOT_DIR);
//                        println!("Корневой каталог: {:?}", root_path);
                        // Формируем путь к изображению
                        let background_path = root_path.join("background.png");
                        // Создаем CSS для фонового изображения
                        let css_provider = CssProvider::new();
                        let css = format!(
                            "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                            background_path.to_str().unwrap()
                        );
                        css_provider.load_from_data(&css);
                        let style_context = rotate_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    }  


	    
	            let vbox = Box::new(gtk4::Orientation::Vertical, 5);
                    vbox.set_margin_top(10);
                    vbox.set_margin_start(10);
                    vbox.set_margin_end(10);
                    vbox.set_margin_bottom(10);
                    
                    let label_address = Label::new(Some("Новый private_key:"));
                    let entry_address = Entry::new();
                    entry_address.set_placeholder_text(Some("Введите новый private_key"));
                    
	            let ok_button = Button::with_label("Rotate");
	            let close_button = Button::with_label("Close");

                    vbox.append(&label_address);
                    vbox.append(&entry_address);
                    vbox.append(&ok_button);
                    vbox.append(&close_button);
	    	    
	            // Работа с окном в замыканиях
	            let rotate_window_clone = Rc::clone(&rotate_window);
                    let entry_address_clone = entry_address.clone();
	            ok_button.connect_clicked(move |_| {
                        let address_sender = entry_address_clone.text().to_string();
	                if address_sender.is_empty() {
	                    println!("Ошибка: private_key не может быть пустым!");
	                } else {
                            println!("Адрес: {}", address_sender);

                            // Сохраняем текущий каталог
		            let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		            // Переходим в каталог /opt/aptos-core
                            if env::set_current_dir("/opt/aptos-core").is_ok() {
                                println!("Текущий каталог изменен на /opt/aptos-core");

                                let aptos_binary = format!("{}/target/cli/aptos", APTOS_CLI_PATH);
//                                let output = Command::new("./target/cli/aptos")
//                                let output = Command::new(APTOS_CLI_PATH)
                                let output = Command::new(&aptos_binary)
		                    .args([
		                        "account", "rotate-key",
                                        "--new-private-key", &address_sender,
                                        "--save-to-profile", "rotate",
		                        "--max-gas", "1300",
		                        "--expiration-secs", "60",
		                        "--assume-yes",
		                    ])
		                    .output();

		                match output {
		                    Ok(result) => {
		                        let stderr = String::from_utf8_lossy(&result.stderr);
                                        let txn_link_regex = Regex::new(r"Transaction submitted: https://explorer\.aptoslabs\.com/txn/.*").unwrap();
                                        let stdout = String::from_utf8_lossy(&result.stdout);

			                let mut txn_link = String::new();
			
			                // Ищем совпадения с шаблоном на каждой строке
                                        for line in stderr.lines() {
			                    if let Some(captures) = txn_link_regex.captures(line) {
			                        txn_link = captures.get(0).map_or("".to_string(), |m| m.as_str().to_string());
			                        break;
			                    }
			                }

//                                        println!("Полный результат stdout:\n{}", String::from_utf8_lossy(&result.stdout));
//                                        println!("Полный результат stderr:\n{}", String::from_utf8_lossy(&result.stderr));
//                                        println!("До проверки транзакция:\n{}", txn_link);

                                        // Создаем новое окно для вывода сообщения
//			                let message_window = Window::new();
                                        let message_window = Rc::new(RefCell::new(Window::new()));
                                        let window = message_window.borrow_mut();
                                        window.set_title(Some("Результат транзакции"));
                                        window.set_default_size(400, 200);
			
			                let vbox = Box::new(Orientation::Vertical, 0);
			                let text_view = TextView::new();
                                        let scrolled_window = ScrolledWindow::new();
                                        scrolled_window.set_vexpand(true);
                                        scrolled_window.set_hexpand(true);
                                        scrolled_window.set_child(Some(&text_view));
			                let text_buffer = text_view.buffer();
                                        if txn_link.is_empty() {
				            text_buffer.set_text(&format!("{}\n\n!! Не забудьте сохранить новые данные: private_key, account и public_key, после чего нажмите Remove(для удаления старых данных) и импортируйте кошелек с новыми данными.\n\n{}", txn_link, stdout));
                                            println!("{}\n\n{}", txn_link, stdout);
				        } else {
                                            text_buffer.set_text(&format!("{}\n\n!! Не забудьте сохранить новые данные: private_key, account и public_key, после чего нажмите Remove(для удаления старых данных) и импортируйте кошелек с новыми данными.\n\n{}", txn_link, stdout));
                                            println!("{}\n\n{}", txn_link, stdout);
				        }

                                        let close_button = Button::with_label("Закрыть");
                                        vbox.append(&scrolled_window);
                                        let message_window_clone = Rc::clone(&message_window);
                                        close_button.connect_clicked(move |_| {
                                            let window = message_window_clone.borrow_mut();
                                            window.close();
                                        });

                                        text_view.set_margin_top(10);
                                        text_view.set_margin_bottom(10);
                                        text_view.set_margin_start(10);
                                        text_view.set_margin_end(10);
                                        close_button.set_margin_bottom(10);

                                        vbox.append(&text_view);
                                        vbox.append(&close_button);

                                        window.set_child(Some(&vbox));
                                        window.show();
			
			                // Если статус команды не успешный, выводим ошибку
			                if !result.status.success() {
                                            let error_message = format!(
				                "\nОшибка выполнения команды:\n{}",
				                String::from_utf8_lossy(&result.stderr)
				            );
				            text_buffer.insert_at_cursor(&error_message);
				            println!("{}", error_message);
			                }
		                    }
		                    Err(e) => println!("Не удалось выполнить команду:\n{}", e),
		                }
                            } else {
                                println!("Не удалось перейти в каталог /opt/aptos-core");
                            }

                            // Возвращаемся в предыдущий каталог
                            env::set_current_dir(&current_dir).expect("Не удалось вернуться в предыдущий каталог");
		            println!("Возвращен в каталог: {:?}", current_dir);

	                    rotate_window_clone.borrow_mut().close();
                        }
	            });

                    let rotate_window_clone = Rc::clone(&rotate_window);
	            close_button.connect_clicked(move |_| {
	                rotate_window_clone.borrow_mut().close();
	            });
	    
	            rotate_window.borrow_mut().set_child(Some(&vbox));
	            rotate_window.borrow_mut().show();
	        });
	    
	        app.run();

                wallet_window_clone6.close();
	    }
	
//	    async fn receive() {
            async fn receive(theme_value: i32) {
                // Создаем всплывающее окно и оборачиваем его в Rc
	        let receive_window = Rc::new(Window::new());
	        receive_window.set_title(Some("Receive"));
	        receive_window.set_default_size(300, 20);

                // Ставим тему к Receive-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                println!("Receive: Current theme flag value inside activate: {}", theme_value);
                // Проверяем значение theme_value
                if theme_value == 1 {
                    // Создаем CSS для окна
                    let css_provider = CssProvider::new();
                    // Загружаем данные CSS как строку
                    css_provider.load_from_data("window { background-color: black; }");
                    // Применяем CSS к окну
                    let style_context = receive_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } else if theme_value == 2 {
                    let root_path = Path::new(ROOT_DIR);
//                    println!("Корневой каталог: {:?}", root_path);
                    // Формируем путь к изображению
                    let background_path = root_path.join("background.png");
                    // Создаем CSS для фонового изображения
                    let css_provider = CssProvider::new();
                    let css = format!(
                        "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                        background_path.to_str().unwrap()
                    );
                    css_provider.load_from_data(&css);
                    let style_context = receive_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } 
                // Создаем вертикальный контейнер для размещения виджетов
	        let frame = Box::new(Orientation::Vertical, 10);
	    
	        // Создаем текстовое виджет
	        let text_view = TextView::new();
	        text_view.set_wrap_mode(gtk4::WrapMode::Word);
	        text_view.set_editable(false); // Делаем текстовое виджет только для чтения
	        text_view.set_size_request(380, 200); // Установка размера текстового виджета
	    
	        text_view.set_margin_top(10);
	        text_view.set_margin_start(10);
	        text_view.set_margin_end(10);
	        frame.set_margin_bottom(10);

                if let Some((_private_key, apt_addr)) = get_account().await {
//	            println!("Private key: {}", private_key);
//	            println!("Account address: {}", apt_addr);	    
	            
	            // Создаем текстовый буфер с передачей None
	            let buffer = TextBuffer::new(None); // Передаем None в качестве аргумента
	            let text = format!("{}\n", apt_addr);
	            buffer.set_text(&text);
	    
	            // Предположим, что у вас есть TextView, назовем его `text_view`
                    text_view.set_buffer(Some(&buffer));
	            // Примерная строка text_view, измените на фактическую, если необходимо
//	            println!("Текстовый буфер установлен с private_key: {}", text);
    	        } else {
	            eprintln!("Ошибка при получении информации о аккаунте.");
	        }

                // Создаем скроллбар
	        let scrolled_window = ScrolledWindow::new(); // Удаляем аргументы
	        scrolled_window.set_child(Some(&text_view));
	        scrolled_window.set_vexpand(true);
	    
	        // Добавляем скроллбар в контейнер
	        frame.append(&scrolled_window);
	    
	        // Создаем кнопку для закрытия окна
	        let close_button = Button::with_label("Close");
	    
	        // Клонируем Rc для использования в замыкании
	        let receive_window_clone = Rc::clone(&receive_window);
	        close_button.connect_clicked(move |_| {
	            receive_window_clone.close(); // Закрываем окно при нажатии
	        });
	    
	        // Добавляем кнопку закрытия в контейнер
	        frame.append(&close_button);
	    
	        receive_window.set_child(Some(&frame)); // Устанавливаем контейнер как содержимое окна
	        receive_window.show(); // Отображаем окно

	        println!("Receive");
	    }
	
//	    async fn export() {
            async fn export(theme_value: i32) {
	        // Создаем всплывающее окно и оборачиваем его в Rc
	        let export_window = Rc::new(Window::new());
	        export_window.set_title(Some("Export"));
	        export_window.set_default_size(300, 20);
	    
	        // Создаем вертикальный контейнер для размещения виджетов
	        let frame = Box::new(Orientation::Vertical, 10);
	    
	        // Создаем текстовое виджет
	        let text_view = TextView::new();
	        text_view.set_wrap_mode(gtk4::WrapMode::Word);
	        text_view.set_editable(false); // Делаем текстовое виджет только для чтения
	        text_view.set_size_request(380, 200); // Установка размера текстового виджета

                // Ставим тему к Export-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                println!("Export: Current theme flag value inside activate: {}", theme_value);
                // Проверяем значение theme_value
                if theme_value == 1 {
                    // Создаем CSS для окна
                    let css_provider = CssProvider::new();
                    // Загружаем данные CSS как строку
                    css_provider.load_from_data("window { background-color: black; }");
                    // Применяем CSS к окну
                    let style_context = export_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } else if theme_value == 2 {
                    let root_path = Path::new(ROOT_DIR);
//                    println!("Корневой каталог: {:?}", root_path);
                    // Формируем путь к изображению
                    let background_path = root_path.join("background.png");
                    // Создаем CSS для фонового изображения
                    let css_provider = CssProvider::new();
                    let css = format!(
                        "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                        background_path.to_str().unwrap()
                    );
                    css_provider.load_from_data(&css);
                    let style_context = export_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } 
	    
	        text_view.set_margin_top(10);
	        text_view.set_margin_start(10);
	        text_view.set_margin_end(10);
	        frame.set_margin_bottom(10);

//                let private_key = get_account().unwrap_or_else(|| {
//                    eprintln!("Ошибка при получении private_key.");
//                    "".to_string()
//                });
//                get_account();

//	        if let Some((private_key, apt_addr)) = get_account() {
                if let Some((private_key, _apt_addr)) = get_account().await {
//	            println!("Private key: {}", private_key);
//	            println!("Account address: {}", apt_addr);	    
	            
	            // Создаем текстовый буфер с передачей None
	            let buffer = TextBuffer::new(None); // Передаем None в качестве аргумента
	            let text = format!("{}\n", private_key);
	            buffer.set_text(&text);
	    
	            // Предположим, что у вас есть TextView, назовем его `text_view`
                    text_view.set_buffer(Some(&buffer));
	            // Примерная строка text_view, измените на фактическую, если необходимо
//	            println!("Текстовый буфер установлен с private_key: {}", text);
    	        } else {
	            eprintln!("Ошибка при получении информации о аккаунте.");
	        }

//                println!("Private key: {}", private_key);
//                let text_value = "Your Variable Value";
	    
	        // Создаем текстовый буфер с передачей None
//	        let buffer = TextBuffer::new(None); // Передаем None в качестве аргумента
//                let text = format!("{}\n", private_key);
//                buffer.set_text(&text); 
//	        text_view.set_buffer(Some(&buffer)); // Устанавливаем буфер в TextView
	    
	        // Создаем скроллбар
	        let scrolled_window = ScrolledWindow::new(); // Удаляем аргументы
	        scrolled_window.set_child(Some(&text_view));
	        scrolled_window.set_vexpand(true);
	    
	        // Добавляем скроллбар в контейнер
	        frame.append(&scrolled_window);
	    
	        // Создаем кнопку для закрытия окна
	        let close_button = Button::with_label("Close");
	    
	        // Клонируем Rc для использования в замыкании
	        let export_window_clone = Rc::clone(&export_window);
	        close_button.connect_clicked(move |_| {
	            export_window_clone.close(); // Закрываем окно при нажатии
	        });
	    
	        // Добавляем кнопку закрытия в контейнер
	        frame.append(&close_button);
	    
	        export_window.set_child(Some(&frame)); // Устанавливаем контейнер как содержимое окна
	        export_window.show(); // Отображаем окно

	    }
	
//	    async fn donate(wallet_window_clone5: Rc<Window>) {
            async fn donate(wallet_window_clone5: Rc<Window>, theme_value: i32) {
            if !check_aptos_cli() {
                return;
            }
//            let path_to_aptos_cli = Path::new("/opt/aptos-core/target/cli/aptos");
//		if !path_to_aptos_cli.exists() {
//                    println!("Приложение aptos-cli не найдено. Чтобы работал кошелек, нужно подключиться к ноде.");
//		    return;
//		}
//                println!("[ok] aptos-cli найден.");

                let app = Application::new(Some("com.example.Donate"), Default::default());
	        app.connect_activate(move |_| {
	            let donate_window = Rc::new(RefCell::new(Window::new()));
                    donate_window.borrow_mut().set_title(Some("Введите сумму"));
	            donate_window.borrow_mut().set_default_size(300, 150);

                    // Ставим тему к Donate-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                    println!("Donate: Current theme flag value inside activate: {}", theme_value);
                    // Проверяем значение theme_value
                    if theme_value == 1 {
                        // Создаем CSS для окна
                        let css_provider = CssProvider::new();
                        // Загружаем данные CSS как строку
                        css_provider.load_from_data("window { background-color: black; }");
                        // Применяем CSS к окну
//                        let style_context = donate_window.style_context();
                        let style_context = donate_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    } else if theme_value == 2 {
                        let root_path = Path::new(ROOT_DIR);
//                        println!("Корневой каталог: {:?}", root_path);
                        // Формируем путь к изображению
                        let background_path = root_path.join("background.png");
                        // Создаем CSS для фонового изображения
                        let css_provider = CssProvider::new();
                        let css = format!(
                            "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                            background_path.to_str().unwrap()
                        );
                        css_provider.load_from_data(&css);
                        let style_context = donate_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    }
	    
	            let vbox = Box::new(gtk4::Orientation::Vertical, 5);
                    vbox.set_margin_top(10);
                    vbox.set_margin_start(10);
                    vbox.set_margin_end(10);
                    vbox.set_margin_bottom(10);
                    
                    let label_amount = Label::new(Some("Сумма:"));
                    let entry_amount = Entry::new();
                    entry_amount.set_placeholder_text(Some("Введите сумму для отправки"));
                    
	            let ok_button = Button::with_label("Donate");
	            let close_button = Button::with_label("Close");

                    vbox.append(&label_amount);
                    vbox.append(&entry_amount);
                    vbox.append(&ok_button);
                    vbox.append(&close_button);

                    // Работа с окном в замыканиях
	            let donate_window_clone = Rc::clone(&donate_window);
                    let entry_amount_clone = entry_amount.clone();
	            ok_button.connect_clicked(move |_| {
                        let amount_text = entry_amount_clone.text().to_string();
                        let valid_number_regex = regex::Regex::new(r"^\d+(\.\d+)?$").unwrap();
	                if amount_text.is_empty() {
                            println!("Ошибка: Сумма не может быть пустой!");
                        } else if amount_text.parse::<f64>().is_err() {
                            println!("Ошибка: Сумма должна быть числом!");
                        } else if !valid_number_regex.is_match(&amount_text) {
                            println!("Ошибка: Сумма не коректна!");
	                } else {
                            println!("Сумма: {}", amount_text);

                            let amount_atomic = (amount_text.parse::<f64>().unwrap_or(0.0) * 1e8) as u64;
                            println!("В атомарных еденицах: {}", amount_atomic);
 
                            // Покупка премиума
                            if amount_atomic >= 100000000 {
                                let mut config = config_load().unwrap();
                                // Проверка и обновление значения premium
                                check_and_update_premium(&mut config);
                                println!("Премиум куплен, данные аккаунта зашифрованы и записаны в конфиг. Не делитесь ни с кем конфигом. (:");
                            } else {
                                println!("Донат отправлен меньше 1 APT.");
                            }
  
                            // Сохраняем текущий каталог
		            let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		            // Переходим в каталог /opt/aptos-core
                            if env::set_current_dir("/opt/aptos-core").is_ok() {
                                println!("Текущий каталог изменен на /opt/aptos-core");

                                let aptos_binary = format!("{}/target/cli/aptos", APTOS_CLI_PATH);
//                                let output = Command::new("./target/cli/aptos")
//                                let output = Command::new(APTOS_CLI_PATH)
                                let output = Command::new(&aptos_binary)
		                    .args([
		                        "account", "transfer",
		                        "--account", "0x60919385df081d9b73895462a68d016f9d38eae9f8a4c5d041567c5a0999261d",
                                        "--amount", &amount_atomic.to_string(),
		                        "--profile", "default",
		                        "--max-gas", "1300",
		                        "--expiration-secs", "60",
		                        "--assume-yes",
		                    ])
		                    .output();

		                match output {
		                    Ok(result) => {
//		                        if result.status.success() {
//		                            println!("Результат:\n{}", String::from_utf8_lossy(&result.stdout));
//		                        } else {
//                                            println!("Результат:\n{}", String::from_utf8_lossy(&result.stdout));
//		                            println!("Ошибка выполнения команды:\n{}", String::from_utf8_lossy(&result.stderr));
//		                        }
		                        let stderr = String::from_utf8_lossy(&result.stderr);
                                        let txn_link_regex = Regex::new(r"Transaction submitted: https://explorer\.aptoslabs\.com/txn/.*").unwrap();
                                        let stdout = String::from_utf8_lossy(&result.stdout);
		                        	
			                let mut txn_link = String::new();
			
			                // Ищем совпадения с шаблоном на каждой строке
			                for line in stderr.lines() {
			                    if let Some(captures) = txn_link_regex.captures(line) {
			                        txn_link = captures.get(0).map_or("".to_string(), |m| m.as_str().to_string());
			                        break;
			                    }
			                }

//                                        println!("Полный результат stdout:\n{}", String::from_utf8_lossy(&result.stdout));
//                                        println!("Полный результат stderr:\n{}", String::from_utf8_lossy(&result.stderr));
//                                        println!("До проверки транзакция:\n{}", txn_link);

                                        // Создаем новое окно для вывода сообщения
//			                let message_window = Window::new();
                                        let message_window = Rc::new(RefCell::new(Window::new()));
                                        let window = message_window.borrow_mut();
                                        window.set_title(Some("Результат транзакции"));
                                        window.set_default_size(400, 200);
			
			                let vbox = Box::new(Orientation::Vertical, 0);
			                let text_view = TextView::new();
                                        let scrolled_window = ScrolledWindow::new();
                                        scrolled_window.set_vexpand(true);
                                        scrolled_window.set_hexpand(true);
                                        scrolled_window.set_child(Some(&text_view));
			                let text_buffer = text_view.buffer();
                                        if txn_link.is_empty() {
				            text_buffer.set_text(&format!("{}\n\n{}", txn_link, stdout));
                                            println!("{}\n\n{}", txn_link, stdout);
				        } else {
//				            text_buffer.set_text(&format!("Найдена транзакция:\n{}", txn_link));
                                            text_buffer.set_text(&format!("{}\n\n{}", txn_link, stdout));
//                                            println!("Найдена транзакция:\n{}", txn_link);
                                            println!("{}\n\n{}", txn_link, stdout);
				        }

//                                        let close_button = Button::with_label("Закрыть");
//
//				        close_button.connect_clicked(move |_| {
//				            message_window.close(); // Закрываем окно
//				        });
                                        let close_button = Button::with_label("Закрыть");
                                        vbox.append(&scrolled_window);
                                        let message_window_clone = Rc::clone(&message_window);
                                        close_button.connect_clicked(move |_| {
//                                            message_window_clone.borrow_mut().close(); // Закрытие окна
                                            let window = message_window_clone.borrow_mut();
                                            window.close();
                                        });

                                        text_view.set_margin_top(10);
                                        text_view.set_margin_bottom(10);
                                        text_view.set_margin_start(10);
                                        text_view.set_margin_end(10);
                                        close_button.set_margin_bottom(10);

                                        vbox.append(&text_view);
                                        vbox.append(&close_button);

                                        window.set_child(Some(&vbox));
                                        window.show();
			
			                // Если статус команды не успешный, выводим ошибку
			                if !result.status.success() {
//			                    println!("Ошибка выполнения команды:\n{}", String::from_utf8_lossy(&result.stderr));
                                            let error_message = format!(
				                "\nОшибка выполнения команды:\n{}",
				                String::from_utf8_lossy(&result.stderr)
				            );
				            text_buffer.insert_at_cursor(&error_message);
				            println!("{}", error_message);
			                }
		                    }
		                    Err(e) => println!("Не удалось выполнить команду:\n{}", e),
		                }
                            } else {
                                println!("Не удалось перейти в каталог /opt/aptos-core");
                            }

                            //

                            // Возвращаемся в предыдущий каталог
                            env::set_current_dir(&current_dir).expect("Не удалось вернуться в предыдущий каталог");
		            println!("Возвращен в каталог: {:?}", current_dir);

	                    donate_window_clone.borrow_mut().close();
                        }
	            });
	    
	            let donate_window_clone = Rc::clone(&donate_window);
	            close_button.connect_clicked(move |_| {
	                donate_window_clone.borrow_mut().close();
	            });
	    
	            donate_window.borrow_mut().set_child(Some(&vbox));
	            donate_window.borrow_mut().show();
	        });
	    
	        app.run();

                wallet_window_clone5.close();

            }


//            async fn send(wallet_window_clone4: Rc<Window>) {
            async fn send(wallet_window_clone4: Rc<Window>, theme_value: i32) {
            if !check_aptos_cli() {
                return;
            }

//            let path_to_aptos_cli = Path::new("/opt/aptos-core/target/cli/aptos");
//		if !path_to_aptos_cli.exists() {
//                    println!("Приложение aptos-cli не найдено. Чтобы работал кошелек, нужно подключиться к ноде.");
//		    return;
//		}
//                println!("[ok] aptos-cli найден.");
	        
                let app = Application::new(Some("com.example.Send"), Default::default());
	        app.connect_activate(move |_| {
	            let send_window = Rc::new(RefCell::new(Window::new()));
                    send_window.borrow_mut().set_title(Some("Введите адрес и сумму"));
	            send_window.borrow_mut().set_default_size(300, 150);

                    let premium_flag = Rc::new(RefCell::new(false));
                    match config_load() {
	                Ok(config) => {
                            *premium_flag.borrow_mut() = config.premium.as_deref() == Some("jFg{s;QdsF#45#)&@e22./#d");
                            println!("premium: {}", premium_flag.borrow());
	                } 
	                Err(e) => {
	                    eprintln!("Error loading config: {}", e);
	                },
	            }
                    
                    // Ставим тему к Send-окну:
                    println!("Send: Current theme flag value inside activate: {}", theme_value);
                    // Проверяем значение theme_value
                    if theme_value == 1 {
                        // Создаем CSS для окна
                        let css_provider = CssProvider::new();
                        // Загружаем данные CSS как строку
                        css_provider.load_from_data("window { background-color: black; }");
                        // Применяем CSS к окну
                        let style_context = send_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    } else if theme_value == 2 {
                        let root_path = Path::new(ROOT_DIR);
//                        println!("Корневой каталог: {:?}", root_path);
                        // Формируем путь к изображению
                        let background_path = root_path.join("background.png");
                        // Создаем CSS для фонового изображения
                        let css_provider = CssProvider::new();
                        let css = format!(
                            "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                            background_path.to_str().unwrap()
                        );
                        css_provider.load_from_data(&css);
                        let style_context = send_window.borrow().style_context();
                        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                    } 
	    
	            let vbox = Box::new(gtk4::Orientation::Vertical, 5);
                    vbox.set_margin_top(10);
                    vbox.set_margin_start(10);
                    vbox.set_margin_end(10);
                    vbox.set_margin_bottom(10);
                    
                    let label_address = Label::new(Some("Адрес получателя:"));
                    let entry_address = Entry::new();
                    entry_address.set_placeholder_text(Some("Введите адрес получателя"));

                    let label_amount = Label::new(Some("Сумма (APT):"));
                    let entry_amount = Entry::new();
                    entry_amount.set_placeholder_text(Some("Введите сумму для отправки в аптосах"));

                    let label_gas = Label::new(Some("Максимальный газ (atomic):")); //
                    let entry_gas = Entry::new(); //
                    entry_gas.set_placeholder_text(Some("Введите лимит газа в атомарных единицах")); 
                    
	            let ok_button = Button::with_label("Send");
	            let close_button = Button::with_label("Close");

                    vbox.append(&label_address);
                    vbox.append(&entry_address);
                    vbox.append(&label_amount);
                    vbox.append(&entry_amount);
                    if *premium_flag.borrow() {
                        vbox.append(&label_gas);
                        vbox.append(&entry_gas);
                    }
                    vbox.append(&ok_button);
                    vbox.append(&close_button);
	    	    
	            // Работа с окном в замыканиях
	            let send_window_clone = Rc::clone(&send_window);
                    let entry_address_clone = entry_address.clone();
                    let entry_amount_clone = entry_amount.clone();
                    let premium_flag_clone = Rc::clone(&premium_flag);
	            ok_button.connect_clicked(move |_| {
                        let address_sender = entry_address_clone.text().to_string();
                        let amount_text = entry_amount_clone.text().to_string();

		        let gas_text = if *premium_flag_clone.borrow() {
		            Some(entry_gas.text().to_string())
		        } else {
		            None
		        };

                        let valid_address_regex = regex::Regex::new(r"^0x[a-fA-F0-9]{64}$").unwrap();
                        let valid_amount_regex = regex::Regex::new(r"^\d+(\.\d+)?$").unwrap();
	    
	                let mut gas_atomic = gas_text
                            .as_ref()
	                    .and_then(|text| text.parse::<u64>().ok())
	                    .unwrap_or(1300);
	    
	                if gas_atomic == 0 {
	                    gas_atomic = 1300;
	                }

                        if address_sender.is_empty() {
			    println!("Ошибка: Адрес не может быть пустым!");
                            return;
			} else if !valid_address_regex.is_match(&address_sender) {
			    println!("Ошибка: Адрес имеет неправильный формат!");
                            return;
			} else if amount_text.is_empty() {
			    println!("Ошибка: Сумма не может быть пустой!");
                            return;
			} else if !valid_amount_regex.is_match(&amount_text) {
			    println!("Ошибка: Сумма должна быть числом и корректным форматом!");
                            return;
			} else if *premium_flag.borrow() {
			    if let Some(gas_text) = &gas_text {
//			        if gas_text.is_empty() {
//			            println!("Ошибка: Газ не может быть пустым!");
//			            return;
//			        } else 
                                if gas_text.parse::<u64>().is_err() {
			            println!("Ошибка: Газ должен быть целым числом!");
                                    return;
			        }
			    }
			}
	    
	                let amount_atomic = (amount_text.parse::<f64>().unwrap() * 1e8) as u64;

                        println!("Адрес: {}", address_sender);
                        if *premium_flag_clone.borrow() {
                            println!("Газ в атомарных еденицах: {}", gas_atomic);
                        }
                        println!("Сумма в атомарных еденицах: {}", amount_atomic);

                            // Сохраняем текущий каталог
		            let current_dir = env::current_dir().expect("Не удалось получить текущий каталог"); 
		            // Переходим в каталог /opt/aptos-core
                            if env::set_current_dir("/opt/aptos-core").is_ok() {
                                println!("Текущий каталог изменен на /opt/aptos-core");

                                let aptos_binary = format!("{}/target/cli/aptos", APTOS_CLI_PATH);
//                                let output = Command::new("./target/cli/aptos")
//                                let output = Command::new(APTOS_CLI_PATH)
                                let output = Command::new(&aptos_binary)
		                    .args([
		                        "account", "transfer",
		                        "--account", &address_sender,
                                        "--amount", &amount_atomic.to_string(),
		                        "--profile", "default",
		                        "--max-gas", &gas_atomic.to_string(),
		                        "--expiration-secs", "60",
		                        "--assume-yes",
		                    ])
		                    .output();

		                match output {
		                    Ok(result) => {
//		                        if result.status.success() {
//		                            println!("Результат:\n{}", String::from_utf8_lossy(&result.stdout));
//		                        } else {
//                                            println!("Результат:\n{}", String::from_utf8_lossy(&result.stdout));
//		                            println!("Ошибка выполнения команды:\n{}", String::from_utf8_lossy(&result.stderr));
//		                        }
		                        let stderr = String::from_utf8_lossy(&result.stderr);
                                        let txn_link_regex = Regex::new(r"Transaction submitted: https://explorer\.aptoslabs\.com/txn/.*").unwrap();
                                        let stdout = String::from_utf8_lossy(&result.stdout);

			                let mut txn_link = String::new();
			
			                // Ищем совпадения с шаблоном на каждой строке
                                        for line in stderr.lines() {
			                    if let Some(captures) = txn_link_regex.captures(line) {
			                        txn_link = captures.get(0).map_or("".to_string(), |m| m.as_str().to_string());
			                        break;
			                    }
			                }

//                                        println!("Полный результат stdout:\n{}", String::from_utf8_lossy(&result.stdout));
//                                        println!("Полный результат stderr:\n{}", String::from_utf8_lossy(&result.stderr));
//                                        println!("До проверки транзакция:\n{}", txn_link);

                                        // Создаем новое окно для вывода сообщения
//			                let message_window = Window::new();
                                        let message_window = Rc::new(RefCell::new(Window::new()));
                                        let window = message_window.borrow_mut();
                                        window.set_title(Some("Результат транзакции"));
                                        window.set_default_size(400, 200);
			
			                let vbox = Box::new(Orientation::Vertical, 0);
			                let text_view = TextView::new();
                                        let scrolled_window = ScrolledWindow::new();
                                        scrolled_window.set_vexpand(true);
                                        scrolled_window.set_hexpand(true);
                                        scrolled_window.set_child(Some(&text_view));
			                let text_buffer = text_view.buffer();
                                        if txn_link.is_empty() {
				            text_buffer.set_text(&format!("{}\n\n{}", txn_link, stdout));
                                            println!("{}\n\n{}", txn_link, stdout);
				        } else {
//				            text_buffer.set_text(&format!("Найдена транзакция:\n{}", txn_link));
                                            text_buffer.set_text(&format!("{}\n\n{}", txn_link, stdout));
//                                            println!("Найдена транзакция:\n{}", txn_link);
                                            println!("{}\n\n{}", txn_link, stdout);
				        }

//                                        let close_button = Button::with_label("Закрыть");
//
//				        close_button.connect_clicked(move |_| {
//				            message_window.close(); // Закрываем окно
//				        });
                                        let close_button = Button::with_label("Закрыть");
                                        vbox.append(&scrolled_window);
                                        let message_window_clone = Rc::clone(&message_window);
                                        close_button.connect_clicked(move |_| {
//                                            message_window_clone.borrow_mut().close(); // Закрытие окна
                                            let window = message_window_clone.borrow_mut();
                                            window.close();
                                        });

                                        text_view.set_margin_top(10);
                                        text_view.set_margin_bottom(10);
                                        text_view.set_margin_start(10);
                                        text_view.set_margin_end(10);
                                        close_button.set_margin_bottom(10);

                                        vbox.append(&text_view);
                                        vbox.append(&close_button);

                                        window.set_child(Some(&vbox));
                                        window.show();
			
			                // Если статус команды не успешный, выводим ошибку
			                if !result.status.success() {
//			                    println!("Ошибка выполнения команды:\n{}", String::from_utf8_lossy(&result.stderr));
                                            let error_message = format!(
				                "\nОшибка выполнения команды:\n{}",
				                String::from_utf8_lossy(&result.stderr)
				            );
				            text_buffer.insert_at_cursor(&error_message);
				            println!("{}", error_message);
			                }
		                    }
		                    Err(e) => println!("Не удалось выполнить команду:\n{}", e),
		                }
                            } else {
                                println!("Не удалось перейти в каталог /opt/aptos-core");
                            }

                            //

                            // Возвращаемся в предыдущий каталог
                            env::set_current_dir(&current_dir).expect("Не удалось вернуться в предыдущий каталог");
		            println!("Возвращен в каталог: {:?}", current_dir);

	                    send_window_clone.borrow_mut().close();
//                        }
	            });
	    
	            let send_window_clone = Rc::clone(&send_window);
	            close_button.connect_clicked(move |_| {
	                send_window_clone.borrow_mut().close();
	            });
	    
	            send_window.borrow_mut().set_child(Some(&vbox));
	            send_window.borrow_mut().show();
	        });
	    
	        app.run();

                wallet_window_clone4.close();

            }

////            fn get_account() -> Option<String> {
//            async fn get_account() -> Option<(String, String)> {
//                let path_to_yaml = "/opt/aptos-core/.aptos/config.yaml";
//                if fs::metadata(path_to_yaml).is_err() {
//                    eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//                    return None;
//                }
//                let use_awk = true;
//
//                let get_value = |key: &str| -> Option<String> {
//	            if use_awk {
////	                println!("Получение значения для ключа с помощью awk: {}", key);
//                        println!("Получение значения для ключа с помощью awk");
//	                let command = format!(
//	                    r#"awk '/default:/{{flag=1; next}} /^[^ ]/{{flag=0}} flag {{print}}' {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'"#,
//	                    path_to_yaml, key
//	                );
////	                println!("Выполнение команды: {}", command);
//	    
//	                let output = Command::new("sh")
//	                    .arg("-c")
//	                    .arg(&command)
//	                    .output()
//	                    .expect("Failed to execute command");
//	    
//	                if !output.stdout.is_empty() {
////	                    println!("Команда выполнена успешно. Вывод: {:?}", String::from_utf8_lossy(&output.stdout));
//	                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
//	                } else {
//	                    println!("Команда выполнена, но вывод пустой. Ошибка: {:?}", String::from_utf8_lossy(&output.stderr));
//	                    None
//	                }
//	            } else {
////                        println!("Получение значения для ключа с помощью Rust: {}", key);
//                        println!("Получение значения для ключа с помощью Rust");
//	                let file = fs::File::open(path_to_yaml).expect("Не удалось открыть файл");
//	                let reader = io::BufReader::new(file);
//	    
//	                // Определение шаблона регулярного выражения
//	                let regex_pattern = match key {
//	                    "private_key:" => r#"private_key:[^\w]*"0x([a-f0-9]{64})""#,
//	                    "account:" => r#"account:[^\w]*([a-f0-9]{64})"#,
//	                    _ => return None,
//	                };
//	    
//	                let key_re = Regex::new(regex_pattern).unwrap();
////	                println!("Регулярное выражение: {:?}", key_re);
//	    
//	                for line in reader.lines() {
//	                    let line = line.expect("Не удалось прочитать строку");
////	                    println!("Чтение строки: {}", line);
//	                    if let Some(caps) = key_re.captures(&line) {
////	                        println!("Найдено совпадение: {:?}", caps);
//	                        return Some(format!("0x{}", &caps[1]));
//	                    }
//	                }
//	    
//	                None
//	            }
//	        };
//
//	        // Получение private_key
//	        let private_key = get_value("private_key:").unwrap_or_else(|| {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//	            "".to_string()
//	        });
//	    
//	        // Проверка формата private_key
//	        let key_regex = Regex::new(r"^0x[a-fA-F0-9]{64}$").unwrap();
//	        if private_key.is_empty() || !key_regex.is_match(&private_key) {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции или ключ имеет неправильный формат.");
//	            return None;
//	        }
//	    
//	        // Получение account
//	        let apt_addr = get_value("account:").unwrap_or_else(|| {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//	            "".to_string()
//	        });
//	    
//	        // Проверка формата account
//	        if apt_addr.is_empty() || !key_regex.is_match(&apt_addr) {
//	            eprintln!("Адрес аккаунта имеет неправильный формат.");
//	            return None;
//	        }
//
//                //
////                Some(private_key)
//                Some((private_key, apt_addr))
//            }
            
            wallet_window_rc.set_child(Some(&popup_grid));
            wallet_window_rc.present();
        });
        grid.attach(&wallet_button, 0, 2, 1, 1);

        #[derive(Deserialize)]
	struct Config {
	    use_awk: String,
	}
	
	fn load_config() -> Option<Config> {
//	    let config_path = Path::new("aptos_clockwork.yaml");
            let config_path = format!("{}/aptos_clockwork.yaml", ROOT_DIR);
	    let config_content = fs::read_to_string(config_path).ok()?;
	    serde_yaml::from_str(&config_content).ok()
	}

        async fn get_account() -> Option<(String, String)> {
                let path_to_yaml = "/opt/aptos-core/.aptos/config.yaml";
                if fs::metadata(path_to_yaml).is_err() {
                    eprintln!("Кошелек должен быть импортирован для работы этой функции.");
                    return None;
                }

                let config = load_config();

	        // Устанавливаем флаг на основе значения из файла конфигурации
	        let use_awk = match config.as_ref().map(|c| c.use_awk.as_str()) {
	            Some("yes") => true,
	            Some("no") => false,
	            _ => {
	                eprintln!("Неизвестное значение use_awk в конфигурации. Используется значение по умолчанию: false.");
	                false
	            }
	        };
//                let use_awk = true;
//                let use_awk = false;

                let get_value = |key: &str| -> Option<String> {
	            if use_awk {
//	                println!("Получение значения для ключа с помощью awk: {}", key);
                        println!("Получение значения для ключа с помощью awk");
	                let command = format!(
	                    r#"awk '/default:/{{flag=1; next}} /^[^ ]/{{flag=0}} flag {{print}}' {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'"#,
	                    path_to_yaml, key
	                );
//	                println!("Выполнение команды: {}", command);
	    
	                let output = Command::new("sh")
	                    .arg("-c")
	                    .arg(&command)
	                    .output()
	                    .expect("Failed to execute command");
	    
	                if !output.stdout.is_empty() {
//	                    println!("Команда выполнена успешно. Вывод: {:?}", String::from_utf8_lossy(&output.stdout));
	                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
	                } else {
	                    println!("Команда выполнена, но вывод пустой. Ошибка: {:?}", String::from_utf8_lossy(&output.stderr));
	                    None
	                }
	            } else {
//                        println!("Получение значения для ключа с помощью Rust: {}", key);
                        println!("Получение значения для ключа с помощью Rust");
	                let file = fs::File::open(path_to_yaml).expect("Не удалось открыть файл");
	                let reader = io::BufReader::new(file);
	    
	                // Определение шаблона регулярного выражения
	                let regex_pattern = match key {
	                    "private_key:" => r#"private_key:[^\w]*"0x([a-f0-9]{64})""#,
	                    "account:" => r#"account:[^\w]*([a-f0-9]{64})"#,
	                    _ => return None,
	                };
	    
	                let key_re = Regex::new(regex_pattern).unwrap();
//	                println!("Регулярное выражение: {:?}", key_re);
	    
	                for line in reader.lines() {
	                    let line = line.expect("Не удалось прочитать строку");
//	                    println!("Чтение строки: {}", line);
	                    if let Some(caps) = key_re.captures(&line) {
//	                        println!("Найдено совпадение: {:?}", caps);
	                        return Some(format!("0x{}", &caps[1]));
	                    }
	                }
	    
	                None
	            }
	        };


	        // Получение private_key
	        let private_key = get_value("private_key:").unwrap_or_else(|| {
	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
	            "".to_string()
	        });
	    
	        // Проверка формата private_key
	        let key_regex = Regex::new(r"^0x[a-fA-F0-9]{64}$").unwrap();
	        if private_key.is_empty() || !key_regex.is_match(&private_key) {
	            eprintln!("Кошелек должен быть импортирован для работы этой функции или ключ имеет неправильный формат.");
	            return None;
	        }
	    
	        // Получение account
	        let apt_addr = get_value("account:").unwrap_or_else(|| {
	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
	            "".to_string()
	        });
	    
	        // Проверка формата account
	        if apt_addr.is_empty() || !key_regex.is_match(&apt_addr) {
	            eprintln!("Адрес аккаунта имеет неправильный формат.");
	            return None;
	        }
            Some((private_key, apt_addr))
        }

        // Кнопка Transaction
        let input_clone1 = Rc::clone(&input);
        let transaction_button = Button::with_label("Transaction");
        transaction_button.connect_clicked(move |_| {
//            println!("Transaction button clicked");
            // Вставьте здесь код для обработки нажатия на кнопку Transaction
            let entered_text = input_clone1.borrow().text().to_string();
            // Проверка на пустую строку
            if entered_text.trim().is_empty() {
                // Выводим предупреждение
                let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Поле ввода пустое. Пожалуйста, введите адрес.");
                dialog.connect_response(|dialog, _| dialog.close());
                dialog.show();
                // Завершаем выполнение функции
                return;
            }
            println!("Address: {}", entered_text);
            // Проверка формата адреса
            let re = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();
            if !re.is_match(&entered_text) {
                let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Неверный формат адреса!");
                dialog.connect_response(|dialog, _| dialog.close());
                dialog.show();
                return;
            }
            // Формируем команду curl с заголовками и телом запроса
	    let output = Command::new("curl")
	        .arg("-sS")
	        .arg("--request").arg("GET")
	        .arg("--url").arg("https://fullnode.mainnet.aptoslabs.com/v1/accounts/0xfad6ba58af3c3a7d4a68215a47db5721fdb9d87b5c1cc4a1063aa97017493b7c/transactions")
	        .arg("--header").arg("Accept: application/json")
	        .output()
	        .expect("Не удалось выполнить команду curl");
	
	    let output_str = str::from_utf8(&output.stdout).expect("Ошибка преобразования строки");
	
	    let transactions: Vec<Value> = serde_json::from_str(output_str).expect("Ошибка парсинга JSON");
	    
	    for transaction in transactions {
	        let sequence_number = transaction["sequence_number"].as_str().unwrap_or("");
	        let timestamp = transaction["timestamp"].as_str().unwrap_or("");
	        let hash = transaction["hash"].as_str().unwrap_or("");
	        let gas_used = transaction["gas_used"].as_str().unwrap_or("");
	        let success = transaction["success"].as_bool().unwrap_or(false);
	        
	        let timestamp_num = timestamp.parse::<f64>().unwrap_or(0.0) / 1000000.0;

                let datetime = Utc.timestamp_opt(timestamp_num as i64, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Invalid timestamp".to_string());

	
	        println!("{{ sequence_number: {}, timestamp: {}, hash: {}, gas_used: {}, success: {} }}", 
	            sequence_number, datetime, hash, gas_used, success);
	    }
            let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "15 последних транзакций насраны в консоль.");
            dialog.connect_response(|dialog, _| dialog.close());
            dialog.show();

        });
        grid.attach(&transaction_button, 1, 2, 1, 1);

        // Кнопка SPKR
        let spkr_button = Button::with_label("SPKR");
        spkr_button.connect_clicked(move |_| {

            // Создаем новое окно
            let spkr_window = Window::new();
            spkr_window.set_title(Some("SPKR"));
            spkr_window.set_default_size(200, 200);
            // Ставим тему к SPKR-окну:
//            if *premium_flag.borrow() {
//                println!("Флаг premium внутри приложения: true");
//            } else {
//                println!("Флаг premium внутри приложения: false");
//            }
                println!("SPKR: Current theme flag value inside activate: {}", theme_value);
                // Проверяем значение theme_value
                if theme_value == 1 {
                    // Создаем CSS для окна
                    let css_provider = CssProvider::new();
                    // Загружаем данные CSS как строку
                    css_provider.load_from_data("window { background-color: black; }");
                    // Применяем CSS к окну
                    let style_context = spkr_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } else if theme_value == 2 {
                    let root_path = Path::new(ROOT_DIR);
//                    println!("Корневой каталог: {:?}", root_path);
                    // Формируем путь к изображению
                    let background_path = root_path.join("background.png");
                    // Создаем CSS для фонового изображения
                    let css_provider = CssProvider::new();
                    let css = format!(
                        "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
                        background_path.to_str().unwrap()
                    );
                    css_provider.load_from_data(&css);
                    let style_context = spkr_window.style_context();
                    style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                } 
            // Создаем сетку для содержимого всплывающего окна
            let popup_grid = Grid::new();
            popup_grid.set_column_spacing(10);
            popup_grid.set_row_spacing(10);

            glib::MainContext::default().spawn_local(async move {
                if !Path::new("/bin/freebsd-version").exists() {
		    eprintln!("Данная функция работает только на стационарном компьютере с OS FreeBSD");
                    let dialog = MessageDialog::new(
                        None::<&Window>,
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        "Данная функция работает только на стационарном компьютере с OS FreeBSD",
                    );
                    dialog.set_title(Some("Installation (I)"));
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.show();
                    return;
//		} else {
//                    eprintln!("[ok] OS FreeBSD");
                }
                if Command::new("kldstat").arg("-m").arg("speaker").status().map_or(true, |s| !s.success()) {
		    eprintln!("Модуль speaker не загружен. Загрузите модуль при помощи:\n# kldload speaker");
                    let dialog = MessageDialog::new(
                        None::<&Window>,
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        "Модуль speaker не загружен. Загрузите модуль при помощи:\n# kldload speaker",
                    );
                    dialog.set_title(Some("Installation (II)"));
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.show();
		    return;
//		} else {
//                    eprintln!("[ok] speaker.ko найден");
                }
                const FILE_PATH: &str = "/etc/devfs.conf";
                let file_path = "/etc/devfs.conf";

//		// Проверка существования файла и его чтение
		if let Ok(content) = fs::read_to_string(FILE_PATH) {
		    let perm = content.lines()
		        .find(|line| line.trim().starts_with("perm") && line.contains("speaker"));
		    let own = content.lines()
		        .find(|line| line.trim().starts_with("own") && line.contains("speaker"));
		
		    // Вывод результатов
//		    println!("perm: {:?}", perm);
//		    println!("own: {:?}", own);
		
		    // Проверка значений own
		    if let Some(own_value) = own {
		        if own_value.starts_with('#') {
                            eprintln!("[Error]: Похоже значение, выбранное из own начинается с #");
                            show_installation_dialog(file_path);
		            return;
		        } else if !own_value.contains("0666") {
                            eprintln!("[Error]: Не верная группа own , должно быть 0666");
		            show_installation_dialog(file_path);
                            return;
		        }
		    } else {
                        eprintln!("[Error]: Нет строки own в файле {} или она закомментирована", FILE_PATH);
		        show_installation_dialog(file_path);
                        return;
		    }
		    // Проверка значений perm
		    if let Some(perm_value) = perm {
		        if perm_value.starts_with('#') {
                            eprintln!("[Error]: Похоже значение, выбранное из perm начинается с #");
                            show_installation_dialog(file_path);
		            return;
		        } else if !perm_value.contains("0666") {
                            eprintln!("[Error]: Не верная группа perm, должно быть 0666");
		            show_installation_dialog(file_path);
		            return;
		        }
		    } else {
                        eprintln!("[Error]: Нет строки perm в файле {} или она закомментирована", FILE_PATH);
		        show_installation_dialog(file_path);
		        return;
		    }
		    // Если все проверки пройдены
//		    println!("[ok] Настройки для speaker найдены и корректны.");
		} else {
		    eprintln!("[Error]: Файл {} не существует или не может быть прочитан.", FILE_PATH);
		}

//                let path_to_yaml = "/opt/aptos-core/.aptos/config.yaml";
//                if fs::metadata(path_to_yaml).is_err() {
//                    eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//                    return;
//                }

//                let get_value = |key: &str| {
//                    println!("Получение значения для ключа: {}", key);
//                    let output = Command::new("sh")
////		    Command::new("sh")
//		        .arg("-c")
//		        .arg(format!(
//		            "awk '/default:/'{{flag=1;next}}'/^[^ ]/'{{flag=0}}flag {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'",
//		            path_to_yaml, key
//		        ))
//		        .output()
//		        .expect("Failed to execute command");
//                    println!("Команда выполнена успешно.");
//                    output
//		};


//                get_account();

//                println!("Private key: {}", private_key);

//                let use_awk = true;

//                let get_value = |key: &str| {
////	            println!("Получение значения для ключа: {}", key);
//	            let command = format!(
////	                "awk '/default:/'{{flag=1;next}}'/^[^ ]/'{{flag=0}}flag {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'",
//                        r#"awk '/default:/{{flag=1; next}} /^[^ ]/{{flag=0}} flag {{print}}' {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'"#,
//	                path_to_yaml, key
//	            );
////	            println!("Выполнение команды: {}", command);
//	            let output = Command::new("sh")
//	                .arg("-c")
//	                .arg(&command)
//	                .output()
//	                .expect("Failed to execute command");
//	    
//	            if !output.stdout.is_empty() {
////	                println!("Команда выполнена успешно. Вывод: {:?}", String::from_utf8_lossy(&output.stdout));
//	            } else {
//	                println!("Команда выполнена, но вывод пустой. Ошибка: {:?}", String::from_utf8_lossy(&output.stderr));
//	            }
//	            output
//	        };
		
//                let get_value = |key: &str| -> Option<String> {
//	            if use_awk {
////	                println!("Получение значения для ключа с помощью awk: {}", key);
//                        println!("Получение значения для ключа с помощью awk");
//	                let command = format!(
//	                    r#"awk '/default:/{{flag=1; next}} /^[^ ]/{{flag=0}} flag {{print}}' {} | grep '{}' | head -n 1 | grep -oE '[a-f0-9]{{64}}' | sed 's/^/0x/'"#,
//	                    path_to_yaml, key
//	                );
////	                println!("Выполнение команды: {}", command);
//	    
//	                let output = Command::new("sh")
//	                    .arg("-c")
//	                    .arg(&command)
//	                    .output()
//	                    .expect("Failed to execute command");
//	    
//	                if !output.stdout.is_empty() {
////	                    println!("Команда выполнена успешно. Вывод: {:?}", String::from_utf8_lossy(&output.stdout));
//	                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
//	                } else {
//	                    println!("Команда выполнена, но вывод пустой. Ошибка: {:?}", String::from_utf8_lossy(&output.stderr));
//	                    None
//	                }
//	            } else {
////                        println!("Получение значения для ключа с помощью Rust: {}", key);
//                        println!("Получение значения для ключа с помощью Rust");
//	                let file = fs::File::open(path_to_yaml).expect("Не удалось открыть файл");
//	                let reader = io::BufReader::new(file);
//	    
//	                // Определение шаблона регулярного выражения
//	                let regex_pattern = match key {
//	                    "private_key:" => r#"private_key:[^\w]*"0x([a-f0-9]{64})""#,
//	                    "account:" => r#"account:[^\w]*([a-f0-9]{64})"#,
//	                    _ => return None,
//	                };
//	    
//	                let key_re = Regex::new(regex_pattern).unwrap();
////	                println!("Регулярное выражение: {:?}", key_re);
//	    
//	                for line in reader.lines() {
//	                    let line = line.expect("Не удалось прочитать строку");
////	                    println!("Чтение строки: {}", line);
//	                    if let Some(caps) = key_re.captures(&line) {
////	                        println!("Найдено совпадение: {:?}", caps);
//	                        return Some(format!("0x{}", &caps[1]));
//	                    }
//	                }
//	    
//	                None
//	            }
//	        };
//
////		let private_key_output = get_value("private_key:");
////		let private_key = String::from_utf8_lossy(&private_key_output.stdout).trim().to_string();
////	
////		// Проверка формата private_key
////		let key_regex = Regex::new(r"^0x[a-fA-F0-9]{64}$").unwrap();
////		if private_key.is_empty() || !key_regex.is_match(&private_key) {
////		    eprintln!("Кошелек должен быть импортирован для работы этой функции или ключ имеет неправильный формат.");
////		    return;
////		}
////		
////		let apt_addr_output = get_value("account:");
////		let apt_addr_lossy = String::from_utf8_lossy(&apt_addr_output.stdout);
////		let apt_addr = apt_addr_lossy.trim();
////		
////		// Проверка формата apt_addr
////		if apt_addr.is_empty() || !key_regex.is_match(apt_addr) {
////		    eprintln!("Адрес аккаунта имеет неправильный формат.");
////		    return;
////		}
//                
//                // Получение private_key
//	        let private_key = get_value("private_key:").unwrap_or_else(|| {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//	            "".to_string()
//	        });
//	    
//	        // Проверка формата private_key
//	        let key_regex = Regex::new(r"^0x[a-fA-F0-9]{64}$").unwrap();
//	        if private_key.is_empty() || !key_regex.is_match(&private_key) {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции или ключ имеет неправильный формат.");
//	            return;
//	        } 
//	        println!("Private key: {}", private_key);
//	    
//	        // Получение account
//	        let apt_addr = get_value("account:").unwrap_or_else(|| {
//	            eprintln!("Кошелек должен быть импортирован для работы этой функции.");
//	            "".to_string()
//	        });
//	    
//	        // Проверка формата account
//	        if apt_addr.is_empty() || !key_regex.is_match(&apt_addr) {
//	            eprintln!("Адрес аккаунта имеет неправильный формат.");
//	            return;
//	        }
//                println!("Найденный кошелек: {}", apt_addr);
//
                // Переменная для хранения значения apt_addr вне блока if let

                let mut apt_addr = String::new();
	        let mut private_key = String::new();
	
	        // Вызов асинхронной функции
	        if let Some((pk, extracted_apt_addr)) = get_account().await {
	            private_key = pk;
//	            println!("Private key: {}", private_key);
//	            println!("Account address: {}", extracted_apt_addr);	
	            // Пример использования значений в других частях кода
//	            println!("Найденный кошелек: {}", extracted_apt_addr);
	            // Присваиваем значение apt_addr извлеченному значению
	            apt_addr = extracted_apt_addr;
	        } else {
	            eprintln!("Ошибка при получении информации о аккаунте.");
	        }
	        // Используем переменную apt_addr за пределами блока if let
//	        println!("Private key: {}", private_key);
//                let apt_addr = "0xfad6ba58af3c3a7d4a68215a47db5721fdb9d87b5c1cc4a1063aa97017493b7c" ;
                println!("Account address: {}", apt_addr);
                let _ = &private_key;
//                let _ = &apt_addr;

                // Получаем значения переменных окружения
	        let api_key = env::var("API_KEY").unwrap_or_else(|_| "API_KEY".to_string());
	        let api_user = env::var("API_USER").unwrap_or_else(|_| "API_USER".to_string());
	        let limit = 25;
	        let mut offset = 0;

	        // Создаем HTTP-клиент
	        let client = Client::new();

                let mut nft_info: Vec<(String, String)> = Vec::new();
		
	        loop {
	            // Формируем JSON-запрос
	            let query = json!({
	                "query": "query fetchWalletInventoryWithListings( $where: nfts_bool_exp, $order_by: [nfts_order_by!] $offset: Int $limit: Int! ) { aptos { nfts(where: $where, order_by: $order_by, offset: $offset, limit: $limit) { id token_id token_id_index name media_url media_type ranking owner delegated_owner burned staked version chain_state claimable claimable_by claimable_reason claimable_contract_key collection { id slug semantic_slug title supply verified floor } listings(where: { listed: { _eq: true } }, order_by: { price: asc }) { id price price_str block_time seller market_name nonce contract { key } } topBid: bids( where: { status: { _eq: \"active\" } } order_by: { price: desc } limit: 1 ) { id bidder price } lastSale: actions( where: { type: { _in: [\"buy\", \"accept-collection-bid\", \"accept-bid\"] } } order_by: { block_time: desc } limit: 1 ) { price } contract { commission: default_commission { key market_fee market_name royalty is_custodial } } } } }",
	                "variables": {
	                    "where": {
	                        "_or": [
	                            { "owner": { "_eq": apt_addr }, "listed": { "_eq": true } },
	                            { "owner": { "_eq": apt_addr } },
	                            { "claimable_by": { "_eq": apt_addr } }
	                        ]
	                    },
	                    "order_by": [
	                        { "collection": { "title": "asc" } },
	                        { "ranking": "asc_nulls_last" },
	                        { "token_id_index": "asc_nulls_last" }
	                    ],
	                    "limit": limit,
	                    "offset": offset
	                }
	            });

                    // Выполняем HTTP-запрос
                    let response = match client
			    .post("https://api.indexer.xyz/graphql")
			    .header("x-api-key", &api_key)
			    .header("x-api-user", &api_user)
			    .header("Content-Type", "application/json")
			    .json(&query)
			    .send()
			    .await {
			        Ok(resp) => resp, // Если запрос успешен, сохраняем ответ
			        Err(err) => {
			            eprintln!("Ошибка при выполнении запроса: {}", err);
			            return; // Выход из блока в случае ошибки
			        }
			    };

                    // Получаем ответ в виде JSON
		    let response_json: serde_json::Value = match response.json().await {
		        Ok(json) => {
//		            println!("Response JSON: {:?}", json); // Выводим JSON-ответ
		            json // Если обработка успешна, сохраняем JSON
		        }
		        Err(err) => {
		            eprintln!("Ошибка при обработке ответа: {}", err);
		            return; // Выход из блока в случае ошибки
		        }
		    };

                    // Обрабатываем каждый элемент из массива NFTs
		    if let Some(nfts) = response_json["data"]["aptos"]["nfts"].as_array() {
		        for nft in nfts {
		            // Получаем значения полей "name" и "title"
		            let name = nft["name"].as_str().unwrap_or("Unknown name").to_string();
		            let title = nft["collection"]["title"].as_str().unwrap_or("Unknown title").to_string();
		            nft_info.push((name, title)); // Сохраняем вектор кортежей
		        }
		
		        // Если количество NFT меньше лимита, прекращаем цикл
		        if nfts.len() < limit {
		            break; // Прекращаем цикл
		        }
		
		        // Увеличиваем OFFSET и продолжаем цикл
		        offset += limit;
		        println!("Ждём...");
		        thread::sleep(Duration::from_secs(3));
		    } else {
		        eprintln!("NFTs не найдены или неверный формат ответа");
		        break;
		    }
	        }

                // Создаем и выводим nft_lines
	        let mut nft_lines = Vec::new();
	
	        for (name, title) in nft_info {
	            let formatted_line = format!("Name: {}, Collection: {}", name, title);
	            nft_lines.push(formatted_line.clone());
	        }

                // Преобразуем данные
                let mut output = Vec::new();
		
//            println!("Кол-во строк до функции: {:?}", label1_clone_async.borrow().text());

//            // Добавляем тестовые строки вручную для диагностики и дебага
                output.push(("Hermione #14".to_string(), "Triss Collection".to_string()));
                output.push(("Alice #47".to_string(), "Wonderland Collection".to_string()));
                output.push(("asr2-0das".to_string(), "Random Collection".to_string()));
                output.push(("Triss #43".to_string(), "Random Collection".to_string()));
                output.push(("Zoe #64".to_string(), "Random Collection".to_string()));
                output.push(("Cersei #124".to_string(), "Random Collection".to_string()));
//                println!("{:?}", output);

                // Инициализация текста кнопок и меток
	        let mut labels = vec![
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	            "Not on the balance sheet",
	        ];
	        let mut play_status = vec!["None"; 9];

                // Заданные строки для проверки
	        let possible_outputs = vec![
	            "Hermione #14",
	            "Triss #43",
	            "Alice #47",
	            "Zoe #64",
	            "Cersei #124",
	            "Girl #5",
	            "Girl #6",
	            "Girl #7",
	            "Girl #8",
	        ];

                for (index, possible_output) in possible_outputs.iter().enumerate() {
		    if output.iter().any(|(name, collection)| name == possible_output || collection == possible_output) {
		        labels[index] = possible_output;
		        play_status[index] = "Play";
		    }
		}

	        // Создание меток и кнопок
	        for i in 0..9 {
		    let text_label = Label::new(Some(labels[i])); // Используем обновленные метки
		    popup_grid.attach(&text_label, 3, i as i32, 1, 1);
		
		    let button = Button::with_label(play_status[i]); // Используем обновленные кнопки
                    button.set_size_request(100, 20); // Задаем фиксированный размер кнопки (ширина, высота)

                    // Устанавливаем отступ сверху для кнопки
		    if i == 0 {
		        button.set_margin_top(10); // Отступ 10 пикселей для первой кнопки
		    }

		    if play_status[i] == "None" {
		        button.set_sensitive(false); // Отключаем кнопку, если статус "None"
		    } else {
		        let index = i; // Создаем захват для замыкания
		        let button_clone = button.clone();
		        button.connect_clicked(move |_| {
		            execute_condition(index); // Вызываем функцию при нажатии
		            
		            // Делаем кнопку неактивной и меняем текст
		            button_clone.set_sensitive(false);
		            button_clone.set_label("None");
		
		            // Таймер для отображения обратного отсчета
		            let countdown_seconds = 23; // Общее время до разблокировки
		            let mut seconds_remaining = countdown_seconds; // Переменная для отслеживания оставшегося времени
		
		            let button_clone_for_timer = button_clone.clone(); // Создаём новый клон для таймера
		            timeout_add_seconds_local(1, move || { // Запускаем таймер с интервалом 1 секунда
		                if seconds_remaining > 0 {
		                    seconds_remaining -= 1; // Уменьшаем оставшееся время на 1 секунду
		                    button_clone_for_timer.set_label(&format!("Play ({}s)", seconds_remaining)); // Обновляем текст кнопки
		                    ControlFlow::Continue // Продолжаем таймер
		                } else {
		                    button_clone_for_timer.set_sensitive(true); // Разблокируем кнопку
		                    button_clone_for_timer.set_label("Play"); // Обновляем текст кнопки
		                    ControlFlow::Break // Завершаем таймер
		                }
		            });
		        });
		    }
		    popup_grid.attach(&button, 8, i as i32, 1, 1);
	        }	

                // Добавляем текстовую метку
                let text_label = Label::new(Some("Здесь отображаются NFT, которые имеют\nвстроенный музыкальный code. Code можно\nвоспроизвести при нажатии на Play.\nНайти NFT с кодом из коллекции Clockwork OG\nможно на маркетплейсах: wapal или tradeport."));
                popup_grid.attach(&text_label, 0, 9, 12, 1);

                // Создаем кнопку "Close" для закрытия окна
                let close_button = Button::with_label("Close");
                let spkr_window_rc = Rc::new(spkr_window);

                close_button.set_margin_bottom(10);

                // Используем Rc для управления временем жизни spkr_window
                let spkr_window_clone = Rc::clone(&spkr_window_rc);
                close_button.connect_clicked(move |_| {
                    spkr_window_clone.close(); // Закрытие окна
                });
                popup_grid.attach(&close_button, 0, 12, 12, 1); // Кнопка Close в позиции (0, 3)

                spkr_window_rc.set_child(Some(&popup_grid));
                spkr_window_rc.present();
            });
        });
        grid.attach(&spkr_button, 2, 2, 1, 1);

        // Кнопка About
        let about_button = Button::with_label("About");
        about_button.connect_clicked(|_| {
            on_about_button_clicked();
        });
        grid.attach(&about_button, 3, 2, 1, 1);

        // Если нужно установить кастомный фон, загружаем изображение
//        if use_custom_background {
//            match set_custom_background(&window).await {
//                Ok(_) => println!("Кастомный фон успешно установлен."),
//                Err(err) => eprintln!("Не удалось установить кастомный фон: {}", err),
//            }
//        }
                                          
        // Устанавливаем сетку как содержимое окна
        window.set_child(Some(&grid));

        // Отображаем окно
        window.present();
    });

    app.run();

    Ok(())
}

fn execute_condition(index: usize) {
    let mut music = vec![None; 5];
    // Здесь вы можете обработать условие нажатия кнопки
    match index {
        0 => {
            println!("Name: Hermione #14, Music: Hogwarts Themes – Jeremy Soule");
            music[0] = Some("T110 L4EF+L8GF+L4E P4AL8GF+L4E P4BL8AGF+E L4DL8DF+L4EP4 L2MLB.L8MNBO5L16CO4B L2MLA+L8A+MNBO5L4F+ L2MLG.L8GMNL16AG L4F+L8GF+L4O4BL4O5D+ O5MLB.L8BMNL16O6CO5B L2A+.L4F+ L4BO4BO5CO4A+ D+EF+G L1O3F+ L4GL8F+GL4AL8GF+ L2GL4AF+ L2B.L4B MLA+L2F+MNA+ MLAFMNA L2G+L4AMLBL1BMN T110 O3L4EF+L8GF+L4E P4AL8GF+L4E P4BL8AGF+E L4DL8DF+L4EP4 EF+L2MLD+ L2D+MNL4EF+ D+L2AL4F+ EF+L8D+EL4F+ GL8D+EL4F+GL1G");
        },
        1 => {
            println!("Name: Triss #43, Music: The Slopes of the Blessure");
            music[0] = Some("T85 P8L8O3DO4L4DL8CDEC L4O3AL8O2FEF.L32EFEL4D O3L8DEFAL4GL8O4CO3G L4AP8L16GL32FGFL4EL16FGEF L8DDL4O4DL8CDEC L4F.L12EFEL4DO3B L8DEFA G.L32AGL8FE L4D.L8AO4L4DL8DE L4FL8O3AGL4AL4O4D O3L8FEDO2AL4O3CL8DE L4FL8O2AGL4AL4D O1L4DO2AO3DO4L8DE L4FL8O3AGL4AO4D L8FEDO3AL4O4CL8DE L4FL8O3AGL4AL4D L1O1D P8L8O3DO4L4DL8CDEC L4O3AL8O4FEF.L24EFEL4D L8O3DEFAL4GL8O4CO3G L4AL8O4FEDO3AFE DDL4O4DL8CDEC L4F.L12EFEL8D.L16O3DL4B L8DEFAG.L32AGL8FE L4D.L8AO4L4DL8DE");
            music[1] = Some("T85 L4FL8O3AGL4AO4D O1L8DAO3DAG.L32AGL8FE L4CD.L12EFEL8DC L4D.L8AO4L4EL8EF O4L4FO3L8AGL4AO4D L8FEDO3AO4L4CL8DE L4FL8O3AGL4AO4D L1O1D");
        }
        2 => {
            println!("Name: Alice #47, Music: Menuet_-_Luigi_Boccherini");
            music[0] = Some("T80 P2L16O5DC+DE L8DO4L4DF+L8A AGL4G L16GF+GA L8GO3L4AO4EL8G GF+L4F+ O5L6DL16O4B L8AG+G+G+ O5L6DL16O4B L8AG+G+G+ O5L6DL16O4B O5L8C+O4AF+O5DO4L8BL16EG+ L4AP4 L8O2AAAAO4L16O5DC+DE L8DO4L4DF+L8A AGL4G L16GF+GA L8GO3L4AO4EL8G GF+L4F+ O5L6DL16O4B L8AG+G+G+ O5L6DL16O4B L8AG+G+G+ O5L6DL16O4B O5L8C+O4AF+O5DO4L8BL16EG+ L4AP4");
            music[1] = Some("T80 O4L6FL16D L8EO3AAA O4L6DO3L16A L8O4C+O3AAA O4L6FL16D L8EO3AAA O4L6DO3L16A L8O4C+O3AAAL16O5DC+DE L8DO4L4DF+L8A AGL4G L16GF+GA L8GO3L4AO4EL8G GF+L4F+L6GL16E L6DL16C+L8C+C+ L6GL16E L6DL16C+L8C+C+L6GL16E L8F+DO3BO4GL6EL16D L4DL8O5D");
        },
        3 => {
            println!("Name: Zoe #64, Music: Yankee_Doodle");
            music[0] = Some("T120 02L4B-L8B-B-L4B-L8B-L16B-B- L8B-B-B-B-B-B-O3E MSAABMNO4C+MSO3AO4C+O3BMNE MSAABMNO4C+O3L4AMSG+MNL8E MSAABO4MNC+MSDC+O3BMNA MSG+EF+G+L4MNAL8MSAMNMSE AABMNO4C+O3MSAO4MSC+O3BMNE MSAABMNO4C+L4O3AL8MSG+MNE MSAABMNO4C+MSDC+O3BMNA MSG+EF+G+L4MNAL8AMNE MSF+.L16G+L8F+EF+G+AF+ E.L16F+L8EDC+C+EE F+.L16G+L8F+EF+G+AF+ EMNAMSG+MNBL4AL8MSAMNE MSF+.L16G+L8F+EF+G+AF+ E.L16F+L8EDC+C+EE F+.L16G+L8F+EF+G+AF+ EAG+MNBL4A MSL8AE L1A");
        },
        4 => {
            println!("Name: Cersei #124, Music: Game_of_trones");
            music[0] = Some("T168 O3L4GCL8E-F L4GCL8E-F L4GCL8E-F L4GCL8E-F L4GCL8EF L4GCL8EF L4GCL8EF L4GCL8EF L2.G C. L8E-FL2G CL8E-F L4DO2GL8B-O3C L4DO2GL8B-O3C L4DO2GL8B-O3C L4DMSO2GB-MN L2O3F. O2B-. L8O3E-DL2F O2B- O3L8E-D L4CO2FL8A-B- O3L4CO2FL8A-B- O3L4CO2FL8A-B- O3L4CO2MSFA-MN");
            music[1] = Some("T168 O3L2G. C. L8E-FL2G CL8E-F L2DO2L8B-O3C L2DL8O2B-O3C L2DL8O2B-O3C L4DO2B-D L2F. O2B-. O3DL4E- L2D. L4CO2GL8A-B- L4O3CO2GL8A-B- L4O3CO2GL8A-B- L4CCC O4CO3E-L8A-B- O4L4CO3E-L8A-C L4B-E-L8GA- L4B-E-L8GB- L4A-CL8FG L4A-CL8GA- L4GCL8E-F L4GCL8E-F L4E-O2A-O3L8CD L4E-O2A-O3L8CD L4E-O2A-O3E- FE-F GCL8E-F L4GCL8E-F L4GCL8E-F L4GA-B-");
            music[2] = Some("T168 L4O4CO3E-L8A-B- O4L4CO3E-L8A-C L4B-E-L8GA- L4B-E-G A-CL8FG L4A-CL8GA- L4GCL8E-F L4GCD E-O2A-O3L8CD L4O3E-O2A-O3L8CD L4O3E-O2A-O3E- DO2GO3D CO2GL8A-B- L4O3CO2GL8A-B- L4O3CO2GL8A-B- L4O3CO2GL8A-B- L4O3CO4GL8A-B- O5L4CO4GL8A-B- O5L4CO4GL8A-B- O5L2C.");
        },
        5 => {
            println!("Name: TEXT");
            music[0] = Some("AAA- BCG# AA");
            music[1] = Some("P32");
        },
        6 => {
            println!("Name: TEXT");
            music[0] = Some("AAA- BCG# AA");
        },
        7 => {
            println!("Name: TEXT");
            music[0] = Some("AAA- BCG# AA");
        },
        8 => {
            println!("Name: TEXT");
            music[0] = Some("AAA- BCG# AA");
        },
//        9 => println!("Name: TEXT"),
        // Добавьте остальные условия
        _ => println!("Условие для элемента {}", index),
    }
    // Отсрочка результата debug после нажатия на кнопку (показывает выхлоп [код] в консоль):
//    for (i, item) in music.iter().enumerate() {
//        println!("music{}: {}", i + 1, item.unwrap_or(""));
//    }

    // Отправляем каждую строку по очереди в устройство /dev/speaker
    let spath = "/dev/speaker";
    for (i, item) in music.iter().enumerate() {
        if let Some(content) = item {
            match OpenOptions::new().write(true).open(spath) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", content) {
                        eprintln!("Ошибка записи в устройство {} (music{}): {}", spath, i + 1, e);
                    }
                }
                Err(e) => {
                    eprintln!("Не удалось открыть {} для music{}: {}", spath, i + 1, e);
                }
            }
        }
    }
//    // Собираем строки из массива music
//    let output = music
//        .iter()
//        .map(|item| item.unwrap_or("")) // Заменяем None на пустую строку
//        .collect::<Vec<_>>()
//        .join(" "); // Объединяем значения через пробел
//
//    // Записываем в устройство /dev/speaker
//    let spath = "/dev/speaker";
//    match OpenOptions::new().write(true).open(spath) {
//        Ok(mut file) => {
//            if let Err(e) = writeln!(file, "{}", output) {
//                eprintln!("Ошибка записи в устройство {}: {}", spath, e);
//            }
//        }
//        Err(e) => {
//            eprintln!("Не удалось открыть {}: {}", spath, e);
//        }
//    }

}

// Функция для создания таблицы для Coin
fn create_coin_table() -> TreeView {
    let treeview = TreeView::new();

    let columns = vec![
        ("Name", 150),
        ("Amount", 100),
        ("Symbol", 100),
        ("Standard", 100),
        ("Decimals", 100),
        ("Asset_type", 200),
    ];

    for (i, (title, width)) in columns.iter().enumerate() {
        let renderer = CellRendererText::new();
        let column = TreeViewColumn::new();
        column.set_title(title);
        column.pack_start(&renderer, true);
        column.add_attribute(&renderer, "text", i as i32);
        column.set_fixed_width(*width);
        treeview.append_column(&column);
    }

    treeview
}

// Функция для форматирования числа
fn format_amount(value: f64) -> String {
    if value.abs() < 1e-9 {
        // Если значение ноль, возвращаем "0.0"
        "0.0".to_string()
    } else if value.abs() < 1e-9 {
        // Если значение слишком мало, выводим в экспоненциальной форме
        format!("{:.1e}", value)
    } else {
        // Получаем строковое представление значения
        let s = format!("{}", value);

        // Проверяем, есть ли дробная часть
        if let Some(pos) = s.find('.') {
            let integer_part = &s[..pos];
            let fractional_part = &s[pos + 1..];

            // Если дробная часть меньше двух цифр, добавляем нули
            if fractional_part.is_empty() {
                format!("{}.{:0<2}", integer_part, "00")
            } else {
                format!("{}.{fractional_part}", integer_part)
            }
        } else {
            // Если дробной части нет, возвращаем целое число с ".00"
            format!("{}.0", s)
        }
    }
}

// Функция для обновления таблицы на основе JSON данных
fn update_coin_table(treeview: &TreeView, json_data: &Value, label: Rc<RefCell<Label>>) {
    // Очищаем существующие данные в таблице
    let model = TreeStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type()]);
    treeview.set_model(Some(&model));
//    println!("Updating coin table with JSON data: {:?}", json_data);
    let mut row_count = 0;

//    println!("table: {:?}", model);
    // Проверяем, есть ли данные в JSON
    if let Some(items) = json_data["data"]["current_fungible_asset_balances"].as_array() {
//        println!("Items: {:?}", items);
        for item in items {
//            println!("Item: {:?}", item);
            // Проверяем наличие необходимых полей
            if let (Some(metadata), Some(name)) = (item["metadata"].as_object(), item["metadata"]["name"].as_str()) {
                // Извлекаем необходимые значения
                let amount = item["amount"].as_u64().unwrap_or(0);
                let decimals = metadata["decimals"].as_u64().unwrap_or(0);
                let symbol = metadata["symbol"].as_str().unwrap_or("N/A");
                let standard = metadata["token_standard"].as_str().unwrap_or("N/A");
                let asset_type = item["asset_type"].as_str().unwrap_or("N/A");

                // Рассчитываем количество с учетом decimals, деля amount на 10^decimals
                let amount_value = amount as f64 / 10f64.powi(decimals as i32);

                // Форматируем значение amount
                let formatted_amount = format_amount(amount_value);

                // Добавляем строку в таблицу
                model.insert_with_values(None, None, &[
                    (0, &name),                                // Ссылаемся на name
                    (1, &formatted_amount),                     // Используем отформатированное значение
                    (2, &symbol),                              // Ссылаемся на symbol
                    (3, &standard),                            // Ссылаемся на standard
                    (4, &format!("{}", decimals)),             // Форматируем decimals
                    (5, &asset_type),                          // Ссылаемся на asset_type
                ]);
                row_count += 1;
                println!("Name: {}, Amount: {}, Symbol: {}", name, formatted_amount, symbol);
            } else {
                // Если metadata или name нет, пропускаем вставку
                println!("Пропускаем элемент: {:?}", item);
            }
        }
    } else {
        println!("Данные не найдены или структура неверная");
    }
    let label_borrowed = label.borrow_mut();
    label_borrowed.set_text(&format!("Lines: {}", row_count));
    println!("Кол-во строк после функции: {:?}", label_borrowed.text());
}

// Функция для создания таблицы для NFT
fn create_nft_table() -> TreeView {
    let treeview = TreeView::new();

    let columns = vec![
        ("Name", 350),
        ("Collection", 400),
    ];
    for (i, (title, width)) in columns.iter().enumerate() {
        let renderer = CellRendererText::new();
        let column = TreeViewColumn::new();
        column.set_title(title);
        column.pack_start(&renderer, true);
        column.add_attribute(&renderer, "text", i as i32);
        column.set_fixed_width(*width);
        treeview.append_column(&column);
    }
    treeview
}

// Функция для обновления данных в таблице NFT
fn update_nft_table(label: Rc<RefCell<Label>>, nft_scroll: &ScrolledWindow, data: Vec<(std::string::String, std::string::String)>) {

    if !data.iter().any(|(_, collection)| collection == "Clockwork OG") {
        eprintln!("Ошибка: Коллекция 'Clockwork OG' не найдена в данных.");
        let dialog = MessageDialog::new(None::<&Window>, DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, "Ошибка: NFT-коллекция 'Clockwork OG' не найдена в данных.");
        dialog.connect_response(|dialog, _| dialog.close());
        dialog.show();
        return; // Прекращаем выполнение функции
    }

    let mut row_count = 0;

    let store = TreeStore::new(&[
        String::static_type(),
        String::static_type(),
    ]);

    for (name, collection) in data {
        store.insert_with_values(None, None, &[
            (0, &name),
            (1, &collection),
        ]);
        row_count += 1;
        println!("Name: {}, Collection: {}", name, collection);
    }

    let nft_table: TreeView = nft_scroll.child().unwrap().downcast().unwrap();
    nft_table.set_model(Some(&store));

    // Обновляем текст метки
    let label_borrowed = label.borrow();
    label_borrowed.set_text(&format!("Lines: {}", row_count));
    println!("Кол-во строк после функции: {:?}", label_borrowed.text());
}

fn clear_nft_table(nft_scroll: &ScrolledWindow) {
    let nft_table: TreeView = nft_scroll.child().unwrap().downcast().unwrap();
    if let Some(model) = nft_table.model() {
        // Привязываем модель к TreeStore
        let store: TreeStore = model.downcast().unwrap();
        store.clear(); // Очищаем TreeStore
    }
}

// Функция для обработки нажатия на кнопку About
fn on_about_button_clicked() {

    // Флаг премиум
    let premium_flag = Rc::new(RefCell::new(false));
    let theme_flag = Rc::new(RefCell::new(0));
    
    match config_load() {
        Ok(config) => {
            if config.premium.as_deref() == Some("jFg{s;QdsF#45#)&@e22./#d") {
                *premium_flag.borrow_mut() = true;
//              println!("premium yes: {}", config.premium.as_ref().unwrap());
              println!("Premium flag is enabled");
                if config.theme.as_deref() == Some("1") {
                  println!("theme 1");
                    *theme_flag.borrow_mut() = 1;
                } else if config.theme.as_deref() == Some("2") {
                    *theme_flag.borrow_mut() = 2;
                } else {
                    println!("theme other");
                }
            } else {
              println!("premium no");
                *premium_flag.borrow_mut() = false;
            }
        },
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            *premium_flag.borrow_mut() = false;
        },
    }
    let theme_value = *theme_flag.borrow();
    // Создаем всплывающее окно и оборачиваем его в Rc
    let about_window = Rc::new(Window::new());
    about_window.set_title(Some("About"));
    about_window.set_default_size(300, 200);

    println!("Current theme flag value inside activate: {}", theme_value);
    // Проверяем значение theme_value
    if theme_value == 1 {
        // Создаем CSS для окна
        let css_provider = CssProvider::new();

        // Загружаем данные CSS как строку
        css_provider.load_from_data("window { background-color: black; }");

        // Применяем CSS к окну
        let style_context = about_window.style_context();
        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    } else if theme_value == 2 {
        let root_path = Path::new(ROOT_DIR);
//        println!("Корневой каталог: {:?}", root_path);
        // Формируем путь к изображению
        let background_path = root_path.join("background.png");
        // Создаем CSS для фонового изображения
        let css_provider = CssProvider::new();
        let css = format!(
            "window {{ background: repeat center/cover url('file://{}'); background-size: auto; }}",
            background_path.to_str().unwrap()
        );
        css_provider.load_from_data(&css);
        let style_context = about_window.style_context();
        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    // Создаем вертикальный контейнер для размещения виджетов
    let frame = Box::new(Orientation::Vertical, 10);

    // Создаем текстовое виджет
    let text_view = TextView::new();
    text_view.set_wrap_mode(gtk4::WrapMode::Word);
    text_view.set_editable(false); // Делаем текстовое виджет только для чтения
    text_view.set_size_request(380, 200); // Установка размера текстового виджета

    text_view.set_margin_top(10);
    text_view.set_margin_start(10);
    text_view.set_margin_end(10);
    frame.set_margin_bottom(10);

    // Создаем текстовый буфер с передачей None
    let buffer = TextBuffer::new(None); // Передаем None в качестве аргумента
    buffer.set_text("Разработано Aptos Clockwork Community.\n\
    Для FreeBSD Operating System\n\
    и Aptos Blockchain на сети mainnet в 2024 году.\n\n\
    Официальные ссылки проекта:\n\
    X: https://x.com/aptos_cw\n\
    TG: https://t.me/aptos_clockwork\n\
    GIT: https://github.com/Clockwork6400/aptos_clockwork.git\n\
    Matrix: https://matrix.to/#/!fyLWLLXmXjUTKnzmCS:matrix.org?via=matrix.org\n");

    text_view.set_buffer(Some(&buffer)); // Устанавливаем буфер в TextView

    // Создаем скроллбар
    let scrolled_window = ScrolledWindow::new(); // Удаляем аргументы
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_vexpand(true);

    // Добавляем скроллбар в контейнер
    frame.append(&scrolled_window);

    // Создаем кнопку для закрытия окна
    let close_button = Button::with_label("Close");

    // Клонируем Rc для использования в замыкании
    let about_window_clone = Rc::clone(&about_window);
    close_button.connect_clicked(move |_| {
        about_window_clone.close(); // Закрываем окно при нажатии
    });

    // Добавляем кнопку закрытия в контейнер
    frame.append(&close_button);

    about_window.set_child(Some(&frame)); // Устанавливаем контейнер как содержимое окна
    about_window.show(); // Отображаем окно
}

fn show_installation_dialog(file_path: &str) {
//    eprintln!(
//        "Найдите в файле {} следующее:\n\n# Allow members of group operator to cat things to the speaker\n#own    speaker root:operator\n#perm    speaker 0660\n\nРасскомментируйте и замените на 0666. Это позволит всем обращаться к speaker, а не только пользователю root. Должно получится следующее:\n\n# Allow members of group operator to cat things to the speaker\nown speaker 0666\nperm     speaker 0666",
//        file_path
//    );
    let dialog = MessageDialog::new(
        None::<&Window>,
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        &format!(
            "Найдите в файле {} следующее:\n\n# Allow members of group operator to cat things to the speaker\n#own      speaker root:operator\n#perm    speaker 0660\n\nРасскомментируйте и замените на 0666. Это позволит всем обращаться к speaker, а не только пользователю root. Должно получится следующее:\n\n# Allow members of group operator to cat things to the speaker\nown     speaker 0666\nperm      speaker 0666",
            file_path
        ),
    );
    dialog.set_title(Some("Instalation (III)"));
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
    return;
}



