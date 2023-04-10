use std::ops::Deref;
use std::ops::DerefMut;

pub enum TypeJson<'a> {
    Object(&'a mut ObjectJson),
    List(&'a mut ListJson),
    Text(&'a mut str),
    Number(&'a mut f32),
    Null,
}

pub trait Json {
    fn json(&mut self) -> TypeJson;
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

    pub fn create(&mut self, key: &str) -> &mut ObjectJson {
        self.set(key, ObjectJson::new());
        if let TypeJson::Object(obj) = self.parameters.last_mut().unwrap().1.deref_mut().json() {
            obj
        } else {
            unreachable!();
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&mut (dyn Json + 'static)> {
        self.parameters.iter_mut().find(|el| el.0 == key).map(|el| el.1.deref_mut())
    }
    
    pub fn as_object(&mut self, key: &str) -> Option<&mut ObjectJson> {
        if let TypeJson::Object(obj) = self.get(key)?.json() {
            Some(obj)
        } else {
            None
        }
    }
}

impl Json for ObjectJson {
    fn json(&mut self) -> TypeJson {
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

    pub fn add<T: Json + 'static>(&mut self, obj: T) {
        self.list.push(Box::new(obj));
    }

    pub fn get(&mut self, index: usize) -> Option<&mut (dyn Json + 'static)> {
        self.list.get_mut(index).map(|b|&mut **b)
    }
}

impl Json for ListJson {
    fn json(&mut self) -> TypeJson {
        TypeJson::List(self)
    }
}

pub struct NumberJson;

pub struct TextJson {
    value: String,
}

impl TextJson {
    pub fn new(value: String) -> TextJson {
        TextJson {
            value,
        }
    }
}

impl Json for TextJson {
    fn json(&mut self) -> TypeJson {
        TypeJson::Text(&mut self.value)
    }
}

impl Deref for TextJson {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for TextJson {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct NullJson;

pub fn object() -> ObjectJson {
    ObjectJson::new()
}

pub fn array() -> ListJson {
    ListJson::new()
}

pub fn text<T: ToString>(txt: T) -> TextJson {
    TextJson::new(txt.to_string())
}

#[cfg(test)]
mod tests {
    use crate::objects::*;
    #[test]
    fn edit_objects() {
        let mut root = object();
        let mut obj1 = object();
        let mut obj_sub_1 = object();
        
        obj1.set("field", text("hello World"));
        root.set("key1", obj1);
        {
            let obj2 = root.create("key2");
            obj_sub_1.set("field", text("hello World 2"));
            obj2.set("key-sub-1", obj_sub_1);
        }

        match root.get("key1").unwrap().json() {
            TypeJson::Object(obj) => {
                match obj.get("field").unwrap().json() {
                    TypeJson::Text(msg) => assert_eq!(msg, "hello World"),
                    _ => assert!(false),
                }
            },
            _ => assert!(false),
        }

    }
    #[test]
    fn edit_list() {
        let mut root = array();
        let obj1 = object();

        root.add(obj1);
        assert!(root.get(0).is_some());
    }
    #[test]
    fn edit_text() {
        let mut text = text("message");
        assert_eq!(&*text, "message");
        *text = String::from("message edit");
        assert_eq!(&*text, "message edit");
    }
}