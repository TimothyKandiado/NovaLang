use crate::object::{NativeFunction, NovaObject};

pub fn common_native_functions() -> Vec<NativeFunction> {
    vec![
        hello_native(),
        println_native(),
        print_native(),
        time_native(),
    ]
}

pub fn hello_native() -> NativeFunction {
    let function = |_: Vec<NovaObject>| -> Result<NovaObject, String> {
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
        function,
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
        function,
    }
}

pub fn time_native() -> NativeFunction {
    let function = |arguments: Vec<NovaObject>| -> Result<NovaObject, String> {
        if arguments.len() != 1 {
            return Err(format!(
                " Incorrect number of arguments for 'time()', {} needed while {} provided",
                1,
                arguments.len()
            ));
        }

        let argument = &arguments[0];
        if !argument.is_string() {
            return Err("Function 'time()' requires a string argument".to_string());
        }

        let argument = argument.to_string();

        match argument.as_str() {
            "milli" => {
                let epoch = chrono::Utc::now().timestamp_millis();
                #[cfg(feature = "debug")]
                println!("epoch = {}", epoch);

                Ok(NovaObject::Int64(epoch as i64))
            }

            "micro" => {
                let epoch = chrono::Utc::now().timestamp_micros();
                #[cfg(feature = "debug")]
                println!("epoch = {}", epoch);

                Ok(NovaObject::Int64(epoch as i64))
            }

            "sec" => {
                let epoch = chrono::Utc::now().timestamp();
                #[cfg(feature = "debug")]
                println!("epoch = {}", epoch);

                Ok(NovaObject::Int64(epoch as i64))
            }

            "nano" => {
                let epoch = chrono::Utc::now().timestamp_nanos_opt().unwrap();
                #[cfg(feature = "debug")]
                println!("epoch = {}", epoch);

                Ok(NovaObject::Int64(epoch as i64))
            }

            _ => Err(format!("Unknown option: {}", argument)),
        }
    };

    NativeFunction {
        name: "time".to_string(),
        function,
    }
}
