pub mod card;
pub mod routes;
pub mod service;

pub use card::{append_annotation, AnnotationEntry, CardStatus, CorpusCard};
pub use routes::routes;
pub use service::{
    annotate_card, create_card_from_instance, get_card, list_cards, update_card_status,
};

#[cfg(test)]
mod tests;
