use std::collections::HashMap;
use std::fmt::Debug;

use leptos::ev::Event;
use leptos::prelude::{event_target_value, Get, Memo, RwSignal, Signal, Update};

use validator::Validate;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, SubmitEvent};

pub trait FormStruct: Clone + Debug + Validate {
    fn get(&self, name: &str) -> Option<String>;
    fn set(&mut self, name: &str, value: &str);
}

#[derive(Clone)]
pub struct Form<T: Clone + Default + FormStruct + Send + Sync + 'static> {
    values: RwSignal<T>,
    errors: RwSignal<HashMap<String, Option<String>>>,
}

impl<T: Clone + Default + FormStruct + Send + Sync + 'static> Form<T> {
    pub fn new() -> Form<T> {
        let values: RwSignal<T> = RwSignal::new(Default::default());
        let errors = RwSignal::new(HashMap::new());

        Self { values, errors }
    }

    pub fn with_initial_values(values: T) -> Form<T> {
        let values: RwSignal<T> = RwSignal::new(values);
        let errors = RwSignal::new(HashMap::new());

        Self { values, errors }
    }

    pub fn value(&self, field: &str) -> Signal<String> {
        let field = field.to_string();
        let values = self.values;

        Memo::new(move |_| values.get().get(&field).unwrap_or_default()).into()
    }

    pub fn value_opt(&self, field: &str) -> Signal<Option<String>> {
        let field = field.to_string();
        let values = self.values;

        Memo::new(move |_| values.get().get(&field)).into()
    }

    pub fn set_field_value(&self, field: &str, value: Option<String>) {
        let values = self.values;

        values.update(|values| {
            values.set(field, &value.unwrap_or_default());
        });
    }

    pub fn error(&self, field: &str) -> Signal<Option<String>> {
        let field = field.to_string();
        let errors = self.errors;

        Memo::new(move |_| errors.get().get(&field).cloned().flatten()).into()
    }

    /// Input Handler for Form Inputs of type [`HtmlInputElement`]
    pub fn handle_input(&self) -> impl Fn(Event) + Copy + 'static {
        let values = self.values;

        move |ev: Event| {
            if let Some(target) = ev.target() {
                if let Ok(el) = target.dyn_into::<HtmlInputElement>() {
                    let name = el.name();

                    values.update(|values| {
                        let value = event_target_value(&ev);
                        values.set(&name, &value);
                    });
                }
            }
        }
    }

    pub fn handle_submit<F: Fn(T)>(&self, cb: F) -> impl Fn(SubmitEvent) {
        let errors = self.errors;
        let values = self.values;

        move |ev| {
            ev.prevent_default();

            if let Err(validation_err) = values.get().validate() {
                validation_err
                    .field_errors()
                    .iter()
                    .for_each(|(field, f_errors)| {
                        f_errors.iter().for_each(|err| {
                            errors.update(|e| {
                                e.insert(
                                    field.to_string(),
                                    Some(
                                        err.message
                                            .clone()
                                            .map(|m| m.to_string())
                                            .unwrap_or_default(),
                                    ),
                                );
                            });
                        });
                    });

                return;
            }

            cb(values.get());
        }
    }
}
