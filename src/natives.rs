use crate::object::{NativeFunction, NovaObject};

pub fn common_native_functions() -> Vec<NativeFunction> {
    vec![hello_native(), println_native(), print_native()]
}

pub fn hello_native() -> NativeFunction {
    let function = |_: Vec::<NovaObject>| -> Result<NovaObject, String> {
        println!("Hello Native Function!!!");
        Ok(NovaObject::None)
    };

    NativeFunction {
        name: "Hello".to_string(),
        function,
    }
}

pub fn print_native() -> NativeFunction {
    let function = |arguments: Vec<NovaObject>| -> Result<NovaObject, String> {
        for argument in arguments {
            print!("{}", argument);
        }

        Ok(NovaObject::None)
    };

    NativeFunction {
        name: "print".to_string(),
        function
    }
}

pub fn println_native() -> NativeFunction {
    let function = |arguments: Vec<NovaObject>| -> Result<NovaObject, String> {
        for argument in arguments {
            print!("{}", argument);
        }
        println!();

        Ok(NovaObject::None)
    };

    NativeFunction {
        name: "println".to_string(),
        function
    }
}