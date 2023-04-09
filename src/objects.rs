
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

    pub fn create(&mut self, key: &str) -> &mut (dyn Json + 'static) {
        self.set(key, ObjectJson::new());
        self.parameters.last_mut().unwrap().1.deref_mut()
    }

    pub fn get(&self, key: &str) -> Option<&(dyn Json + 'static)> {
        self.parameters.iter().find(|el| el.0 == key).map(|el| &*el.1)
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
}

impl Json for ListJson {
    fn json(&mut self) -> TypeJson {
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

#[cfg(test)]
mod tests {
    use crate::objects::*;
    #[test]
    fn edit_objects() {
        let mut root = json();
        let obj1 = json();

        root.set("key1", obj1);
        let obj2 = root.create("key2");
        //obj2.set("key-sub-1", obj1);
    }
}