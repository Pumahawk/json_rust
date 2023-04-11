use crate::objects as json;

pub trait Automa {
    type Input;
    type Output;
    fn can_start(&self, input: Self::Input) -> bool;
    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String>;

    fn process(&self, first: Self::Input, iter: &mut  dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String> {
        let mut iter = std::iter::once(first).chain(iter);
        self.start(&mut iter)
    }

}

enum StrAtm {
    N1, N2,
}

struct StrAutoma {   
}

impl StrAutoma {
    fn new() -> StrAutoma {
        StrAutoma {
        }
    }
}

impl Automa for StrAutoma {
    type Input = char;
    type Output = String;

    fn can_start(&self, input: Self::Input) -> bool {
        input == '"'
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String> {
        let mut status = StrAtm::N1;
        let mut chars = Vec::new();
        for c in iter {
            match status {
                StrAtm::N1 => {
                    match c {
                        '"' => {
                            status = StrAtm::N2;
                        },
                        _ => return Err(String::from("invalid")),
                    }
                },
                StrAtm::N2 => {
                    match c {
                        '"' => return Ok(chars.iter().collect()),
                        c => chars.push(c),
                    }
                },
            }
        }
        Err(String::from("unable to retrieve str"))
    }
}

struct StringAutoma {
    value: String,
}

impl StringAutoma {
}

impl std::convert::From<String> for StringAutoma {
    fn from(value: String) -> Self {
        StringAutoma {
            value,
        }
    }
}

impl std::convert::From<&str> for StringAutoma {
    fn from(value: &str) -> Self {
        StringAutoma::from(String::from(value))
    }
}

impl Automa for StringAutoma {
    type Input = char;
    type Output = ();
    fn can_start(&self, input: Self::Input) -> bool {
        if let Some(v) =self.value.chars().next() { v == input} else { false }
    }
    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String> {
        let mut i = 0;
        while let Some(c) = iter.next() {
            if self.value.as_bytes()[i] == c as u8 {
                i += 1;
                if i == self.value.len() {
                    return Ok(());
                }
            } else {
                return Err(String::from("Erro String match. Char not equal"));
            }
        }
        Err(String::from("Invalid StringAutoma parse."))
    }
}

enum NumberAtm {
    N1,
    N2,
    N3,
}

struct NumberAutoma;

impl NumberAutoma {
    pub fn new() -> NumberAutoma {
        NumberAutoma {
        }
    }
}

impl Automa for NumberAutoma {
    type Input=char;
    type Output=(json::NumberJson, Option<Self::Input>);

    fn can_start(&self, input: Self::Input) -> bool {
        is_number(input)
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String> {
        let mut status = NumberAtm::N1;
        let mut number_chars = Vec::new();
        while let Some(c) = iter.next() {
            match status {
                NumberAtm::N1 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                            status = NumberAtm::N2;
                        }
                        _ => return Err(String::from("Unable to read first number")),
                    }
                },
                NumberAtm::N2 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                        },
                        '.' => {
                            number_chars.push(c);
                            status = NumberAtm::N3;
                        },
                        c => {
                            return Ok((json::number(number_chars.iter().collect::<String>().parse().unwrap()), Some(c)));
                        }
                    }
                },
                NumberAtm::N3 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                        },
                        c => {
                            return Ok((json::number(number_chars.iter().collect::<String>().parse().unwrap()), Some(c)));
                        }
                    }
                },
            }
        }
        Err(String::from("Unable to retrieve number"))
    }
}

enum JsonAtm {
    N1, N2, N3, N4, N5,
}

struct JsonAutoma {
}

impl JsonAutoma {
    fn new() -> JsonAutoma {
        JsonAutoma {
        }
    }
}

impl Automa for JsonAutoma {
    type Input = char;
    type Output = json::ObjectJson;

