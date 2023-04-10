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
        match self.get(key) {
            Some(TypeJson::Object(obj)) => obj,
            _ => unreachable!(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<TypeJson> {
        self.parameters
            .iter_mut()
            .find(|el| el.0 == key)
            .map(|el| el.1.deref_mut())
            .map(|el|el.json())
    }
    
    pub fn as_object(&mut self, key: &str) -> Option<&mut ObjectJson> {
        match self.get(key) {
            Some(TypeJson::Object(obj)) => Some(obj),
            _ => None,
        }
    }

    pub fn as_text(&mut self, key: &str) -> Option<&mut str> {
        match self.get(key) {
            Some(TypeJson::Text(msg)) => Some(msg),
            _ => None,
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

    pub fn get(&mut self, index: usize) -> Option<TypeJson> {
        self.list.get_mut(index).map(|b|&mut **b).map(|b|b.json())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=TypeJson> {
        self.list.iter_mut().map(|b|&mut **b).map(|b|b.json())
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

        match root.get("key1") {
            Some(TypeJson::Object(obj)) => {
                match obj.get("field") {
                    Some(TypeJson::Text(msg)) => assert_eq!(msg, "hello World"),
                    _ => assert!(false),
                }
            },
            _ => assert!(false),
        }

        match root
            .as_object("key2")
            .and_then(|obj|obj.as_object("key-sub-1"))
            .and_then(|obj|obj.get("field")) {
                Some(TypeJson::Text(msg)) => assert_eq!(msg, "hello World 2"),
                _ => assert!(false), 
            }
        
        assert_eq!("hello World 2", root
            .as_object("key2")
            .and_then(|obj|obj.as_object("key-sub-1"))
            .and_then(|obj|obj.as_text("field"))
            .unwrap()
        );

    }
    #[test]
    fn edit_list() {
        let mut root = array();
        let mut obj1 = object();
        obj1.set("key1", text("value1"));

        root.add(obj1);
        root.add(text("second"));
        assert!(root.get(0).is_some());

        let mut iter = root.iter_mut();
        match iter.next() {
            Some(TypeJson::Object(obj)) => match obj.get("key1") {
                Some(TypeJson::Text(msg)) => assert_eq!(msg, "value1"),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }

        match iter.next() {
            Some(TypeJson::Text(msg)) => assert_eq!(msg, "second"),
            _ => assert!(false),
        }
    }
    #[test]
    fn edit_text() {
        let mut text = text("message");
        assert_eq!(&*text, "message");
        *text = String::from("message edit");
        assert_eq!(&*text, "message edit");
    }
}