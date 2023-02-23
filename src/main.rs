use boa_engine::{
    builtins::JsArgs,
    class::{Class, ClassBuilder},
    object::JsArray,
    property::Attribute,
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize)]
struct Person {
    name: String,
    age: u8,
}

impl Person {
    fn say_hello(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| context.construct_type_error("`this` is not a `Person` object"))?;

        println!("Hello {}-year-old {}!", this.age, this.name);

        Ok(JsValue::undefined())
    }
}

impl Class for Person {
    const NAME: &'static str = "Person";
    const LENGTH: usize = 2;

    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let age = args.get_or_undefined(1).to_u32(context)?;

        if !(0..=150).contains(&age) {
            context
                .throw_range_error(format!("invalid age `${age}`. Must be between 0 and 150"))?;
        }

        let age = u8::try_from(age).expect("we already checked that it was in range");

        let person = Person {
            name: name.to_string(),
            age,
        };

        Ok(person)
    }

    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method("say_hello", 0, Self::say_hello);

        Ok(())
    }
}

fn say_hello(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0);

    if name.is_undefined() {
        println!("Hello World!");
    } else {
        println!("Hello {}!", name.to_string(context)?);
    }

    Ok(JsValue::undefined())
}

fn reverse_append(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let arr = args
        .get_or_undefined(0)
        .as_object()
        .ok_or_else(|| context.construct_type_error("argument must be an array"))?;

    let arr = JsArray::from_object(arr.clone(), context)?;

    let reverse = arr.reverse(context)?;
    reverse.push("My Project", context)?;

    let global_object = context.global_object().clone();
    let version = global_object
        .get("MY_PROJECT_VERSION", context)
        .unwrap_or_default();

    reverse.push(version, context)?;

    Ok((*reverse).clone().into())
}

fn main() {
    /* let js_code = r#"
    let person = new Person("John", 28);
    person.say_hello();
    "#; */

    let js_code = r#"
    let arr = ['a', 2, 5.4, "Hello"];
    reverseAppend(arr);
    "#;

    let mut context = Context::default();

    context.register_global_builtin_function("say_hello", 1, say_hello);
    // attribute はプロパティ属性が書き込み可能かどうかを示す
    context.register_global_property("MY_PROJECT_VERSION", "1.0.0", Attribute::all());
    context
        .register_global_class::<Person>()
        .expect("could not register class");
    context.register_global_builtin_function("reverseAppend", 1, reverse_append);

    match context.eval(js_code) {
        Ok(res) => {
            println!("{}", res.to_string(&mut context).unwrap());
        }
        Err(e) => {
            eprintln!("Uncaught {}", e.display());
        }
    }
}