    fn can_start(&self, input: Self::Input) -> bool {
        input == '{'
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> Result<Self::Output, String> {
        let mut iter: Box<dyn Iterator<Item=char>> = Box::new(std::iter::empty().chain(iter));
        let mut status = JsonAtm::N1;
        let mut key = None;
        let mut json_object = json::object();
        while let Some(c) = iter.next() {
            match status {
                JsonAtm::N1 => {
                    match c {
                        '{' => {
                            status = JsonAtm::N2;
                        },
                        _ => return Err(String::from("invalid from node: N1"))
                    }
                },
                JsonAtm::N2 => {
                    let str_automa = StrAutoma::new();
                    match c {
                        c if is_space(c) => continue,
                        c if str_automa.can_start(c) => {
                            let result = str_automa.process(c, &mut iter);
                            match result {
                                Ok(k) => {
                                    key = Some(k);
                                }
                                Err(msg) => return Err(msg),
                            }
                            status = JsonAtm::N3;
                        },
                        _ => return Err(String::from("invalid from node: N2"))
                    }
                },
                JsonAtm::N3 => {
                    match c {
                        c if is_space(c) => continue,
                        ':' => {
                            status = JsonAtm::N4;
                        },
                        other => return Err(format!("invalid from node: N3. Value: {other}"))
                    }
                },
                JsonAtm::N4 => {
                    let str_automa = StrAutoma::new();
                    let json_automa = JsonAutoma::new();
                    let null_automa = StringAutoma::from("null");
                    match c {
                        c if is_space(c) => continue,
                        c if json_automa.can_start(c) => {
                            let result = json_automa.process(c, &mut iter);
                            match result {
                                Ok(value) => {
                                    json_object.set(&key.take().unwrap(), value);
                                },
                                Err(msg) => return Err(msg),
                            }
                            status = JsonAtm::N5;
                        },
                        c if null_automa.can_start(c) => {
                            let result = null_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_object.set(&key.take().unwrap(), json::null());
                                    status = JsonAtm::N5;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        c if str_automa.can_start(c) => {
                            let result = str_automa.process(c, &mut iter);
                            match result {
                                Ok(value) => {
                                    json_object.set(&key.take().unwrap(), json::text(value));
                                },
                                Err(msg) => return Err(msg),
                            }
                            status = JsonAtm::N5;
                        },
                        _ => return Err(String::from("invalid from node: N4"))
                    }
                },
                JsonAtm::N5 => {
                    match c {
                        c if is_space(c) => continue,
                        '}' => return Ok(json_object),
                        ',' => {
                            status = JsonAtm::N2;
                        }
                        _ => return Err(String::from("invalid from node: N5"))
                    }
                },
            }
        }
        Err(String::from("invalid json automa"))
    }
}

fn is_space(c: char) -> bool {
    match c {
        ' ' | '\t' | '\n' | '\r' => true,
        _ => false,
    }
}

fn is_number(c: char) -> bool {
    c >= '0' && c <= '9'
}

#[cfg(test)]
mod test {

    use crate::objects::*;
    use crate::automa::*;

    #[test]
    fn is_space_test() {
        assert!(is_space(' '));
        assert!(is_space('\t'));
        assert!(is_space('\r'));
        assert!(is_space('\n'));
    }

    #[test]
    fn str_automa() {
        let str_automa = StrAutoma::new();
        let input = String::from("\"input_automa\"");

        if let Ok(msg) = str_automa.start(&mut input.chars()) {
            assert_eq!("input_automa", msg);
        } else {
            assert!(false);
        }

        let input = String::from("\"input_automa\": 1234");
        let mut iter = input.chars();
        let result = str_automa.start(&mut iter).unwrap();
        let rest: String = iter.collect();

        assert_eq!("input_automa", result);
        assert_eq!("input_automa", result);
        assert_eq!(": 1234", rest);
        
    }

    #[test]
    fn json_automa() {
        let json_autom = JsonAutoma::new();
        let input = String::from("{\"key\":\"input_automa\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(mut json_object) => {
                assert_eq!("input_automa", if let TypeJson::Text(msg) = json_object.get("key").unwrap() {msg} else {"none"});
            },
            Err(msg) => {
                assert_eq!("ok", msg);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\":\"input_automa_2\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(mut json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("input_automa_2", if let TypeJson::Text(msg) = json_object.get("key2").unwrap() {msg} else {"none"});
            },
            Err(msg) => {
                assert_eq!("ok", msg);
            }
        }

        let input = String::from("{  \"key1\" \t : \n \"input_automa_1\"  \t,\r \"key2\":\"input_automa_2\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(mut json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("input_automa_2", if let TypeJson::Text(msg) = json_object.get("key2").unwrap() {msg} else {"none"});
            },
            Err(msg) => {
                assert_eq!("ok", msg);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\":{\"key\":\"input_automa\"}}");

        match json_autom.start(&mut input.chars()) {
            Ok(mut json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                let jo = json_object.get("key2").unwrap();
                match jo {
                    TypeJson::Object(v) => {
                        match v.get("key") {
                            Some(TypeJson::Text(msg)) => assert_eq!("input_automa", msg),
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                }
            },
            Err(msg) => {
                assert_eq!("ok", msg);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\": null}");

        match json_autom.start(&mut input.chars()) {
            Ok(mut json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("null", if let TypeJson::Null = json_object.get("key2").unwrap() {"null"} else {"none"});
            },
            Err(msg) => {
                assert_eq!("ok", msg);
            }
        }
    }

    #[test]
    fn string_automa() {
        let input = String::from("null");
        let string_automa = StringAutoma::from("null");

        let mut iter = input.chars();
        match string_automa.start(&mut iter) {
            Ok(_) => {
                assert!(true);
            },
            _ => assert!(false),
        }

        let mut iter = input.chars().chain(std::iter::once('c'));
        match string_automa.start(&mut iter) {
            Ok(_) => {
                assert!(true);
            },
            _ => assert!(false),
        }

        assert_eq!(Some('c'), iter.next());

    }
}