use crate::error::Result;

#[cfg(feature = "lua-hooks")]
pub struct HookExecutor {
    lua: mlua::Lua,
}

#[cfg(feature = "lua-hooks")]
impl HookExecutor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            lua: mlua::Lua::new(),
        })
    }

    pub fn execute(&self, script: &str) -> Result<()> {
        self.lua.load(script).exec()?;
        Ok(())
    }
}

#[cfg(not(feature = "lua-hooks"))]
pub struct HookExecutor;

#[cfg(not(feature = "lua-hooks"))]
impl HookExecutor {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn execute(&self, _script: &str) -> Result<()> {
        Ok(())
    }
}
