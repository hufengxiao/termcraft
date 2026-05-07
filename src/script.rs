#![allow(dead_code)]
use mlua::prelude::*;

/// Lua scripting engine for in-game automation
pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let lua = Lua::new();

        // Register basic API functions
        lua.globals().set("print", lua.create_function(|_, msg: String| {
            println!("[Lua] {}", msg);
            Ok(())
        }).unwrap()).unwrap();

        // Block manipulation API
        lua.globals().set("set_block", lua.create_function(|_, (x, y, z, block_name): (i32, i32, i32, String)| {
            // This will be connected to the world via a closure later
            println!("[Lua] set_block({}, {}, {}, {})", x, y, z, block_name);
            Ok(())
        }).unwrap()).unwrap();

        // Player position API
        lua.globals().set("get_player_pos", lua.create_function(|_, _: ()| {
            // Placeholder - will be connected at runtime
            Ok((0.0, 0.0, 0.0))
        }).unwrap()).unwrap();

        // Redstone signal API
        lua.globals().set("get_signal", lua.create_function(|_, (x, y, z): (i32, i32, i32)| {
            println!("[Lua] get_signal({}, {}, {})", x, y, z);
            Ok(0u8)
        }).unwrap()).unwrap();

        // Chat/message API
        lua.globals().set("chat", lua.create_function(|_, msg: String| {
            println!("[Chat] {}", msg);
            Ok(())
        }).unwrap()).unwrap();

        // Math helpers
        lua.globals().set("sin", lua.create_function(|_, x: f64| Ok(x.sin())).unwrap()).unwrap();
        lua.globals().set("cos", lua.create_function(|_, x: f64| Ok(x.cos())).unwrap()).unwrap();
        lua.globals().set("sqrt", lua.create_function(|_, x: f64| Ok(x.sqrt())).unwrap()).unwrap();

        Self { lua }
    }

    /// Execute a Lua script string
    pub fn exec(&self, code: &str) -> Result<String, String> {
        match self.lua.load(code).exec() {
            Ok(()) => Ok("OK".to_string()),
            Err(e) => Err(format!("Lua error: {}", e)),
        }
    }

    /// Execute a Lua file
    pub fn exec_file(&self, path: &str) -> Result<String, String> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| format!("File read error: {e}"))?;
        self.exec(&code)
    }

    /// Call a named function with arguments
    pub fn call_function(&self, name: &str, args: &[f64]) -> Result<f64, String> {
        let func: LuaFunction = self.lua.globals().get(name)
            .map_err(|e| format!("Function '{}' not found: {}", name, e))?;

        let lua_args: Vec<LuaValue> = args.iter()
            .map(|&a| LuaValue::Number(a))
            .collect();

        func.call::<LuaValue>(lua_args)
            .map_err(|e| format!("Call error: {}", e))
            .and_then(|v| match v {
                LuaValue::Number(n) => Ok(n),
                _ => Err("Function did not return a number".to_string()),
            })
    }

    /// Register a Rust function that can be called from Lua
    pub fn register_function<F>(&self, name: &str, func: F)
    where
        F: Fn(&Lua, mlua::Value) -> LuaResult<mlua::Value> + 'static,
    {
        if let Ok(f) = self.lua.create_function(func) {
            let _ = self.lua.globals().set(name, f);
        }
    }
}
