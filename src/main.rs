use regex::Regex;
use rustyline::DefaultEditor;
use std::{
    collections::HashMap,
    env::args,
    fs::{read_to_string, File},
    io::Write,
    process::exit,
};

const DELIMITER: [&str; 19] = [
    r"[あ-ん]",
    r"[ア-ン]",
    r"[a-z]",
    r"[A-Z]",
    r"\s",
    r"、",
    r",",
    r"。",
    r"\.",
    r"ー",
    r"\-",
    r"〜",
    r"\~",
    r"！",
    r"!",
    r"＾",
    r"\^",
    r"？",
    r"\?",
];

fn main() {
    let mut openmind = Core {
        stack: vec![],
        memory: HashMap::from([
            ("改行".to_string(), Type::String("\n".to_string())),
            ("空白".to_string(), Type::String(" ".to_string())),
        ]),
        cache: Type::Null,
        backs: 0,
    };

    let args = args().collect::<Vec<String>>();
    if let Some(path) = args.get(1) {
        if let Ok(code) = read_to_string(path) {
            openmind.eval(code);
        } else {
            eprintln!("エラー！ファイルが開けませんでした");
        };
    } else {
        println!("日本語プログラミング言語 かぐや");
        let mut rl = DefaultEditor::new().unwrap();

        loop {
            let mut code = String::new();
            loop {
                let enter = rl.readline("> ").unwrap_or_default().trim().to_string();
                if enter.is_empty() {
                    break;
                }
                code += &format!("{enter} ");
            }

            if !code.is_empty() {
                openmind.eval(code);
                println!(
                    "スタック〔 {} 〕",
                    &openmind
                        .stack
                        .iter()
                        .map(|i| i.get_symbol())
                        .collect::<Vec<String>>()
                        .join(" | ")
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    Function(String),
    Null,
}

impl Type {
    fn get_number(&self) -> f64 {
        match self {
            Type::Number(i) => i.to_owned(),
            Type::String(s) | Type::Function(s) => s.trim().parse().unwrap_or_default(),
            Type::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Type::Null => 0.0,
        }
    }

    fn get_string(&self) -> String {
        match self {
            Type::Number(i) => i.to_string(),
            Type::String(s) | Type::Function(s) => s.to_owned(),
            Type::Bool(b) => if *b { "真" } else { "偽" }.to_string(),
            Type::Null => String::new(),
        }
    }

    fn get_symbol(&self) -> String {
        match self {
            Type::Number(i) => i.to_string(),
            Type::String(s) | Type::Function(s) => format!("「{}」", s),
            Type::Bool(b) => if *b { "真" } else { "偽" }.to_string(),
            Type::Null => "無".to_string(),
        }
    }

    fn get_bool(&self) -> bool {
        match self {
            Type::Number(i) => *i != 0.0,
            Type::String(s) | Type::Function(s) => !s.is_empty(),
            Type::Bool(b) => *b,
            Type::Null => false,
        }
    }
}

#[derive(Clone, Debug)]
struct Core {
    stack: Vec<Type>,
    memory: HashMap<String, Type>,
    backs: usize,
    cache: Type,
}

impl Core {
    fn tokenize(soruce: String) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_parentheses: usize = 0;

        for c in soruce.chars() {
            match c {
                '「' => {
                    in_parentheses += 1;
                    current_token.push(c);
                }
                '」' => {
                    if in_parentheses != 0 {
                        current_token.push(c);
                        in_parentheses -= 1;
                        if in_parentheses == 0 {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    }
                }
                other => {
                    if Regex::new(DELIMITER.join("|").as_str())
                        .unwrap()
                        .is_match(&other.to_string())
                    {
                        if in_parentheses != 0 {
                            current_token.push(c);
                        } else if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    } else {
                        current_token.push(c);
                    }
                }
            }
        }

        if !(in_parentheses != 0 || current_token.is_empty()) {
            tokens.push(current_token);
        }
        tokens
    }

    fn eval(&mut self, soruce: String) {
        let tokens = Core::tokenize(soruce);
        for token in tokens.iter() {
            let token = token.trim().to_string();
            if token.is_empty() {
                continue;
            }

            if let Some(Type::Function(code)) = self.memory.get(&token) {
                self.eval(code.to_string());
            } else if let Some(value) = self.memory.get(&token) {
                self.stack.push(value.to_owned());
            } else if let Ok(i) = token.parse::<f64>() {
                self.stack.push(Type::Number(i))
            } else if Regex::new(r"^[０-９]+(?:\.[０-９]+)?$")
                .unwrap()
                .is_match(&token)
            {
                self.stack.push(Type::Number(
                    token
                        .replace("０", "0")
                        .replace("１", "1")
                        .replace("２", "2")
                        .replace("３", "3")
                        .replace("４", "4")
                        .replace("５", "5")
                        .replace("６", "6")
                        .replace("７", "7")
                        .replace("８", "8")
                        .replace("９", "9")
                        .parse()
                        .unwrap(),
                ))
            } else if token == "真" {
                self.stack.push(Type::Bool(true));
            } else if token == "偽" {
                self.stack.push(Type::Bool(false))
            } else if token == "無" {
                self.stack.push(Type::Null);
            } else if token == "其" {
                self.stack.push(self.cache.clone());
            } else if token.starts_with("「") && token.ends_with("」") {
                let mut token = token.clone();
                token.remove(token.find("「").unwrap());
                token.remove(token.rfind("」").unwrap());
                self.stack.push(Type::String(token))
            } else {
                match token.as_str() {
                    "表示" => {
                        println!("{}", self.pop().get_string());
                    }
                    "入力待" => {
                        let prompt = &self.pop().get_string();
                        self.stack.push(Type::String(
                            DefaultEditor::new()
                                .unwrap()
                                .readline(prompt)
                                .unwrap_or_default(),
                        ))
                    }
                    "結合" => {
                        let str2 = self.pop().get_string();
                        let str1 = self.pop().get_string();
                        self.stack.push(Type::String(str1 + &str2));
                    }
                    "足" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1 + num2));
                    }
                    "引" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1 - num2));
                    }
                    "掛" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1 * num2));
                    }
                    "割" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1 / num2));
                    }
                    "余" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1 % num2));
                    }
                    "乗" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Number(num1.powf(num2)));
                    }
                    "等" => {
                        let str1 = self.pop().get_string();
                        let str2 = self.pop().get_string();
                        self.stack.push(Type::Bool(str1 == str2));
                    }
                    "大" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Bool(num1 > num2));
                    }
                    "小" => {
                        let num2 = self.pop().get_number();
                        let num1 = self.pop().get_number();
                        self.stack.push(Type::Bool(num1 < num2));
                    }
                    "和" => {
                        let bool2 = self.pop().get_bool();
                        let bool1 = self.pop().get_bool();
                        self.stack.push(Type::Bool(bool1 || bool2));
                    }
                    "積" => {
                        let bool2 = self.pop().get_bool();
                        let bool1 = self.pop().get_bool();
                        self.stack.push(Type::Bool(bool1 && bool2));
                    }
                    "否" => {
                        let bool1 = self.pop().get_bool();
                        self.stack.push(Type::Bool(!bool1));
                    }
                    "代入" => {
                        let name = self.pop().get_string();
                        let value = self.pop();
                        self.cache = value.clone();
                        self.memory.insert(name, value);
                    }
                    "初期化" => {
                        let name = self.pop().get_string();
                        let value = self.pop();
                        self.cache = value.clone();
                        if !self.memory.contains_key(&name) {
                            self.memory.insert(name, value);
                        }
                    }
                    "定義" => {
                        let name = self.pop().get_string();
                        let code = self.pop().get_string();
                        self.memory.insert(name, Type::Function(code));
                    }
                    "評価" => {
                        let code = self.pop().get_string();
                        self.eval(code)
                    }
                    "条件分岐" => {
                        let code_false = self.pop().get_string();
                        let code_true = self.pop().get_string();
                        let condition = self.pop().get_bool();
                        if condition {
                            self.eval(code_true)
                        } else {
                            self.eval(code_false)
                        }
                    }
                    "反復" => {
                        let code = self.pop().get_string();
                        let condition = self.pop().get_string();
                        while {
                            self.eval(condition.clone());
                            if self.backs != 0 {
                                self.backs -= 1;
                                return;
                            }
                            self.pop().get_bool()
                        } {
                            self.eval(code.clone());
                            if self.backs != 0 {
                                self.backs -= 1;
                                return;
                            }
                        }
                    }
                    "読" => {
                        let path = self.pop().get_string();
                        self.stack
                            .push(Type::String(read_to_string(path).unwrap_or(String::new())));
                    }
                    "書" => {
                        let path = self.pop().get_string();
                        let value = self.pop().get_string();
                        File::create(path).unwrap().write(value.as_bytes()).unwrap();
                    }
                    "戻" => {
                        self.backs = self.pop().get_number() as usize;
                    }
                    "返" => {
                        let a = self.pop();
                        let b = self.pop();
                        self.stack.push(a);
                        self.stack.push(b);
                    }
                    "写" => {
                        let a = self.pop();
                        self.stack.push(a.clone());
                        self.stack.push(a);
                    }
                    "終了" => exit(0),
                    other => self.stack.push(Type::String(other.to_string())),
                }
            }

            if self.backs != 0 {
                self.backs -= 1;
                return;
            }
        }
    }

    fn pop(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            Type::Null
        }
    }
}
