use std::mem::size_of;
use windows::core::Result as WinResult;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

pub struct WinModules {
    snap: HANDLE,
    first: bool,
}

impl WinModules {
    pub fn new() -> WinResult<Self> {
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, 0) }?;

        Ok(Self { snap, first: true })
    }
}

impl Iterator for WinModules {
    type Item = MODULEENTRY32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = MODULEENTRY32 {
            dwSize: size_of::<MODULEENTRY32>() as u32,
            ..Default::default()
        };

        let r = match self.first {
            true => {
                self.first = false;

                unsafe { Module32First(self.snap, &mut entry) }
            }
            false => unsafe { Module32Next(self.snap, &mut entry) },
        };

        match r.as_bool() {
            true => Some(entry),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut at_least_one = false;

        for m in WinModules::new().unwrap() {
            at_least_one = true;

            assert!(m.modBaseAddr as usize > 0);
        }

        assert!(at_least_one);
    }
}
