pub enum TypeJson<'a> {
    Object(&'a ObjectJson),
    List(&'a ListJson),
    Text(&'a str),
    Number(&'a f32),
    Null,
}

pub trait Json {
    fn json(&self) -> TypeJson;
}

pub struct ObjectJson {
    parameters: Vec<(String, Box<dyn Json>)>,
}

impl ObjectJson {

    fn new() -> ObjectJson {
        ObjectJson {
            parameters: Vec::new(),
        }
    }

    pub fn set<T: Json + 'static>(&mut self, key: &str, obj: T) {
        self.parameters.push((String::from(key), Box::new(obj)));
    }
}

impl Json for ObjectJson {
    fn json(&self) -> TypeJson {
        TypeJson::Object(self)
    }
}

pub struct ListJson {
    list: Vec<Box<dyn Json>>,
}

impl ListJson {
    pub fn new() -> ListJson {
        ListJson {
            list: Vec::new(),
        }
    }
}

impl Json for ListJson {
    fn json(&self) -> TypeJson {
        TypeJson::List(self)
    }
}

pub struct NumberJson;
pub struct TextJson;
pub struct NullJson;

pub fn json() -> ObjectJson {
    ObjectJson::new()
}

pub fn array() -> ListJson {
    ListJson::new()
}
