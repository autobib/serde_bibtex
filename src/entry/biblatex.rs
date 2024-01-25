use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;
use unicase::UniCase;

#[derive(Debug, Deserialize)]
enum EntryType {
    Article,
    Book,
    Booklet,
    InBook,
    InCollection,
    InProceedings,
    Manual,
    MastersThesis,
    PhdThesis,
    Misc,
    Proceedings,
    TechReport,
    Unpublished,
    MvBook,
    BookInBook,
    SuppBook,
    Periodical,
    SuppPeriodical,
    Collection,
    MvCollection,
    SuppCollection,
    Reference,
    MvReference,
    InReference,
    MvProceedings,
    Report,
    Patent,
    Thesis,
    Online,
    Software,
    Dataset,
    Set,
    XData,
}
