use std::{fmt::Debug, marker::PhantomData};

use serde::Serialize;

use crate::futurecop::*;

pub trait GetterSetter<T> {
    fn get(&self) -> &T;

    fn set(&mut self, value: T); 
}

#[derive(Clone, Copy, Serialize)]
pub struct VolatileGlobal<T: Debug> {
    address: u32,
    phantom: PhantomData<T>,
}

impl<T: Debug> VolatileGlobal<T> {
    pub const fn new(address: u32) -> Self {
        Self {
            address: address,
            phantom: PhantomData,
        }
    }
}

impl<T: Debug> Debug for VolatileGlobal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.get();

        write!(f, "{:?}", value)
    }
}

impl<T: Debug> GetterSetter<T> for VolatileGlobal<T> {
    fn get(&self) -> &T {
        let value: &T;

        unsafe {
            let raw_value = self.address as *const T;
            value = &*raw_value;
        }

        return value;
    }

    fn set(&mut self, value: T) {
        unsafe {
            let raw_value = self.address as *mut T;
            (*raw_value) = value;
        }
    }
}

#[derive(Serialize)]
pub struct Global<T: Debug> {
    value: T,
}

impl<T: Debug> Global<T> {
    pub fn new(default: T) -> Self {
        Self {
            value: default,
        }
    }
}

impl<T: Debug> GetterSetter<T> for Global<T> {
    fn get(&self) -> &T {
        return &self.value;
    }

    fn set(&mut self, value: T) {
        self.value = value;
    }
}

impl<T: Debug> Debug for Global<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Clone, Copy)]
pub struct SelectedGameMode {
    volatile_value: VolatileGlobal<u32>,
}

impl SelectedGameMode {
    pub const fn new(address: u32) -> Self {
        Self {
            volatile_value: VolatileGlobal::<u32>::new(address),
        }
    }
}

impl GetterSetter<GameMode> for SelectedGameMode {
    fn get(&self) -> &GameMode {
        let raw_value = self.volatile_value.get();

        if *raw_value == 0 {
            return &GameMode::PrecinctAssault;
        }

        return &GameMode::CrimeWar;
    }

    fn set(&mut self, value: GameMode) {
        match value {
            GameMode::CrimeWar => self.volatile_value.set(0),
            GameMode::PrecinctAssault => self.volatile_value.set(1),
        }
    }
}

impl Debug for SelectedGameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &GameMode;

        value = self.get();

        f.debug_struct("SelectedGameMode").field("value", &value).finish()
    }
}