use std::{env::args, fs::read_to_string};

use regex::Regex;

fn main() {
    if let Some(path) = args().collect::<Vec<String>>().get(1) {
        if let Ok(code) = read_to_string(path) {
            let mut openmind = Core { stack: vec![] };
            openmind.eval(code);
        } else {
            eprintln!("エラー！ファイルが開けませんでした");
        };
    } else {
        println!("日本語プログラミング言語OpenMind");
    }
}

#[derive(Clone, Debug)]
enum Type {
    Number(f64),
    String(String),
}

impl Type {
    fn get_number(&self) -> f64 {
        match self {
            Type::Number(i) => i.to_owned(),
            Type::String(s) => s.trim().parse().unwrap_or_default(),
        }
    }

    fn get_string(&self) -> String {
        match self {
            Type::Number(i) => i.to_string(),
            Type::String(s) => s.to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
struct Core {
    stack: Vec<Type>,
}

impl Core {
    fn tokenize(soruce: String) -> Option<Vec<String>> {
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
                    if if let Ok(i) = Regex::new(
                        r"[あ-ん]|[ア-ン]|[a-z]|[A-Z]| |\n|\t|\r|　|,|、|。|\.|ー|〜|!|！",
                    ) {
                        i
                    } else {
                        return None;
                    }
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
        Some(tokens)
    }

    fn eval(&mut self, soruce: String) -> Option<()> {
        let tokens = Core::tokenize(soruce)?;
        dbg!(&tokens);

        for token in tokens.iter() {
            let token = token.trim().to_string();
            if token.is_empty() {
                continue;
            }

            if let Ok(i) = token.parse::<f64>() {
                self.stack.push(Type::Number(i))
            } else if token.starts_with("「") && token.ends_with("」") {
                let mut token = token.clone();
                token.remove(token.find("「")?);
                token.remove(token.rfind("」")?);
                self.stack.push(Type::String(token))
            } else {
                match token.as_str() {
                    "表示" => {
                        println!("{}", self.stack.pop()?.get_string());
                    }
                    "結合" => {
                        let str2 = self.stack.pop()?.get_string();
                        let str1 = self.stack.pop()?.get_string();
                        self.stack.push(Type::String(str1 + &str2));
                    }
                    "足" => {
                        let num2 = self.stack.pop()?.get_number();
                        let num1 = self.stack.pop()?.get_number();
                        self.stack.push(Type::Number(num1 + num2));
                    }
                    "引" => {
                        let num2 = self.stack.pop()?.get_number();
                        let num1 = self.stack.pop()?.get_number();
                        self.stack.push(Type::Number(num1 - num2));
                    }
                    "掛" => {
                        let num2 = self.stack.pop()?.get_number();
                        let num1 = self.stack.pop()?.get_number();
                        self.stack.push(Type::Number(num1 * num2));
                    }
                    "割" => {
                        let num2 = self.stack.pop()?.get_number();
                        let num1 = self.stack.pop()?.get_number();
                        self.stack.push(Type::Number(num1 / num2));
                    }
                    _ => return None,
                }
            }
        }
        Some(())
    }
}
